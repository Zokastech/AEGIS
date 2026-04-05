// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Orchestration: per-entity operator, transformation log, deanonymization.

use crate::encrypt::{reverse_metadata_from_enc, EncryptOperator, ENC_PREFIX};
use crate::error::{AnonymizeError, Result};
use crate::fpe_op::{fpe_digits_transform, split_fpe_token, FpeOperator, FPE_PREFIX};
use crate::hash_op::HashOperator;
use crate::mask::MaskOperator;
use crate::pseudonymize::PseudonymizeOperator;
use crate::redact::RedactOperator;
use crate::replace::ReplaceOperator;
use crate::types::{AnonymizationConfig, AnonymizedResult, ReverseMetadata, TransformationRecord};
use aegis_core::anonymizer::{Operator, OperatorConfig, OperatorType};
use aegis_core::entity::{Entity, EntityType};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

struct Step {
    start: usize,
    end: usize,
    entity_type: EntityType,
    original: String,
    replacement: String,
    operator: String,
    reverse: Option<ReverseMetadata>,
}

pub struct AnonymizerEngine {
    redact: Arc<RedactOperator>,
    replace: Arc<ReplaceOperator>,
    mask: Arc<MaskOperator>,
    hash: Arc<HashOperator>,
    encrypt: Arc<EncryptOperator>,
    fpe: Arc<FpeOperator>,
    pseudonym: Arc<PseudonymizeOperator>,
}

impl Default for AnonymizerEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AnonymizerEngine {
    pub fn new() -> Self {
        Self {
            redact: Arc::new(RedactOperator),
            replace: Arc::new(ReplaceOperator),
            mask: Arc::new(MaskOperator),
            hash: Arc::new(HashOperator),
            encrypt: Arc::new(EncryptOperator),
            fpe: Arc::new(FpeOperator),
            pseudonym: Arc::new(PseudonymizeOperator),
        }
    }

    fn pick_op(&self, oc: &OperatorConfig) -> Arc<dyn Operator> {
        match oc.operator_type {
            OperatorType::Redact => self.redact.clone(),
            OperatorType::Replace => self.replace.clone(),
            OperatorType::Mask => self.mask.clone(),
            OperatorType::Hash => self.hash.clone(),
            OperatorType::Encrypt => self.encrypt.clone(),
            OperatorType::Fpe => self.fpe.clone(),
            OperatorType::Pseudonymize => self.pseudonym.clone(),
            _ => self.redact.clone(),
        }
    }

