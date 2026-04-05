// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! SARIF 2.1.0 output for SAST / GitHub Advanced Security integration.

use aegis_core::entity::{AnalysisResult, Entity};
use serde_json::{json, Value};
use std::collections::HashSet;

fn entity_rule_id(e: &Entity) -> String {
    format!("AEGIS/{}", e.entity_type.config_key())
}

fn byte_offset_to_line_col(s: &str, byte_idx: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    let mut i = 0usize;
    for ch in s.chars() {
        if i >= byte_idx {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += ch.len_utf8();
    }
    (line, col)
}

fn region_for_entity(source: &str, e: &Entity) -> Value {
    let start = e.start.min(source.len());
    let end = e.end.min(source.len()).max(start);
    let (sl, sc) = byte_offset_to_line_col(source, start);
    let (el, ec) = byte_offset_to_line_col(source, end);
    json!({
        "startLine": sl,
        "startColumn": sc,
        "endLine": el,
        "endColumn": ec,
        "charOffset": start,
        "charLength": end.saturating_sub(start),
    })
}

fn sarif_rule_object_id(v: &Value) -> &str {
    v.get("id").and_then(Value::as_str).unwrap_or("")
}

fn collect_rules(keys: &HashSet<String>) -> Vec<Value> {
    let mut rules: Vec<Value> = keys
        .iter()
        .map(|k| {
            json!({
                "id": format!("AEGIS/{k}"),
                "name": format!("Potential PII: {k}"),
                "shortDescription": { "text": format!("AEGIS detected sensitive data ({k})") },
                "fullDescription": { "text": format!("The AEGIS engine reported a {k} entity. Review before publishing.") },
                "defaultConfiguration": { "level": "warning" }
            })
        })
        .collect();
    rules.sort_by(|a, b| sarif_rule_object_id(a).cmp(sarif_rule_object_id(b)));
    rules
}

/// Builds an aggregated SARIF document for one or more analyzed files.
pub fn build_document(tool_version: &str, files: &[(String, String, AnalysisResult)]) -> Value {
    let mut type_keys: HashSet<String> = HashSet::new();
    let mut results: Vec<Value> = Vec::new();

    for (uri, text, res) in files {
        for e in &res.entities {
            let rid = entity_rule_id(e);
            type_keys.insert(e.entity_type.config_key());
            results.push(json!({
                "ruleId": rid,
                "level": "warning",
                "message": {
                    "text": format!(
                        "{} (score {:.3}, recognizer {})",
                        e.entity_type.config_key(),
                        e.score,
                        e.recognizer_name
                    )
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": uri },
                        "region": region_for_entity(text, e)
                    }
                }]
            }));
        }
    }

    let rules = collect_rules(&type_keys);

    json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "AEGIS",
                    "semanticVersion": tool_version,
                    "informationUri": "https://zokastech.fr",
                    "rules": rules
                }
            },
            "results": results
        }]
    })
}
