// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Pont JNI vers le moteur AEGIS (`aegis-core` / `aegis-anonymize`).

use aegis_anonymize::{AnonymizationConfig, AnonymizerEngine};
use aegis_core::config::AnalysisConfig;
use aegis_core::engine::AnalyzerEngineBuilder;
use aegis_core::entity::{AnalysisResult as CoreAnalysis, Entity as CoreEntity, EntityType};
use aegis_core::ffi::ffi_set_last_error;
use jni::objects::{JClass, JString};
use std::ops::Deref;
use jni::sys::{jlong, jstring};
use jni::JNIEnv;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use aegis_regex as _;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct FfiAnonymizeConfig {
    #[serde(default)]
    analysis: AnalysisConfig,
    #[serde(default)]
    operators_by_entity: HashMap<String, aegis_core::anonymizer::OperatorConfig>,
    #[serde(default)]
    default_operator: Option<aegis_core::anonymizer::OperatorConfig>,
}

struct EngineInner {
    analyzer: aegis_core::AnalyzerEngine,
    anonymizer: AnonymizerEngine,
}

type EngineHandle = Mutex<Option<EngineInner>>;

fn set_err(msg: impl AsRef<str>) {
    ffi_set_last_error(msg.as_ref());
}

fn jstr_opt(env: &mut JNIEnv, js: &JString) -> jni::errors::Result<Option<String>> {
    if js.deref().is_null() {
        return Ok(None);
    }
    Ok(Some(env.get_string(js)?.into()))
}

fn parse_languages_json(s: Option<&str>) -> Result<Vec<String>, String> {
    match s {
        None | Some("") => Ok(vec!["en".into(), "fr".into()]),
        Some(raw) => {
            let v: Vec<String> =
                serde_json::from_str(raw.trim()).map_err(|e| format!("languages JSON: {e}"))?;
            if v.is_empty() {
                Ok(vec!["en".into(), "fr".into()])
            } else {
                Ok(v)
            }
        }
    }
}

fn build_engine(config_path: Option<&str>, langs: Vec<String>) -> Result<EngineInner, String> {
    let sl: Vec<&str> = langs.iter().map(|s| s.as_str()).collect();

    let analyzer = if let Some(p) = config_path.filter(|x| !x.is_empty()) {
        let path = Path::new(p);
        let content = fs::read_to_string(path).map_err(|e| format!("config_path {p}: {e}"))?;
        let lower = p.to_lowercase();
        let mut b = AnalyzerEngineBuilder::new().with_default_recognizers(&sl);
        b = if lower.ends_with(".json") {
            b.with_engine_json_str(&content)
                .map_err(|e| e.to_string())?
        } else {
            b.with_engine_yaml_str(&content)
                .map_err(|e| e.to_string())?
        };
        b.build().map_err(|e| e.to_string())?
    } else {
        AnalyzerEngineBuilder::new()
            .with_default_recognizers(&sl)
            .build()
            .map_err(|e| e.to_string())?
    };

    Ok(EngineInner {
        analyzer,
        anonymizer: AnonymizerEngine::new(),
    })
}

fn with_engine<F, R>(handle: jlong, f: F) -> Result<R, String>
where
    F: FnOnce(&EngineInner) -> Result<R, String>,
{
    if handle == 0 {
        return Err("handle null".into());
    }
    let mtx = unsafe { &*(handle as *const EngineHandle) };
    let g = mtx
        .lock()
        .map_err(|_| "engine mutex poisoned".to_string())?;
    let inner = g
        .as_ref()
        .ok_or_else(|| "engine closed".to_string())?;
    f(inner)
}

/// `Java_fr_zokastech_aegis_internal_JniBridge_nativeCreate`
#[no_mangle]
pub extern "system" fn Java_fr_zokastech_aegis_internal_JniBridge_nativeCreate<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    config_path: JString<'local>,
    languages_json: JString<'local>,
) -> jlong {
    let res = (|| -> Result<jlong, String> {
        let path = jstr_opt(&mut env, &config_path).map_err(|e| e.to_string())?;
        let langs_raw = jstr_opt(&mut env, &languages_json).map_err(|e| e.to_string())?;
        let langs = parse_languages_json(langs_raw.as_deref())?;
        let inner = build_engine(path.as_deref(), langs)?;
        let bx: Box<EngineHandle> = Box::new(Mutex::new(Some(inner)));
        Ok(Box::into_raw(bx) as jlong)
    })();
    match res {
        Ok(h) => {
            set_err("");
            h
        }
        Err(e) => {
            set_err(&e);
            0
        }
    }
}

