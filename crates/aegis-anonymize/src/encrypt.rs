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
}