    fn resolve_config<'a>(e: &Entity, config: &'a AnonymizationConfig) -> Cow<'a, OperatorConfig> {
        let key = e.entity_type.config_key();
        if let Some(c) = config.operators_by_entity.get(&key) {
            return Cow::Borrowed(c);
        }
        if let Some(c) = config.operators_by_entity.get("*") {
            return Cow::Borrowed(c);
        }
        if let Some(ref c) = config.default_operator {
            return Cow::Borrowed(c);
        }
        Cow::Owned(OperatorConfig {
            operator_type: OperatorType::Replace,
            params: [("tag".into(), "PII".into())].into_iter().collect(),
        })
    }

    fn build_reverse(
        op_type: OperatorType,
        replacement: &str,
        cfg: &OperatorConfig,
    ) -> Option<ReverseMetadata> {
        let key_id = cfg
            .params
            .get("key_id")
            .cloned()
            .unwrap_or_else(|| "default".into());
        let aad = cfg.params.get("aad").cloned().unwrap_or_default();
        match op_type {
            OperatorType::Encrypt => {
                if replacement.starts_with(ENC_PREFIX) && !replacement.contains("ERR") {
                    reverse_metadata_from_enc(replacement, key_id, aad)
                } else {
                    None
                }
            }
            OperatorType::Fpe => {
                if replacement.starts_with(FPE_PREFIX) && !replacement.contains("ERR") {
                    split_fpe_token(replacement)
                        .map(|(nonce, _)| ReverseMetadata::FpeDigitsV1 { key_id, nonce })
                } else {
                    None
                }
            }
            OperatorType::Pseudonymize | OperatorType::Replace => Some(ReverseMetadata::LedgerOnly),
            _ => None,
        }
    }

    /// Applies configured operators (end → start of source text for stable indices).
    pub fn anonymize(
        &self,
        text: &str,
        entities: &[Entity],
        config: &AnonymizationConfig,
    ) -> AnonymizedResult {
        let mut sorted: Vec<&Entity> = entities.iter().collect();
        sorted.sort_by_key(|e| e.start);

        let mut serial_by_type: HashMap<String, u64> = HashMap::new();
        let mut buf = text.to_string();
        let mut steps: Vec<Step> = Vec::new();
        let mut key_ids_used: HashSet<String> = HashSet::new();

        for e in sorted.iter().rev() {
            if e.end > buf.len() || e.start > e.end {
                continue;
            }
            let mut cfg = Self::resolve_config(e, config).into_owned();
            if matches!(cfg.operator_type, OperatorType::Replace)
                && cfg
                    .params
                    .get("numbered")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false)
            {
                let k = e.entity_type.config_key();
                let n = serial_by_type.entry(k).or_insert(0);
                *n += 1;
                cfg.params.insert("__serial__".into(), n.to_string());
            }
            if matches!(cfg.operator_type, OperatorType::Encrypt | OperatorType::Fpe) {
                let kid = cfg
                    .params
                    .get("key_id")
                    .cloned()
                    .unwrap_or_else(|| "default".into());
                key_ids_used.insert(kid);
            }

            let op = self.pick_op(&cfg);
            let slice = buf[e.start..e.end].to_string();
            let mut ent = (*e).clone();
            ent.text = slice.clone();
            let replacement = op.operate(&ent, &slice, &cfg);
            let operator = format!("{:?}", cfg.operator_type);
            let reverse = Self::build_reverse(cfg.operator_type.clone(), &replacement, &cfg);

            steps.push(Step {
                start: e.start,
                end: e.end,
                entity_type: e.entity_type.clone(),
                original: slice.clone(),
                replacement: replacement.clone(),
                operator,
                reverse,
            });
            buf.replace_range(e.start..e.end, &replacement);
        }

        let mut steps_asc = steps;
        steps_asc.sort_by_key(|s| s.start);

        let mut cursor = 0usize;
        let mut pos = 0usize;
        let mut transformations = Vec::new();
        for s in steps_asc {
            cursor += s.start.saturating_sub(pos);
            let fs = cursor;
            cursor += s.replacement.len();
            let fe = cursor;
            pos = s.end;
            transformations.push(TransformationRecord {
                entity_start: s.start,
                entity_end: s.end,
                original_text: s.original,
                replacement: s.replacement,
                operator: s.operator,
                entity_type: s.entity_type.config_key(),
                final_start: Some(fs),
                final_end: Some(fe),
                reverse: s.reverse,
            });
        }

        AnonymizedResult {
            text: buf,
            transformations,
            key_ids_used: key_ids_used.into_iter().collect(),
            mapping_hints: HashMap::new(),
        }
    }

    /// Deanonymization with one key applied to every `key_id` present in the result.
    pub fn deanonymize(result: &AnonymizedResult, key: &[u8]) -> Result<String> {
        if key.len() != 32 {
            return Err(AnonymizeError::InvalidKey(
                "AES-256 / FPE attendent une clé de 32 octets".into(),
            ));
        }
        let mut m: HashMap<String, Vec<u8>> = HashMap::new();
        m.insert("default".into(), key.to_vec());
        for kid in &result.key_ids_used {
            m.insert(kid.clone(), key.to_vec());
        }
        Self::deanonymize_with_key_map(result, &m)
    }

    /// Deanonymization with a key map (rotation: multiple `key_id` values).
    pub fn deanonymize_with_key_map(
        result: &AnonymizedResult,
        keys: &HashMap<String, Vec<u8>>,
    ) -> Result<String> {
        let mut sorted = result.transformations.clone();
        sorted.sort_by_key(|t| {
            (
                std::cmp::Reverse(t.final_start.unwrap_or(0)),
                std::cmp::Reverse(t.final_end.unwrap_or(0)),
            )
        });

        let mut s = result.text.clone();
        for t in sorted {
            let (fs, fe) = match (t.final_start, t.final_end) {
                (Some(a), Some(b)) => (a, b),
                _ => return Err(AnonymizeError::MissingFinalSpan),
            };
            if fe > s.len() || fs > fe {
                return Err(AnonymizeError::ReplacementMismatch { start: fs, end: fe });
            }
            if s.get(fs..fe) != Some(t.replacement.as_str()) {
                return Err(AnonymizeError::ReplacementMismatch { start: fs, end: fe });
            }
            let orig = Self::resolve_original(&t, keys)?;
            s.replace_range(fs..fe, &orig);
        }
        Ok(s)
    }

    fn resolve_original(
        t: &TransformationRecord,
        keys: &HashMap<String, Vec<u8>>,
    ) -> Result<String> {
        match &t.reverse {
            Some(ReverseMetadata::AesGcmV1 {
                key_id,
                nonce,
                ciphertext,
                aad,
            }) => {
                let kv = keys
                    .get(key_id)
                    .or_else(|| keys.get("default"))
                    .ok_or_else(|| AnonymizeError::MissingKeyId(key_id.clone()))?;
                if kv.len() != 32 {
                    return Err(AnonymizeError::InvalidKey(key_id.clone()));
                }
                let mut kb = [0u8; 32];
                kb.copy_from_slice(kv);
                let pt = EncryptOperator::decrypt_blob(&kb, nonce, ciphertext, aad.as_bytes())
                    .map_err(AnonymizeError::DecryptFailed)?;
                String::from_utf8(pt)
                    .map_err(|e| AnonymizeError::DecryptFailed(format!("utf-8: {e}")))
            }
            Some(ReverseMetadata::FpeDigitsV1 { key_id, nonce }) => {
                let (_, body) = split_fpe_token(&t.replacement)
                    .ok_or_else(|| AnonymizeError::DecryptFailed("jeton FPE".into()))?;
                let kv = keys
                    .get(key_id)
                    .or_else(|| keys.get("default"))
                    .ok_or_else(|| AnonymizeError::MissingKeyId(key_id.clone()))?;
                if kv.len() != 32 {
                    return Err(AnonymizeError::InvalidKey(key_id.clone()));
                }
                let mut kb = [0u8; 32];
                kb.copy_from_slice(kv);
                Ok(fpe_digits_transform(&body, &kb, nonce, true))
            }
            _ => Ok(t.original_text.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_core::entity::EntityType;
    use std::collections::HashMap;

    const KEY32: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    #[test]
    fn person_pseudonym_email_redact_iban_fpe_fr() {
        let engine = AnonymizerEngine::new();
        let text = "Contact: Marie Dubois marie.synth@mail.fr IBAN FR7630006000011234567890189";
        let entities = vec![
            Entity {
                entity_type: EntityType::Person,
                start: 9,
                end: 20,
                text: "Marie Dubois".into(),
                score: 1.0,
                recognizer_name: "t".into(),
                metadata: HashMap::new(),
                decision_trace: None,
            },
            Entity {
                entity_type: EntityType::Email,
                start: 21,
                end: 40,
                text: "marie.synth@mail.fr".into(),
                score: 1.0,
                recognizer_name: "t".into(),
                metadata: HashMap::new(),
                decision_trace: None,
            },
            Entity {
                entity_type: EntityType::Iban,
                start: 46,
                end: 73,
                text: "FR7630006000011234567890189".into(),
                score: 1.0,
                recognizer_name: "t".into(),
                metadata: HashMap::new(),
                decision_trace: None,
            },
        ];
        let mut cfg = AnonymizationConfig::default();
        cfg.operators_by_entity.insert(
            "PERSON".into(),
            OperatorConfig {
                operator_type: OperatorType::Pseudonymize,
                params: HashMap::new(),
            },
        );
        cfg.operators_by_entity.insert(
            "EMAIL".into(),
            OperatorConfig {
                operator_type: OperatorType::Redact,
                params: [("use_empty".into(), "true".into())].into_iter().collect(),
            },
        );
        let mut fpe_p = HashMap::new();
        fpe_p.insert("key_hex".into(), KEY32.into());
        cfg.operators_by_entity.insert(
            "IBAN".into(),
            OperatorConfig {
                operator_type: OperatorType::Fpe,
                params: fpe_p,
            },
        );

        let out = engine.anonymize(text, &entities, &cfg);
        assert!(!out.text.contains("marie.synth"));
        assert!(out.text.contains("FR"));
        let back = AnonymizerEngine::deanonymize(&out, &hex::decode(KEY32).unwrap()).unwrap();
        assert_eq!(back, text);
    }

    #[test]
    fn encrypt_roundtrip_de() {
        let engine = AnonymizerEngine::new();
        let text = "Konto Hans Müller DE89370400440532013000";
        let iban = "DE89370400440532013000";
        let start = text.find(iban).expect("iban substring");
        let end = start + iban.len();
        let entities = vec![Entity {
            entity_type: EntityType::BankAccount,
            start,
            end,
            text: iban.into(),
            score: 1.0,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        }];
        let mut cfg = AnonymizationConfig::default();
        let mut p = HashMap::new();
        p.insert("key_hex".into(), KEY32.into());
        cfg.operators_by_entity.insert(
            "BANK_ACCOUNT".into(),
            OperatorConfig {
                operator_type: OperatorType::Encrypt,
                params: p,
            },
        );
        let out = engine.anonymize(text, &entities, &cfg);
        assert!(out.text.starts_with("Konto Hans Müller "));
        assert!(out.text.contains(ENC_PREFIX));
        let key = hex::decode(KEY32).unwrap();
        let back = AnonymizerEngine::deanonymize(&out, &key).unwrap();
        assert_eq!(back, text);
    }
}
