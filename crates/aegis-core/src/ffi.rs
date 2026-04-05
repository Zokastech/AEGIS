// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Shared FFI helpers (string allocation, last error). Stable C symbols live in `aegis-ffi`.

use crate::config::AnalysisConfig;
use crate::engine::{AnalyzerEngine, AnalyzerEngineBuilder};
use serde_json;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

static LAST_ERROR: Mutex<Option<CString>> = Mutex::new(None);

/// Stores the error message for [`ffi_last_error_ptr`] / `aegis_last_error` in `aegis-ffi`.
pub fn ffi_set_last_error(msg: &str) {
    if let Ok(mut g) = LAST_ERROR.lock() {
        *g = CString::new(msg.to_string()).ok();
    }
}

/// Pointer to the last UTF-8 error NUL-terminated, or null. Do not free.
pub fn ffi_last_error_ptr() -> *const c_char {
    match LAST_ERROR.lock() {
        Ok(g) => g.as_ref().map(|c| c.as_ptr()).unwrap_or(ptr::null()),
        Err(_) => ptr::null(),
    }
}

/// Creates an engine from engine YAML, or empty string + default regex recognizers (`en`, `fr`).
///
/// # Safety
///
/// `config_yaml` must be a valid NUL-terminated C string, or null (treated as empty config).
pub unsafe fn engine_create_boxed(config_yaml: *const c_char) -> Option<Box<AnalyzerEngine>> {
    let yaml = if config_yaml.is_null() {
        ""
    } else {
        // SAFETY: `config_yaml` is non-null; caller must pass a valid NUL-terminated C string.
        match unsafe { CStr::from_ptr(config_yaml) }.to_str() {
            Ok(s) => s,
            Err(_) => {
                ffi_set_last_error("config_yaml: UTF-8 invalide");
                return None;
            }
        }
    };
    let res = if yaml.trim().is_empty() {
        AnalyzerEngineBuilder::new()
            .with_default_recognizers(&["en", "fr"])
            .build()
    } else {
        match AnalyzerEngineBuilder::new().with_engine_yaml_str(yaml) {
            Ok(b) => b.build(),
            Err(e) => Err(e),
        }
    };
    match res {
        Ok(e) => Some(Box::new(e)),
        Err(e) => {
            ffi_set_last_error(&e.to_string());
            None
        }
    }
}

/// Analyze → JSON (`CString`). Free with C `aegis_free_string`.
///
/// # Safety
///
/// `text` must be a valid NUL-terminated C string. `engine` must reference a valid engine
/// (typically from [`engine_create_boxed`]).
pub unsafe fn engine_analyze_json_c(
    engine: &AnalyzerEngine,
    text: *const c_char,
    analysis: Option<AnalysisConfig>,
) -> Option<CString> {
    if text.is_null() {
        ffi_set_last_error("text null");
        return None;
    }
    // SAFETY: `text` checked non-null; caller must pass a valid NUL-terminated C string.
    let s = match unsafe { CStr::from_ptr(text) }.to_str() {
        Ok(x) => x,
        Err(_) => {
            ffi_set_last_error("text: UTF-8 invalide");
            return None;
        }
    };
    match engine.analyze(s, analysis) {
        Ok(r) => match serde_json::to_string(&r) {
            Ok(j) => match CString::new(j) {
                Ok(c) => Some(c),
                Err(_) => {
                    ffi_set_last_error("JSON contient NUL");
                    None
                }
            },
            Err(e) => {
                ffi_set_last_error(&e.to_string());
                None
            }
        },
        Err(e) => {
            ffi_set_last_error(&e.to_string());
            None
        }
    }
}

/// Frees a string allocated by AEGIS core (JSON).
///
/// # Safety
///
/// `p` must be null, or a pointer returned by this crate (e.g. analyze JSON) not yet freed.
pub unsafe fn ffi_string_free(p: *mut c_char) {
    if !p.is_null() {
        // SAFETY: `p` non-null; allocated by this crate, not freed before (see # Safety).
        drop(unsafe { CString::from_raw(p) });
    }
}
