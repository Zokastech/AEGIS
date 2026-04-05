// AEGIS — zokastech.fr — Apache 2.0 / MIT

use aegis_core::anonymizer::OperatorConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata to reverse a transformation (encryption / FPE).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReverseMetadata {
    /// AES-256-GCM: decrypt `ciphertext` with the key named `key_id`.
    AesGcmV1 {
        key_id: String,
        #[serde(with = "serde_hex12")]
        nonce: [u8; 12],
        #[serde(with = "serde_hex_vec")]
        ciphertext: Vec<u8>,
        /// Associated data (same bytes as at encryption).
        #[serde(default)]
        aad: String,
    },
    /// Decimal FPE: reversible with the same key + nonce (plaintext not stored).
    FpeDigitsV1 {
        key_id: String,
        #[serde(with = "serde_hex12")]
        nonce: [u8; 12],
    },
    /// Pseudonym / replacement: restore from ledger `original_text`.
    LedgerOnly,
}

mod serde_hex12 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 12], ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(de: D) -> Result<[u8; 12], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(de)?;
        let v = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if v.len() != 12 {
            return Err(serde::de::Error::custom("expected 12-byte nonce hex"));
        }
        let mut a = [0u8; 12];
        a.copy_from_slice(&v);
        Ok(a)
    }
}

mod serde_hex_vec {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ser.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(de)?;
        hex::decode(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationRecord {
    /// Offsets in **source** text.
    pub entity_start: usize,
    pub entity_end: usize,
    pub original_text: String,
    pub replacement: String,
    pub operator: String,
    pub entity_type: String,
    /// Offsets in **anonymized** text (filled in by the engine afterward).
    #[serde(default)]
    pub final_start: Option<usize>,
    #[serde(default)]
    pub final_end: Option<usize>,
    #[serde(default)]
    pub reverse: Option<ReverseMetadata>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnonymizationConfig {
    /// Operator per entity type (key = [`EntityType::config_key`], or `"*"`).
    #[serde(default)]
    pub operators_by_entity: HashMap<String, OperatorConfig>,
    /// When no per-type entry matches.
    #[serde(default)]
    pub default_operator: Option<OperatorConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedResult {
    pub text: String,
    pub transformations: Vec<TransformationRecord>,
    /// Key ids used for encryption (rotation).
    #[serde(default)]
    pub key_ids_used: Vec<String>,
    #[serde(default)]
    pub mapping_hints: HashMap<String, String>,
}
