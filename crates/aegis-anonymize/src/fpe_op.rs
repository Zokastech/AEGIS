// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Reversible decimal FPE (IBAN, numbers): same length, letters unchanged, digits (de)encrypted.

use aegis_core::anonymizer::{Operator, OperatorConfig};
use aegis_core::entity::Entity;
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes256;
use aes_gcm::aead::OsRng;
use aes_gcm::aead::rand_core::RngCore;

pub const FPE_PREFIX: &str = "aegis_fpe:v1:";

fn parse_key_32(config: &OperatorConfig) -> Result<[u8; 32], ()> {
    let hex_key = config.params.get("key_hex").ok_or(())?;
    let raw = hex::decode(hex_key.trim()).map_err(|_| ())?;
    if raw.len() != 32 {
        return Err(());
    }
    let mut k = [0u8; 32];
    k.copy_from_slice(&raw);
    Ok(k)
}

fn derive_digit_round(key: &[u8; 32], nonce: &[u8; 12], digit_index: u32) -> u8 {
    let cipher = Aes256::new_from_slice(key).expect("32-byte key");
    let mut block = [0u8; 16];
    block[..12].copy_from_slice(nonce);
    block[12..].copy_from_slice(&digit_index.to_be_bytes());
    let mut g = aes::cipher::generic_array::GenericArray::clone_from_slice(&block);
    cipher.encrypt_block(&mut g);
    g[0] % 10
}

pub fn fpe_digits_transform(text: &str, key: &[u8; 32], nonce: &[u8; 12], decrypt: bool) -> String {
    let mut digit_index = 0u32;
    text.chars()
        .map(|c| {
            if c.is_ascii_digit() {
                let d = c.to_digit(10).unwrap() as i32;
                let k = derive_digit_round(key, nonce, digit_index) as i32;
                digit_index += 1;
                let v = if decrypt {
                    (d - k).rem_euclid(10)
                } else {
                    (d + k) % 10
                };
                char::from_digit(v as u32, 10).unwrap()
            } else {
                c
            }
        })
        .collect()
}

/// Parse `aegis_fpe:v1:<hex24>:<body>`.
pub fn split_fpe_token(s: &str) -> Option<([u8; 12], String)> {
    let rest = s.strip_prefix(FPE_PREFIX)?;
    let (nonce_hex, body) = rest.split_once(':')?;
    if nonce_hex.len() != 24 {
        return None;
    }
    let v = hex::decode(nonce_hex).ok()?;
    if v.len() != 12 {
        return None;
    }
    let mut n = [0u8; 12];
    n.copy_from_slice(&v);
    Some((n, body.to_string()))
}

pub struct FpeOperator;

impl Operator for FpeOperator {
    fn operate(&self, entity: &Entity, _text: &str, config: &OperatorConfig) -> String {
        let Ok(key) = parse_key_32(config) else {
            return format!("{FPE_PREFIX}ERR:");
        };
        let mut nonce = [0u8; 12];
        let mut rng = OsRng;
        rng.fill_bytes(&mut nonce);
        let body = fpe_digits_transform(&entity.text, &key, &nonce, false);
        format!("{}{}:{}", FPE_PREFIX, hex::encode(nonce), body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iban_roundtrip() {
        let key = [9u8; 32];
        let nonce = [1u8; 12];
        let iban = "FR7630006000011234567890189";
        let enc = fpe_digits_transform(iban, &key, &nonce, false);
        assert_eq!(enc.len(), iban.len());
        assert!(enc.starts_with("FR"));
        let dec = fpe_digits_transform(&enc, &key, &nonce, true);
        assert_eq!(dec, iban);
    }
}
