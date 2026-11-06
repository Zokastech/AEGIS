// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! **AEGIS** C library: `cdylib` / `staticlib` for Python (ctypes / PyO3), Node (N-API), Go (cgo), etc.
#![allow(clippy::missing_safety_doc)] // C ABI : contrats documentés côté headers / intégrations.

// Dummy reference: without this, the linker may drop `aegis-regex` and the `ctor` that registers
// default recognizers never runs → “no recognizers registered” at init.
use aegis_regex as _;

use aegis_anonymize::{AnonymizationConfig, AnonymizerEngine};
use aegis_core::anonymizer::OperatorConfig;
use aegis_core::config::AnalysisConfig;
use aegis_core::engine::AnalyzerEngineBuilder;
use aegis_core::entity::AnalysisResult;
use aegis_core::ffi::{
    engine_analyze_json_c, ffi_last_error_ptr, ffi_set_last_error, ffi_string_free,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

#[cfg(feature = "python")]
pub mod python_shim;

#[cfg(feature = "nodejs")]
pub mod nodejs_shim;

/// Opaque handle on the C side (never instantiated in Rust).
pub enum AegisHandle {}

struct AegisContext {
    analyzer: aegis_core::AnalyzerEngine,
    anonymizer: AnonymizerEngine,
}

fn panic_to_last_error() {
    ffi_set_last_error("panic Rust interceptée à la frontière FFI");
}

fn cstr_to_str<'a>(ptr: *const c_char, field: &'static str) -> Option<&'a str> {
    if ptr.is_null() {
        ffi_set_last_error(&format!("{field} : pointeur null"));
        return None;
    }
    match unsafe { CStr::from_ptr(ptr) }.to_str() {
        Ok(s) => Some(s),
        Err(_) => {
            ffi_set_last_error(&format!("{field} : UTF-8 invalide"));
            None
        }
    }
}

/// Creates an engine from JSON config (same schema as `aegis-config.yaml` in JSON).
/// Empty string or `NULL` → default regex recognizers `en` + `fr` (if `aegis-regex` is linked).
#[no_mangle]
pub unsafe extern "C" fn aegis_init(config_json: *const c_char) -> *mut AegisHandle {
    catch_unwind(AssertUnwindSafe(|| init_inner(config_json))).unwrap_or_else(|_| {
        panic_to_last_error();
        ptr::null_mut()
    })
}

unsafe fn init_inner(config_json: *const c_char) -> *mut AegisHandle {
    ffi_set_last_error("");
    #[cfg(feature = "ner-onnx")]
    {
        static REGISTER_ONNX_NER: std::sync::Once = std::sync::Once::new();
        REGISTER_ONNX_NER.call_once(|| {
            aegis_core::try_register_onnx_ner_loader(aegis_ner::onnx_ner_backend_loader);
        });
    }
    let json = if config_json.is_null() {
        ""
    } else {
        match CStr::from_ptr(config_json).to_str() {
            Ok(s) => s,
            Err(_) => {
                ffi_set_last_error("aegis_init: config_json UTF-8 invalide");
                return ptr::null_mut();
            }
        }
    };
    let built = if json.trim().is_empty() {
        AnalyzerEngineBuilder::new()
            .with_default_recognizers(&["en", "fr"])
            .build()
    } else {
        match AnalyzerEngineBuilder::new().with_engine_json_str(json) {
            Ok(b) => b.build(),
            Err(e) => {
                ffi_set_last_error(&e.to_string());
                return ptr::null_mut();
            }
        }
    };
    let analyzer = match built {
        Ok(a) => a,
        Err(e) => {
            ffi_set_last_error(&e.to_string());
            return ptr::null_mut();
        }
    };
    let ctx = Box::new(AegisContext {
        analyzer,
        anonymizer: AnonymizerEngine::new(),
    });
    Box::into_raw(ctx) as *mut AegisHandle
}

/// Analyzes text; returns UTF-8 JSON to free with [`aegis_free_string`].
/// `config_json` may be `NULL` or `{}` for engine defaults.
#[no_mangle]
pub unsafe extern "C" fn aegis_analyze(
    handle: *mut AegisHandle,
    text: *const c_char,
    config_json: *const c_char,
) -> *mut c_char {
    catch_unwind(AssertUnwindSafe(|| {
        analyze_inner(handle, text, config_json)
    }))
    .unwrap_or_else(|_| {
        panic_to_last_error();
        ptr::null_mut()
    })
}

