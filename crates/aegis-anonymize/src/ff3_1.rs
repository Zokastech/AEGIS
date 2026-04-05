// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! FF3 (NIST SP 800-38G) — Rust port of reference logic (SINHASantos/OpenSSL),
//! AES key 128/192/256, 8-byte tweak. Compatible with FF3 test vectors from the reference repo.
//!
//! FF3-1 (NIST revision) refines tweak handling; this module follows the FF3 construction
//! exercised by common ACVP tests. Context tweak: derive 8 bytes (e.g. truncated SHA-256).

use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::{Aes128, Aes192, Aes256};
use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::{ToPrimitive, Zero};

#[derive(Debug, Clone, thiserror::Error)]
pub enum Ff3Error {
    #[error("digit out of range for radix")]
    DigitRange,
    #[error("invalid key length (need 16, 24 or 32 bytes after hex decode)")]
    KeyLen,
    #[error("empty input")]
    Empty,
}

const ROUNDS: usize = 8;

fn rev_bytes(b: &mut [u8]) {
    b.reverse();
}

fn ceil2_ceil_half(n: usize) -> usize {
    (n >> 1) + (n & 1)
}

/// Same numeric weighting as Python `ff3.decode_int_r` (not plain big-endian digit parse).
fn decode_int_r_style(s: &[u32], radix: u32) -> BigUint {
    let strlen = s.len();
    let base = BigUint::from(radix as u64);
    let mut num = BigUint::zero();
    for (idx, &digit) in s.iter().rev().enumerate() {
        let power = strlen - (idx + 1);
        num += BigUint::from(digit as u64) * base.pow(power as u32);
    }
    num
}

/// Same as Python `ff3.encode_int_r`: LSB at `v[0]`, pad with digit 0 to `length`.
fn encode_int_r_style(n: &BigUint, length: usize, radix: u32) -> Vec<u32> {
    let base = BigUint::from(radix as u64);
    let mut x: Vec<u32> = Vec::new();
    let mut nn = n.clone();
    while nn >= base {
        let (dv, rem) = nn.div_rem(&base);
        x.push(rem.to_u32().unwrap_or(0));
        nn = dv;
    }
    x.push(nn.to_u32().unwrap_or(0));
    while x.len() < length {
        x.push(0);
    }
    x.truncate(length);
    x
}

fn biguint_to_fixed_12_be(val: &BigUint) -> [u8; 12] {
    let mut out = [0u8; 12];
    let raw = val.to_bytes_be();
    let n = raw.len().min(12);
    out[12 - n..].copy_from_slice(&raw[raw.len() - n..]);
    out
}

fn calculate_p_block(i: usize, w: &[u8; 4], b: &[u32], radix: u32) -> [u8; 16] {
    let mut p = [0u8; 16];
    p[0..4].copy_from_slice(w.as_slice());
    p[3] ^= i as u8;
    let val = decode_int_r_style(b, radix);
    let bbytes = biguint_to_fixed_12_be(&val);
    p[4..16].copy_from_slice(&bbytes);
    p
}

fn aes_encrypt_block(key: &[u8], block: &mut [u8; 16]) -> Result<(), Ff3Error> {
    let mut k = key.to_vec();
    rev_bytes(&mut k);
    match k.len() {
        16 => {
            let c = Aes128::new_from_slice(&k).map_err(|_| Ff3Error::KeyLen)?;
            let mut g = GenericArray::clone_from_slice(block);
            c.encrypt_block(&mut g);
            block.copy_from_slice(&g);
        }
        24 => {
            let c = Aes192::new_from_slice(&k).map_err(|_| Ff3Error::KeyLen)?;
            let mut g = GenericArray::clone_from_slice(block);
            c.encrypt_block(&mut g);
            block.copy_from_slice(&g);
        }
        32 => {
            let c = Aes256::new_from_slice(&k).map_err(|_| Ff3Error::KeyLen)?;
            let mut g = GenericArray::clone_from_slice(block);
            c.encrypt_block(&mut g);
            block.copy_from_slice(&g);
        }
        _ => return Err(Ff3Error::KeyLen),
    }
    Ok(())
}

/// Chiffre un tableau de chiffres `digits[i] ∈ [0, radix)` (longueur n ≥ 1).
pub fn ff3_encrypt(
    digits: &[u32],
    radix: u32,
    key: &[u8],
    tweak8: &[u8; 8],
) -> Result<Vec<u32>, Ff3Error> {
    if digits.is_empty() {
        return Err(Ff3Error::Empty);
    }
    for &d in digits {
        if d >= radix {
            return Err(Ff3Error::DigitRange);
        }
    }
    let n = digits.len();
    let u = ceil2_ceil_half(n);
    let v = n - u;
    let r = BigUint::from(radix as u64);
    let mod_u = r.pow(u as u32);
    let mod_v = r.pow(v as u32);
    let tl: [u8; 4] = tweak8[0..4].try_into().unwrap();
    let tr: [u8; 4] = tweak8[4..8].try_into().unwrap();

    let mut a = digits[0..u].to_vec();
    let mut b = digits[u..n].to_vec();

    for i in 0..ROUNDS {
        let (m, w, modulus) = if (i & 1) == 0 {
            (u, tr, &mod_u)
        } else {
            (v, tl, &mod_v)
        };

        let mut p = calculate_p_block(i, &w, &b, radix);
        rev_bytes(&mut p);
        aes_encrypt_block(key, &mut p)?;
        rev_bytes(&mut p);
        let y = BigUint::from_bytes_be(&p);

        let a_num = decode_int_r_style(&a, radix);
        let c = (a_num + y) % modulus;
        let c_digits = encode_int_r_style(&c, m, radix);

        a = b;
        b = c_digits;
    }

    let mut out = Vec::with_capacity(n);
    out.extend_from_slice(&a);
    out.extend_from_slice(&b);
    Ok(out)
}

