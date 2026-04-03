// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! PII entity types and analysis results.
//!
//! ```
//! use aegis_core::EntityType;
//!
//! let t = EntityType::Email;
//! assert!(matches!(t, EntityType::Email));
//! ```

use crate::error::{AegisError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// PII / sensitive entity types detectable by **AEGIS** (zokastech.fr).
///
/// [`EntityType::Custom`] extends the engine without changing this enum.
///
/// ```
/// use aegis_core::EntityType;
/// assert_eq!(EntityType::Iban.config_key(), "IBAN");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    Person,
    Email,
    Phone,
    CreditCard,
    Iban,
    Ssn,
    Passport,
    DriverLicense,
    IpAddress,
    Url,
    Date,
    Address,
    Organization,
    Location,
    MedicalRecord,
    NationalId,
    TaxId,
    BankAccount,
    CryptoWallet,
    VehiclePlate,
    Custom(String),
}

impl EntityType {
    /// Stable key for config files (YAML / TOML / JSON object).
    ///
    /// ```
    /// use aegis_core::EntityType;
    /// assert_eq!(EntityType::NationalId.config_key(), "NATIONAL_ID");
    /// ```
    pub fn config_key(&self) -> String {
        match self {
            EntityType::Person => "PERSON".into(),
            EntityType::Email => "EMAIL".into(),
            EntityType::Phone => "PHONE".into(),
            EntityType::CreditCard => "CREDIT_CARD".into(),
            EntityType::Iban => "IBAN".into(),
            EntityType::Ssn => "SSN".into(),
            EntityType::Passport => "PASSPORT".into(),
            EntityType::DriverLicense => "DRIVER_LICENSE".into(),
            EntityType::IpAddress => "IP_ADDRESS".into(),
            EntityType::Url => "URL".into(),
            EntityType::Date => "DATE".into(),
            EntityType::Address => "ADDRESS".into(),
            EntityType::Organization => "ORGANIZATION".into(),
            EntityType::Location => "LOCATION".into(),
            EntityType::MedicalRecord => "MEDICAL_RECORD".into(),
            EntityType::NationalId => "NATIONAL_ID".into(),
            EntityType::TaxId => "TAX_ID".into(),
            EntityType::BankAccount => "BANK_ACCOUNT".into(),
            EntityType::CryptoWallet => "CRYPTO_WALLET".into(),
            EntityType::VehiclePlate => "VEHICLE_PLATE".into(),
            EntityType::Custom(s) => format!("CUSTOM:{s}"),
        }
    }

    /// Parses a config key (inverse of [`EntityType::config_key`]).
    ///
    /// ```
    /// use aegis_core::EntityType;
    /// assert_eq!(EntityType::from_config_key("EMAIL").unwrap(), EntityType::Email);
    /// ```
    pub fn from_config_key(key: &str) -> Result<Self> {
        let key = key.trim();
        if let Some(rest) = key.strip_prefix("CUSTOM:") {
            if rest.is_empty() {
                return Err(AegisError::ConfigError(
                    "CUSTOM: nécessite un identifiant non vide".into(),
                ));
            }
            return Ok(EntityType::Custom(rest.to_string()));
        }
        match key.to_ascii_uppercase().as_str() {
            "PERSON" => Ok(EntityType::Person),
            "EMAIL" => Ok(EntityType::Email),
            "PHONE" => Ok(EntityType::Phone),
            "CREDIT_CARD" => Ok(EntityType::CreditCard),
            "IBAN" => Ok(EntityType::Iban),
            "SSN" => Ok(EntityType::Ssn),
            "PASSPORT" => Ok(EntityType::Passport),
            "DRIVER_LICENSE" => Ok(EntityType::DriverLicense),
            "IP_ADDRESS" => Ok(EntityType::IpAddress),
            "URL" => Ok(EntityType::Url),
            "DATE" => Ok(EntityType::Date),
            "ADDRESS" => Ok(EntityType::Address),
            "ORGANIZATION" => Ok(EntityType::Organization),
            "LOCATION" => Ok(EntityType::Location),
            "MEDICAL_RECORD" => Ok(EntityType::MedicalRecord),
            "NATIONAL_ID" => Ok(EntityType::NationalId),
            "TAX_ID" => Ok(EntityType::TaxId),
            "BANK_ACCOUNT" => Ok(EntityType::BankAccount),
            "CRYPTO_WALLET" => Ok(EntityType::CryptoWallet),
            "VEHICLE_PLATE" => Ok(EntityType::VehiclePlate),
            _ => Err(AegisError::ConfigError(format!(
                "type d'entité inconnu: {key}"
            ))),
        }
    }
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.config_key())
    }
}

impl FromStr for EntityType {
    type Err = AegisError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        EntityType::from_config_key(s)
    }
}

/// Per-entity decision trace (dashboard / Playground JSON, snake_case).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsonEntityDecisionTrace {
    pub steps: Vec<JsonTraceStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline_level: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTraceStep {
    pub name: String,
    pub passed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// One detection in the source text.
///
/// ```
/// use aegis_core::entity::{Entity, EntityType};
/// use std::collections::HashMap;
///
/// let e = Entity {
///     entity_type: EntityType::Email,
///     start: 0,
///     end: 5,
///     text: "a@b.c".into(),
///     score: 0.9,
///     recognizer_name: "email".into(),
///     metadata: HashMap::new(),
///     decision_trace: None,
/// };
/// assert!(e.validate_bounds(10));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: EntityType,
    pub start: usize,
    pub end: usize,
    pub text: String,
    /// Confidence score in \[0.0, 1.0\].
    pub score: f64,
    pub recognizer_name: String,
    pub metadata: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_trace: Option<JsonEntityDecisionTrace>,
}

/// Analysis result for one document.
///
/// ```
/// use aegis_core::entity::{AnalysisResult, Entity};
///
/// let r = AnalysisResult {
///     entities: vec![],
///     processing_time_ms: 1,
///     language_detected: Some("fr".into()),
///     text_length: 42,
/// };
/// assert_eq!(r.text_length, 42);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub entities: Vec<Entity>,
    pub processing_time_ms: u64,
    pub language_detected: Option<String>,
    pub text_length: usize,
}

impl Entity {
    /// Checks `start` ≤ `end`, spans fit in `text_len`, and score is in \[0,1\].
    pub fn validate_bounds(&self, text_len: usize) -> bool {
        self.start <= self.end && self.end <= text_len && (0.0..=1.0).contains(&self.score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_custom_roundtrip() {
        let t = EntityType::Custom("FOO".into());
        let j = serde_json::to_string(&t).unwrap();
        let back: EntityType = serde_json::from_str(&j).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn config_key_roundtrip() {
        for et in [
            EntityType::Person,
            EntityType::Iban,
            EntityType::NationalId,
            EntityType::Custom("x".into()),
        ] {
            let k = et.config_key();
            assert_eq!(EntityType::from_config_key(&k).unwrap(), et);
        }
    }
}