unsafe fn analyze_inner(
    handle: *mut AegisHandle,
    text: *const c_char,
    config_json: *const c_char,
) -> *mut c_char {
    ffi_set_last_error("");
    if handle.is_null() {
        ffi_set_last_error("aegis_analyze: handle null");
        return ptr::null_mut();
    }
    let ctx = &*(handle as *mut AegisContext);
    let analysis = if config_json.is_null() {
        None
    } else {
        match CStr::from_ptr(config_json).to_str() {
            Ok(s) if s.trim().is_empty() => None,
            Ok(s) => match serde_json::from_str::<AnalysisConfig>(s) {
                Ok(c) => Some(c),
                Err(e) => {
                    ffi_set_last_error(&format!("aegis_analyze: config_json {e}"));
                    return ptr::null_mut();
                }
            },
            Err(_) => {
                ffi_set_last_error("aegis_analyze: config_json UTF-8 invalide");
                return ptr::null_mut();
            }
        }
    };
    match engine_analyze_json_c(&ctx.analyzer, text, analysis) {
        Some(c) => c.into_raw(),
        None => ptr::null_mut(),
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct FfiAnonymizeConfig {
    #[serde(default)]
    analysis: AnalysisConfig,
    /// Keys in the form `EMAIL`, `PHONE`, etc. ([`EntityType::config_key`]).
    #[serde(default)]
    operators_by_entity: HashMap<String, OperatorConfig>,
    #[serde(default)]
    default_operator: Option<OperatorConfig>,
}

/// Detects entities then anonymizes. `config_json` may include `analysis`, `operators_by_entity`, `default_operator`.
#[no_mangle]
pub unsafe extern "C" fn aegis_anonymize(
    handle: *mut AegisHandle,
    text: *const c_char,
    config_json: *const c_char,
) -> *mut c_char {
    catch_unwind(AssertUnwindSafe(|| {
        anonymize_inner(handle, text, config_json)
    }))
    .unwrap_or_else(|_| {
        panic_to_last_error();
        ptr::null_mut()
    })
}

unsafe fn anonymize_inner(
    handle: *mut AegisHandle,
    text: *const c_char,
    config_json: *const c_char,
) -> *mut c_char {
    ffi_set_last_error("");
    if handle.is_null() {
        ffi_set_last_error("aegis_anonymize: handle null");
        return ptr::null_mut();
    }
    let ctx = &*(handle as *mut AegisContext);
    let t = match cstr_to_str(text, "text") {
        Some(s) => s,
        None => return ptr::null_mut(),
    };
    let cfg = if config_json.is_null() {
        FfiAnonymizeConfig::default()
    } else {
        let sj = match CStr::from_ptr(config_json).to_str() {
            Ok(s) => s,
            Err(_) => {
                ffi_set_last_error("aegis_anonymize: config_json UTF-8 invalide");
                return ptr::null_mut();
            }
        };
        if sj.trim().is_empty() {
            FfiAnonymizeConfig::default()
        } else {
            match serde_json::from_str::<FfiAnonymizeConfig>(sj) {
                Ok(c) => c,
                Err(e) => {
                    ffi_set_last_error(&format!("aegis_anonymize: config_json {e}"));
                    return ptr::null_mut();
                }
            }
        }
    };
    let analyzed = match ctx.analyzer.analyze(t, Some(cfg.analysis)) {
        Ok(r) => r,
        Err(e) => {
            ffi_set_last_error(&e.to_string());
            return ptr::null_mut();
        }
    };
    let anon_cfg = AnonymizationConfig {
        operators_by_entity: cfg.operators_by_entity,
        default_operator: cfg.default_operator,
    };
    let out = ctx.anonymizer.anonymize(t, &analyzed.entities, &anon_cfg);
    #[derive(serde::Serialize)]
    struct AnonymizeFfiOut {
        anonymized: aegis_anonymize::AnonymizedResult,
        analysis: AnalysisResult,
    }
    let payload = AnonymizeFfiOut {
        anonymized: out,
        analysis: analyzed,
    };
    match serde_json::to_string(&payload) {
        Ok(j) => match CString::new(j) {
            Ok(c) => c.into_raw(),
            Err(_) => {
                ffi_set_last_error("aegis_anonymize: JSON contient NUL");
                ptr::null_mut()
            }
        },
        Err(e) => {
            ffi_set_last_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

/// Batch analyze: `texts_json` is a JSON array of strings, e.g. `["a","b"]`. Returns a JSON array of results.
#[no_mangle]
pub unsafe extern "C" fn aegis_analyze_batch(
    handle: *mut AegisHandle,
    texts_json: *const c_char,
) -> *mut c_char {
    catch_unwind(AssertUnwindSafe(|| batch_inner(handle, texts_json))).unwrap_or_else(|_| {
        panic_to_last_error();
        ptr::null_mut()
    })
}

unsafe fn batch_inner(handle: *mut AegisHandle, texts_json: *const c_char) -> *mut c_char {
    ffi_set_last_error("");
    if handle.is_null() {
        ffi_set_last_error("aegis_analyze_batch: handle null");
        return ptr::null_mut();
    }
    let ctx = &*(handle as *mut AegisContext);
    let raw = match cstr_to_str(texts_json, "texts_json") {
        Some(s) => s,
        None => return ptr::null_mut(),
    };
    let texts: Vec<String> = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(e) => {
            ffi_set_last_error(&format!("aegis_analyze_batch: texts_json {e}"));
            return ptr::null_mut();
        }
    };
    let mut results: Vec<AnalysisResult> = Vec::with_capacity(texts.len());
    for t in &texts {
        match ctx.analyzer.analyze(t.as_str(), None) {
            Ok(r) => results.push(r),
            Err(e) => {
                ffi_set_last_error(&e.to_string());
                return ptr::null_mut();
            }
        }
    }
    match serde_json::to_string(&results) {
        Ok(j) => match CString::new(j) {
            Ok(c) => c.into_raw(),
            Err(_) => {
                ffi_set_last_error("aegis_analyze_batch: JSON contient NUL");
                ptr::null_mut()
            }
        },
        Err(e) => {
            ffi_set_last_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

/// Frees a string returned by `aegis_analyze`, `aegis_analyze_batch`, or `aegis_anonymize`.
#[no_mangle]
pub unsafe extern "C" fn aegis_free_string(ptr: *mut c_char) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        ffi_string_free(ptr);
    }));
}

/// Frees a handle created by [`aegis_init`].
#[no_mangle]
pub unsafe extern "C" fn aegis_free(handle: *mut AegisHandle) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !handle.is_null() {
            drop(Box::from_raw(handle as *mut AegisContext));
        }
    }));
}