/// `Java_fr_zokastech_aegis_internal_JniBridge_nativeDestroy`
#[no_mangle]
pub extern "system" fn Java_fr_zokastech_aegis_internal_JniBridge_nativeDestroy<'local>(
    mut _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    if handle == 0 {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle as *mut EngineHandle));
    }
}

fn entity_to_json_value(e: &CoreEntity) -> serde_json::Value {
    serde_json::json!({
        "entityType": e.entity_type.config_key(),
        "start": e.start,
        "end": e.end,
        "text": e.text,
        "score": e.score,
        "recognizerName": e.recognizer_name,
        "metadata": e.metadata,
    })
}

fn analysis_entities_json(r: &CoreAnalysis) -> String {
    let list: Vec<_> = r.entities.iter().map(entity_to_json_value).collect();
    serde_json::to_string(&list).unwrap_or_else(|_| "[]".into())
}

fn full_analysis_json(r: &CoreAnalysis) -> String {
    let v = serde_json::json!({
        "entities": r.entities.iter().map(entity_to_json_value).collect::<Vec<_>>(),
        "processingTimeMs": r.processing_time_ms,
        "languageDetected": r.language_detected,
        "textLength": r.text_length,
    });
    serde_json::to_string(&v).unwrap_or_else(|_| "{}".into())
}

/// `Java_fr_zokastech_aegis_internal_JniBridge_nativeAnalyze`
#[no_mangle]
pub extern "system" fn Java_fr_zokastech_aegis_internal_JniBridge_nativeAnalyze<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    text: JString<'local>,
    analysis_config_json: JString<'local>,
) -> jstring {
    let out = (|| -> Result<String, String> {
        let t = jstr_opt(&mut env, &text)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "text null".to_string())?;
        let cfg_json = jstr_opt(&mut env, &analysis_config_json).map_err(|e| e.to_string())?;
        let analysis_cfg: Option<AnalysisConfig> = match &cfg_json {
            None => None,
            Some(s) if s.trim().is_empty() => None,
            Some(s) => Some(
                serde_json::from_str(s).map_err(|e| format!("analysis config JSON: {e}"))?,
            ),
        };
        with_engine(handle, |eng| {
            let res = eng
                .analyzer
                .analyze(&t, analysis_cfg)
                .map_err(|e| e.to_string())?;
            Ok(analysis_entities_json(&res))
        })
    })();
    match out {
        Ok(s) => env
            .new_string(s)
            .map(|j| j.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        Err(e) => {
            set_err(&e);
            std::ptr::null_mut()
        }
    }
}

/// `Java_fr_zokastech_aegis_internal_JniBridge_nativeAnalyzeFull`
#[no_mangle]
pub extern "system" fn Java_fr_zokastech_aegis_internal_JniBridge_nativeAnalyzeFull<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    text: JString<'local>,
    analysis_config_json: JString<'local>,
) -> jstring {
    let out = (|| -> Result<String, String> {
        let t = jstr_opt(&mut env, &text)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "text null".to_string())?;
        let cfg_json = jstr_opt(&mut env, &analysis_config_json).map_err(|e| e.to_string())?;
        let analysis_cfg: Option<AnalysisConfig> = match &cfg_json {
            None => None,
            Some(s) if s.trim().is_empty() => None,
            Some(s) => Some(
                serde_json::from_str(s).map_err(|e| format!("analysis config JSON: {e}"))?,
            ),
        };
        with_engine(handle, |eng| {
            let res = eng
                .analyzer
                .analyze(&t, analysis_cfg)
                .map_err(|e| e.to_string())?;
            Ok(full_analysis_json(&res))
        })
    })();
    match out {
        Ok(s) => env
            .new_string(s)
            .map(|j| j.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        Err(e) => {
            set_err(&e);
            std::ptr::null_mut()
        }
    }
}

