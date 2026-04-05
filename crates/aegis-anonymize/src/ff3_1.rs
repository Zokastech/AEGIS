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
use std::cmp::Ordering;

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

fn pow_uv(radix: u32, u: usize, v: usize) -> (BigUint, BigUint) {
    let r = BigUint::from(radix as u64);
    match u.cmp(&v) {
        Ordering::Greater => {
            let pow_v = r.pow(v as u32);
            let pow_u = &pow_v * &r;
            (pow_u, pow_v)
        }
        _ => {
            let pow_u = r.pow(u as u32);
            let pow_v = if u == v { pow_u.clone() } else { &pow_u * &r };
            (pow_u, pow_v)
        }
    }
}

fn str2num_rev(src: &[u32], radix: u32) -> BigUint {
    let mut y = BigUint::zero();
    let r = BigUint::from(radix as u64);
    for i in (0..src.len()).rev() {
        y = y * &r + BigUint::from(src[i] as u64);
    }
    y
}

fn num2str_rev(x: &BigUint, dst: &mut [u32], radix: u32) {
    let r = BigUint::from(radix as u64);
    let mut xx = x.clone();
    for d in dst.iter_mut() {
        let (dv, rem) = xx.div_rem(&r);
        *d = rem.to_u32().unwrap_or(0);
        xx = dv;
    }
}

/// `BigUint` → octets big-endian (comme BN_bn2bin).
fn biguint_to_be_bytes_min(x: &BigUint, max_take: usize) -> Vec<u8> {
    let mut v = x.to_bytes_be();
    if v.len() > max_take {
        v = v[v.len() - max_take..].to_vec();
    }
    v
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
    let (pow_u, pow_v) = pow_uv(radix, u, v);
    let temp = (u as f64 * (radix as f64).log2()).ceil() as usize;
    let _b = (temp >> 3) + usize::from((temp & 7) > 0);

    let mut out = digits.to_vec();
    let mut a_off = 0usize;
    let mut b_off = u;

    for i in 0..ROUNDS {
        let m = if (i & 1) == 1 { v } else { u };
        let mut p = [0u8; 16];
        if (i & 1) == 1 {
            p[..4].copy_from_slice(&tweak8[..4]);
        } else {
            p[..4].copy_from_slice(&tweak8[4..8]);
        }
        p[3] ^= i as u8;

        let b_src = &out[b_off..b_off + (n - m)];
        let bnum = str2num_rev(b_src, radix);

        let raw = biguint_to_be_bytes_min(&bnum, 12);
        p[4..16].fill(0);
        let bl = raw.len().min(12);
        if bl > 0 {
            p[16 - bl..16].copy_from_slice(&raw[raw.len() - bl..]);
        }

        rev_bytes(&mut p);
        aes_encrypt_block(key, &mut p)?;
        rev_bytes(&mut p);

        let y = BigUint::from_bytes_be(&p);

        let a_src = &out[a_off..a_off + m];
        let anum = str2num_rev(a_src, radix);
        let qpow = if (i & 1) == 1 { &pow_v } else { &pow_u };
        let c = (anum + y) % qpow;

        let dst_b = &mut out[b_off..b_off + m];
        num2str_rev(&c, dst_b, radix);

        std::mem::swap(&mut a_off, &mut b_off);
    }

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
    let (pow_u, pow_v) = pow_uv(radix, u, v);
    let temp = (u as f64 * (radix as f64).log2()).ceil() as usize;
    let _b = (temp >> 3) + usize::from((temp & 7) > 0);

    let mut out = digits.to_vec();
    let mut a_off = 0usize;
    let mut b_off = u;

    for i in (0..ROUNDS).rev() {
        let m = if (i & 1) == 1 { v } else { u };
        let mut p = [0u8; 16];
        if (i & 1) == 1 {
            p[..4].copy_from_slice(&tweak8[..4]);
        } else {
            p[..4].copy_from_slice(&tweak8[4..8]);
        }
        p[3] ^= i as u8;

        let a_src = &out[a_off..a_off + (n - m)];
        let anum = str2num_rev(a_src, radix);

        let raw = biguint_to_be_bytes_min(&anum, 12);
        p[4..16].fill(0);
        let bl = raw.len().min(12);
        if bl > 0 {
            p[16 - bl..16].copy_from_slice(&raw[raw.len() - bl..]);
        }

        rev_bytes(&mut p);
        aes_encrypt_block(key, &mut p)?;
        rev_bytes(&mut p);

        let y = BigUint::from_bytes_be(&p);

        let b_src = &out[b_off..b_off + m];
        let bnum = str2num_rev(b_src, radix);
        let qpow = if (i & 1) == 1 { &pow_v } else { &pow_u };
        let y_mod = &y % qpow;
        let c = (bnum.clone() + qpow.clone() * BigUint::from(16u32) - y_mod) % qpow;

        let dst_a = &mut out[a_off..a_off + m];
        num2str_rev(&c, dst_a, radix);

        std::mem::swap(&mut a_off, &mut b_off);
    }

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
