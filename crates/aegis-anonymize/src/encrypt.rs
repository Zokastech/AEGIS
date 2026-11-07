// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! AES-256-GCM: random nonce, optional AAD, metadata for decryption.

use aegis_core::anonymizer::{Operator, OperatorConfig};
use aegis_core::entity::Entity;
use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};

use crate::types::ReverseMetadata;

/// Stable prefix in anonymized text (nonce || tag+ciphertext as hex).
pub const ENC_PREFIX: &str = "aegis_enc:v1:";

fn parse_key(config: &OperatorConfig) -> Result<[u8; 32], String> {
    let hex_key = config
        .params
        .get("key_hex")
        .map(|s| s.as_str())
        .ok_or_else(|| "missing key_hex (64 hex chars for AES-256)".to_string())?;
    let raw = hex::decode(hex_key.trim()).map_err(|e| e.to_string())?;
    if raw.len() != 32 {
        return Err(format!(
            "key_hex must decode to 32 bytes, got {}",
            raw.len()
        ));
    }
    let mut k = [0u8; 32];
    k.copy_from_slice(&raw);
    Ok(k)
}

pub struct EncryptOperator;

impl EncryptOperator {
    pub fn decrypt_blob(
        key: &[u8; 32],
        nonce: &[u8; 12],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>, String> {
        if key.len() != 32 {
            return Err("AES-256 key must be 32 bytes".into());
        }
        let key = Key::<Aes256Gcm>::from_slice(key.as_slice());
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce);
        cipher
            .decrypt(
                nonce,
                aes_gcm::aead::Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|_| "aes-gcm decrypt".to_string())
    }
}

impl Operator for EncryptOperator {
    fn operate(&self, entity: &Entity, _text: &str, config: &OperatorConfig) -> String {
        let key = match parse_key(config) {
            Ok(k) => k,
            Err(_) => return format!("{ENC_PREFIX}ERR_KEY"),
        };
        let key_ga = Key::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(key_ga);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let aad = config
            .params
            .get("aad")
            .map(|s| s.as_bytes())
            .unwrap_or_default();
        let payload = aes_gcm::aead::Payload {
            msg: entity.text.as_bytes(),
            aad,
        };
        let Ok(ct) = cipher.encrypt(&nonce, payload) else {
            return format!("{ENC_PREFIX}ERR_ENC");
        };
        let mut blob = Vec::with_capacity(12 + ct.len());
        blob.extend_from_slice(nonce.as_slice());
        blob.extend_from_slice(&ct);
        format!("{ENC_PREFIX}{}", hex::encode(&blob))
    }
}

/// Extrait nonce + ciphertext depuis une valeur produite par [`EncryptOperator`].
pub fn split_enc_blob(replacement: &str) -> Option<([u8; 12], Vec<u8>)> {
    let rest = replacement.strip_prefix(ENC_PREFIX)?;
    let raw = hex::decode(rest).ok()?;
    if raw.len() < 13 {
        return None;
    }
    let mut n = [0u8; 12];
    n.copy_from_slice(&raw[..12]);
    Some((n, raw[12..].to_vec()))
}

pub fn reverse_metadata_from_enc(
    replacement: &str,
    key_id: String,
    aad: String,
) -> Option<ReverseMetadata> {
    let (nonce, ciphertext) = split_enc_blob(replacement)?;
    Some(ReverseMetadata::AesGcmV1 {
        key_id,
        nonce,
        ciphertext,
        aad,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypt_metadata() {
        let nonce = [3u8; 12];
        let ct = b"fakecipher";
        let mut rep = ENC_PREFIX.to_string();
        rep.push_str(&hex::encode([nonce.as_slice(), ct.as_slice()].concat()));
        let rm = reverse_metadata_from_enc(&rep, "k1".into(), String::new()).unwrap();
        match rm {
            ReverseMetadata::AesGcmV1 {
                nonce: n,
                ciphertext,
                aad,
                ..
            } => {
                assert!(aad.is_empty());
                assert_eq!(n, nonce);
                assert_eq!(ciphertext, ct);
            }
            _ => panic!(),
        }
    }

    use aegis_core::entity::EntityType;
    use aegis_core::OperatorType;
    use std::collections::HashMap;

    const KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    fn entity(text: &str) -> Entity {
        Entity {
            entity_type: EntityType::Email,
            start: 0,
            end: text.len(),
            text: text.into(),
            score: 1.0,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        }
    }

    fn cfg(key: &str, aad: Option<&str>) -> OperatorConfig {
        let mut params = HashMap::new();
        params.insert("key_hex".to_string(), key.to_string());
        if let Some(a) = aad {
            params.insert("aad".to_string(), a.to_string());
        }
        OperatorConfig {
            operator_type: OperatorType::Encrypt,
            params,
        }
    }

    fn key() -> [u8; 32] {
        let mut k = [0u8; 32];
        k.copy_from_slice(&hex::decode(KEY_HEX).unwrap());
        k
    }

    #[test]
    fn encrypt_decrypt_roundtrip_with_aad() {
        let out = EncryptOperator.operate(
            &entity("jean@example.fr"),
            "",
            &cfg(KEY_HEX, Some("tenant-1")),
        );
        assert!(out.starts_with(ENC_PREFIX));
        assert!(!out.contains("jean"));
        let (nonce, ct) = split_enc_blob(&out).unwrap();
        let pt = EncryptOperator::decrypt_blob(&key(), &nonce, &ct, b"tenant-1").unwrap();
        assert_eq!(pt, b"jean@example.fr");
    }

    #[test]
    fn decrypt_fails_with_wrong_key_aad_or_tampering() {
        let out = EncryptOperator.operate(&entity("secret"), "", &cfg(KEY_HEX, Some("a")));
        let (nonce, mut ct) = split_enc_blob(&out).unwrap();
        let mut wrong = key();
        wrong[0] ^= 1;
        assert!(EncryptOperator::decrypt_blob(&wrong, &nonce, &ct, b"a").is_err());
        assert!(EncryptOperator::decrypt_blob(&key(), &nonce, &ct, b"b").is_err());
        ct[0] ^= 0xff;
        assert!(EncryptOperator::decrypt_blob(&key(), &nonce, &ct, b"a").is_err());
    }

    #[test]
    fn nonces_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..256 {
            let out = EncryptOperator.operate(&entity("same"), "", &cfg(KEY_HEX, None));
            let (nonce, _) = split_enc_blob(&out).unwrap();
            assert!(seen.insert(nonce), "nonce AES-GCM réutilisé");
        }
    }

    #[test]
    fn invalid_keys_never_leak_plaintext() {
        for bad in ["", "zz", "0011", &KEY_HEX[..62]] {
            let out = EncryptOperator.operate(&entity("plaintext-pii"), "", &cfg(bad, None));
            assert_eq!(out, format!("{ENC_PREFIX}ERR_KEY"));
        }
        let no_key = OperatorConfig {
            operator_type: OperatorType::Encrypt,
            params: HashMap::new(),
        };
        assert!(!EncryptOperator
            .operate(&entity("plaintext-pii"), "", &no_key)
            .contains("plaintext"));
    }

    #[test]
    fn malformed_blobs_rejected() {
        assert!(split_enc_blob("nope").is_none());
        assert!(split_enc_blob(&format!("{ENC_PREFIX}zz")).is_none());
        assert!(split_enc_blob(&format!("{ENC_PREFIX}{}", hex::encode([0u8; 12]))).is_none());
    }
}