/// `Java_fr_zokastech_aegis_internal_JniBridge_nativeAnalyzeBatch`
#[no_mangle]
pub extern "system" fn Java_fr_zokastech_aegis_internal_JniBridge_nativeAnalyzeBatch<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    texts_json: JString<'local>,
) -> jstring {
    let out = (|| -> Result<String, String> {
        let raw = jstr_opt(&mut env, &texts_json)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "texts null".to_string())?;
        let texts: Vec<String> =
            serde_json::from_str(&raw).map_err(|e| format!("texts JSON: {e}"))?;
        with_engine(handle, |eng| {
            let mut batch = Vec::with_capacity(texts.len());
            for t in &texts {
                let res = eng.analyzer.analyze(t, None).map_err(|e| e.to_string())?;
                batch.push(
                    res.entities
                        .iter()
                        .map(entity_to_json_value)
                        .collect::<Vec<_>>(),
                );
            }
            serde_json::to_string(&batch).map_err(|e| e.to_string())
        })
    })();
    match out {
        Ok(s) => env
            .new_string(s)
            .map(|j| j.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        Err(e) => {
            set_err(&e);
            std::ptr::null_mut()
        }
    }
}

/// `Java_fr_zokastech_aegis_internal_JniBridge_nativeAnonymize`
#[no_mangle]
pub extern "system" fn Java_fr_zokastech_aegis_internal_JniBridge_nativeAnonymize<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    text: JString<'local>,
    operators_json: JString<'local>,
) -> jstring {
    let out = (|| -> Result<String, String> {
        let t = jstr_opt(&mut env, &text)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "text null".to_string())?;
        let json = jstr_opt(&mut env, &operators_json)
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        let cfg: FfiAnonymizeConfig = if json.trim().is_empty() || json.trim() == "{}" {
            FfiAnonymizeConfig::default()
        } else {
            let mut raw: serde_json::Value =
                serde_json::from_str(&json).map_err(|e| format!("operators JSON: {e}"))?;
            if let Some(m) = raw.as_object() {
                if !m.contains_key("operators_by_entity")
                    && !m.contains_key("analysis")
                    && !m.contains_key("default_operator")
                {
                    raw = serde_json::json!({ "operators_by_entity": raw });
                }
            }
            serde_json::from_value(raw).map_err(|e| format!("operators JSON: {e}"))?
        };

        with_engine(handle, |eng| {
            let analyzed = eng
                .analyzer
                .analyze(&t, Some(cfg.analysis))
                .map_err(|e| e.to_string())?;
            let anon_cfg = AnonymizationConfig {
                operators_by_entity: cfg.operators_by_entity,
                default_operator: cfg.default_operator,
            };
            let out = eng
                .anonymizer
                .anonymize(&t, &analyzed.entities, &anon_cfg);
            let v = serde_json::json!({
                "text": out.text,
                "transformations": out.transformations.iter().map(|tr| serde_json::json!({
                    "entityStart": tr.entity_start,
                    "entityEnd": tr.entity_end,
                    "originalText": tr.original_text,
                    "replacement": tr.replacement,
                    "operator": tr.operator,
                    "entityType": tr.entity_type,
                })).collect::<Vec<_>>(),
                "keyIdsUsed": out.key_ids_used,
                "mappingHints": out.mapping_hints,
            });
            serde_json::to_string(&v).map_err(|e| e.to_string())
        })
    })();
    match out {
        Ok(s) => env
            .new_string(s)
            .map(|j| j.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        Err(e) => {
            set_err(&e);
            std::ptr::null_mut()
        }
    }
}

/// `Java_fr_zokastech_aegis_internal_JniBridge_nativeLastError`
#[no_mangle]
pub extern "system" fn Java_fr_zokastech_aegis_internal_JniBridge_nativeLastError<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    let ptr = aegis_core::ffi::ffi_last_error_ptr();
    let msg = if ptr.is_null() {
        ""
    } else {
        unsafe {
            std::ffi::CStr::from_ptr(ptr)
                .to_str()
                .unwrap_or("")
        }
    };
    env.new_string(msg)
        .map(|j| j.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// `Java_fr_zokastech_aegis_internal_JniBridge_nativeVersion`
#[no_mangle]
pub extern "system" fn Java_fr_zokastech_aegis_internal_JniBridge_nativeVersion<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    let v = env!("CARGO_PKG_VERSION");
    env.new_string(v)
        .map(|j| j.into_raw())
        .unwrap_or(std::ptr::null_mut())
}