/// Last error as UTF-8 (NUL-terminated), or empty string; do not free.
#[no_mangle]
pub extern "C" fn aegis_last_error() -> *const c_char {
    catch_unwind(AssertUnwindSafe(ffi_last_error_ptr)).unwrap_or(ptr::null())
}

/// `aegis-ffi` crate version (semver), static string; do not free.
#[no_mangle]
pub extern "C" fn aegis_version() -> *const c_char {
    catch_unwind(AssertUnwindSafe(|| {
        concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
    }))
    .unwrap_or(ptr::null())
}

/// Returns 1 if NER ONNX is not required or is loaded; 0 on failure (`aegis_last_error()`).
#[no_mangle]
pub unsafe extern "C" fn aegis_ner_onnx_ready(handle: *mut AegisHandle) -> c_int {
    catch_unwind(AssertUnwindSafe(|| ner_onnx_ready_inner(handle))).unwrap_or_else(|_| {
        panic_to_last_error();
        0
    })
}

fn ner_onnx_ready_inner(handle: *mut AegisHandle) -> c_int {
    ffi_set_last_error("");
    if handle.is_null() {
        ffi_set_last_error("aegis_ner_onnx_ready: handle null");
        return 0;
    }
    let ctx = unsafe { &*(handle as *mut AegisContext) };
    match ctx.analyzer.ensure_ner_onnx_ready() {
        Ok(()) => 1,
        Err(e) => {
            ffi_set_last_error(&e.to_string());
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    unsafe fn sample_engine() -> *mut AegisHandle {
        let h = aegis_init(ptr::null());
        assert!(!h.is_null());
        h
    }

    #[test]
    fn c_style_init_free() {
        unsafe {
            let h = sample_engine();
            aegis_free(h);
        }
    }

    #[test]
    fn c_style_analyze_returns_json() {
        unsafe {
            let h = sample_engine();
            let text = CString::new("contact x@y.co").unwrap();
            let out = aegis_analyze(h, text.as_ptr(), ptr::null());
            assert!(!out.is_null());
            let s = CStr::from_ptr(out).to_str().unwrap();
            assert!(s.contains("entities"));
            aegis_free_string(out);
            aegis_free(h);
        }
    }

    #[test]
    fn c_style_analyze_with_partial_config_json() {
        unsafe {
            let h = sample_engine();
            let text = CString::new("x@y.co").unwrap();
            let cfg = CString::new(r#"{"score_threshold":0.2}"#).unwrap();
            let out = aegis_analyze(h, text.as_ptr(), cfg.as_ptr());
            assert!(!out.is_null());
            aegis_free_string(out);
            aegis_free(h);
        }
    }

    #[test]
    fn c_style_batch() {
        unsafe {
            let h = sample_engine();
            let arr = CString::new(r#"["a@b.co","nothing"]"#).unwrap();
            let out = aegis_analyze_batch(h, arr.as_ptr());
            assert!(!out.is_null());
            let s = CStr::from_ptr(out).to_str().unwrap();
            assert!(s.starts_with('['));
            aegis_free_string(out);
            aegis_free(h);
        }
    }

    #[test]
    fn c_style_anonymize() {
        unsafe {
            let h = sample_engine();
            let text = CString::new("mail x@y.co end").unwrap();
            let cfg = CString::new(
                r#"{"analysis":{"score_threshold":0.2},"operators_by_entity":{"EMAIL":{"operator_type":"redact","params":{}}}}"#,
            )
            .unwrap();
            let out = aegis_anonymize(h, text.as_ptr(), cfg.as_ptr());
            assert!(!out.is_null());
            let s = CStr::from_ptr(out).to_str().unwrap();
            assert!(s.contains("anonymized"));
            aegis_free_string(out);
            aegis_free(h);
        }
    }

    #[test]
    fn c_style_last_error_on_bad_json_init() {
        unsafe {
            let bad = CString::new("{not json").unwrap();
            let h = aegis_init(bad.as_ptr());
            assert!(h.is_null());
            let e = aegis_last_error();
            assert!(!e.is_null());
            let msg = CStr::from_ptr(e).to_str().unwrap();
            assert!(!msg.is_empty());
        }
    }

    #[test]
    fn ffi_version_mean_latency_under_point_one_ms() {
        let n = 10_000u64;
        let t0 = Instant::now();
        for _ in 0..n {
            let p = aegis_version();
            assert!(!p.is_null());
            let _ = unsafe { CStr::from_ptr(p) };
        }
        let mean_ns = t0.elapsed().as_nanos() as f64 / n as f64;
        assert!(
            mean_ns < 100_000.0,
            "moyenne {:.0} ns / appel (limite 100 µs = 0,1 ms)",
            mean_ns
        );
    }

    fn last_error() -> String {
        let p = aegis_last_error();
        if p.is_null() {
            return String::new();
        }
        unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
    }

    #[test]
    fn null_pointers_are_rejected_without_crash() {
        unsafe {
            let txt = std::ffi::CString::new("a@b.co").unwrap();
            assert!(aegis_analyze(ptr::null_mut(), txt.as_ptr(), ptr::null()).is_null());
            assert!(last_error().contains("handle"));
            assert!(aegis_anonymize(ptr::null_mut(), txt.as_ptr(), ptr::null()).is_null());
            assert!(aegis_analyze_batch(ptr::null_mut(), txt.as_ptr()).is_null());
            assert_eq!(aegis_ner_onnx_ready(ptr::null_mut()), 0);
            let h = sample_engine();
            assert!(aegis_analyze(h, ptr::null(), ptr::null()).is_null());
            aegis_free(h);
            aegis_free(ptr::null_mut());
            aegis_free_string(ptr::null_mut());
        }
    }

    #[test]
    fn invalid_utf8_and_json_are_errors() {
        unsafe {
            let h = sample_engine();
            let bad = [0xffu8, 0xfe, 0x00];
            let out = aegis_analyze(h, bad.as_ptr() as *const c_char, ptr::null());
            assert!(out.is_null(), "UTF-8 invalide accepté");
            let txt = std::ffi::CString::new("x").unwrap();
            let cfg = std::ffi::CString::new("{not json").unwrap();
            let out = aegis_analyze(h, txt.as_ptr(), cfg.as_ptr());
            if !out.is_null() {
                aegis_free_string(out);
            } else {
                assert!(!last_error().is_empty());
            }
            let batch = std::ffi::CString::new("not an array").unwrap();
            assert!(aegis_analyze_batch(h, batch.as_ptr()).is_null());
            aegis_free(h);
        }
    }

    #[test]
    fn invalid_init_config_returns_null() {
        unsafe {
            let cfg = std::ffi::CString::new("{\"unknown\": [").unwrap();
            let h = aegis_init(cfg.as_ptr());
            assert!(h.is_null());
            assert!(!last_error().is_empty());
        }
    }

    #[test]
    fn version_is_static_string() {
        let v = aegis_version();
        assert!(!v.is_null());
        assert!(!unsafe { CStr::from_ptr(v) }.to_str().unwrap().is_empty());
    }
}