/// Decrypt (inverse of [`ff3_encrypt`]).
pub fn ff3_decrypt(
    digits: &[u32],
    radix: u32,
    key: &[u8],
    tweak8: &[u8; 8],
) -> Result<Vec<u32>, Ff3Error> {
    if digits.is_empty() {
        return Err(Ff3Error::Empty);
    }
    for &d in digits {
        if d >= radix {
            return Err(Ff3Error::DigitRange);
        }
    }
    let n = digits.len();
    let u = ceil2_ceil_half(n);
    let v = n - u;
    let r = BigUint::from(radix as u64);
    let mod_u = r.pow(u as u32);
    let mod_v = r.pow(v as u32);
    let tl: [u8; 4] = tweak8[0..4].try_into().unwrap();
    let tr: [u8; 4] = tweak8[4..8].try_into().unwrap();

    let mut a = digits[0..u].to_vec();
    let mut b = digits[u..n].to_vec();

    for i in (0..ROUNDS).rev() {
        let (m, w, modulus) = if (i & 1) == 0 {
            (u, tr, &mod_u)
        } else {
            (v, tl, &mod_v)
        };

        let mut p = calculate_p_block(i, &w, &a, radix);
        rev_bytes(&mut p);
        aes_encrypt_block(key, &mut p)?;
        rev_bytes(&mut p);
        let y = BigUint::from_bytes_be(&p);

        let b_num = decode_int_r_style(&b, radix);
        let y_m = &y % modulus;
        let b_m = &b_num % modulus;
        let c = (b_m + modulus - y_m) % modulus;
        let c_digits = encode_int_r_style(&c, m, radix);

        b = a;
        a = c_digits;
    }

    let mut out = Vec::with_capacity(n);
    out.extend_from_slice(&a);
    out.extend_from_slice(&b);
    Ok(out)
}

/// Parse a string into radix digits (e.g. decimal).
pub fn parse_radix_string(s: &str, radix: u32) -> Result<Vec<u32>, Ff3Error> {
    s.chars()
        .map(|c| {
            let v = c.to_digit(radix)?;
            Some(v)
        })
        .collect::<Option<_>>()
        .ok_or(Ff3Error::DigitRange)
}

pub fn format_radix_string(digits: &[u32], radix: u32) -> Result<String, Ff3Error> {
    let mut s = String::with_capacity(digits.len());
    for &d in digits {
        if d >= radix {
            return Err(Ff3Error::DigitRange);
        }
        s.push(char::from_digit(d, radix).ok_or(Ff3Error::DigitRange)?);
    }
    Ok(s)
}

/// Derive an 8-byte tweak from arbitrary context (document, tenant, etc.).
pub fn tweak_from_context(context: &str) -> [u8; 8] {
    use sha2::{Digest, Sha256};
    let h = Sha256::digest(context.as_bytes());
    let mut t = [0u8; 8];
    t.copy_from_slice(&h[..8]);
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FF3 (AES-128) test vector from reference repo `Format-Preserving-Encryption-via-FF3-1` / test.py
    #[test]
    fn nist_style_ff3_vector_radix10() {
        let key = hex::decode("EF4359D8D580AA4F7F036D6F04FC6A94").unwrap();
        let tw = hex::decode("D8E7920AFA330A73").unwrap();
        let mut tweak8 = [0u8; 8];
        tweak8.copy_from_slice(&tw);
        let pt = parse_radix_string("890121234567890000", 10).unwrap();
        let ct = ff3_encrypt(&pt, 10, &key, &tweak8).unwrap();
        let ct_s = format_radix_string(&ct, 10).unwrap();
        assert_eq!(ct_s, "750918814058654607");
        let round = ff3_decrypt(&ct, 10, &key, &tweak8).unwrap();
        assert_eq!(round, pt);
    }

    #[test]
    fn round_trip_random_length_18() {
        let key = [7u8; 16];
        let tw = [1u8; 8];
        let pt = parse_radix_string("123456789012345678", 10).unwrap();
        let ct = ff3_encrypt(&pt, 10, &key, &tw).unwrap();
        let p2 = ff3_decrypt(&ct, 10, &key, &tw).unwrap();
        assert_eq!(pt, p2);
    }
}
