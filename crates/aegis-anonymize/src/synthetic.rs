// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Realistic **synthetic PII** generation per country (statistical profiles).
//!
//! **Consistency**: the same `seed` (or [`subject_seed`] / [`seed_for_entity`]) yields a stable
//! *persona*: first/last name, email, phone, address, birth date, IBAN, and national ID stay
//! deterministic and mutually aligned when you call [`generate_synthetic`] with the same seed for
//! every entity of the same person.

use aegis_core::entity::{Entity, EntityType};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

/// Country profile: distributions, formats, and regional parameters (loaded from YAML).
#[derive(Debug, Clone, Deserialize)]
pub struct CountryProfile {
    pub country_code: String,
    #[serde(default)]
    pub locale: String,
    pub phone_country_prefix: String,
    pub phone_national_digits: u8,
    pub phone_mobile_leading_digits: Vec<String>,
    pub phone_display_format: String,
    pub phone_group_sizes: Vec<u8>,
    pub email_domain: String,
    pub date_format: String,
    pub address_template: String,
    pub iban_bban_numeric: bool,
    pub iban_bban_length: u8,
    pub national_id_kind: NationalIdKind,
    pub firstnames_male: Vec<WeightedName>,
    pub firstnames_female: Vec<WeightedName>,
    pub lastnames: Vec<WeightedName>,
    pub streets: Vec<String>,
    pub cities: Vec<CityPostal>,
    pub age_distribution: Vec<AgeBucket>,
    /// Display width for postal code (e.g. 5 for DE/FR/ES, 4 for NL).
    #[serde(default = "default_postal_width")]
    pub postal_width: u8,
}

fn default_postal_width() -> u8 {
    5
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeightedName {
    pub name: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CityPostal {
    pub name: String,
    pub postal_min: u32,
    pub postal_max: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgeBucket {
    pub min_age: u32,
    pub max_age: u32,
    pub weight: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NationalIdKind {
    FrNir,
    EsDni,
    DeRvnr,
    ItCf,
    NlBsn,
}

/// Synthetic data generator errors ([`non_exhaustive`] for forward-compatible extension).
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SyntheticError {
    #[error("profil inconnu: {0}")]
    UnknownProfile(String),
    #[error("profil YAML invalide: {0}")]
    InvalidProfile(String),
}

/// Generator: registry of embedded or loaded profiles.
#[derive(Debug, Clone)]
pub struct SyntheticDataGenerator {
    profiles: HashMap<String, CountryProfile>,
}

impl Default for SyntheticDataGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntheticDataGenerator {
    /// Embedded profiles for FR, DE, IT, ES, NL.
    pub fn new() -> Self {
        Self::from_embedded_profiles().expect("embedded profiles must be valid")
    }

    pub fn from_embedded_profiles() -> Result<Self, SyntheticError> {
        let mut profiles = HashMap::new();
        for (code, yaml) in [
            ("FR", include_str!("../profiles/fr.yaml")),
            ("DE", include_str!("../profiles/de.yaml")),
            ("IT", include_str!("../profiles/it.yaml")),
            ("ES", include_str!("../profiles/es.yaml")),
            ("NL", include_str!("../profiles/nl.yaml")),
        ] {
            let p: CountryProfile = serde_yaml::from_str(yaml)
                .map_err(|e| SyntheticError::InvalidProfile(format!("{code}: {e}")))?;
            profiles.insert(code.to_string(), p);
        }
        Ok(Self { profiles })
    }

    /// Loads YAML from disk and stores it under `country_code` (uppercase).
    pub fn load_profile_yaml(path: &str, replace_code: Option<&str>) -> Result<CountryProfile, SyntheticError> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| SyntheticError::InvalidProfile(e.to_string()))?;
        let mut p: CountryProfile = serde_yaml::from_str(&raw)
            .map_err(|e| SyntheticError::InvalidProfile(e.to_string()))?;
        if let Some(c) = replace_code {
            p.country_code = c.to_uppercase();
        }
        Ok(p)
    }

    pub fn insert_profile(&mut self, profile: CountryProfile) {
        self.profiles
            .insert(profile.country_code.to_uppercase(), profile);
    }

    pub fn profile(&self, country_code: &str) -> Option<&CountryProfile> {
        self.profiles.get(&country_code.to_uppercase())
    }

    /// Generates a synthetic value for `entity` using the profile and seed.
    pub fn generate(&self, entity: &Entity, country_code: &str, seed: u64) -> Result<String, SyntheticError> {
        let p = self
            .profile(country_code)
            .ok_or_else(|| SyntheticError::UnknownProfile(country_code.to_uppercase()))?;
        Ok(generate_synthetic(entity, p, seed))
    }
}

/// Derived seed so the same subject key (e.g. NER cluster id) stays stable within a document.
pub fn subject_seed(document_seed: u64, subject_key: &str) -> u64 {
    let mut h = blake3::Hasher::new();
    h.update(b"aegis-synthetic-subject-v1");
    h.update(&document_seed.to_le_bytes());
    h.update(subject_key.as_bytes());
    let x = h.finalize();
    u64::from_le_bytes(x.as_bytes()[..8].try_into().unwrap())
}

/// Uses `metadata["synthetic_subject_key"]` when set; otherwise entity text as the coherence key.
pub fn seed_for_entity(document_seed: u64, entity: &Entity) -> u64 {
    let key = entity
        .metadata
        .get("synthetic_subject_key")
        .map(|s| s.as_str())
        .unwrap_or(entity.text.as_str());
    subject_seed(document_seed, key)
}

fn sub_seed(seed: u64, tag: u32) -> u64 {
    let mut h = blake3::Hasher::new();
    h.update(b"aegis-synthetic-sub-v1");
    h.update(&seed.to_le_bytes());
    h.update(&tag.to_le_bytes());
    let x = h.finalize();
    u64::from_le_bytes(x.as_bytes()[..8].try_into().unwrap())
}

fn rng_from(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

fn weighted_pick<T: Clone>(items: &[T], weights: &[u32], rng: &mut StdRng) -> Option<T> {
    let total: u32 = weights.iter().sum();
    if total == 0 || items.is_empty() {
        return None;
    }
    let mut r = rng.gen_range(0..total);
    for (item, w) in items.iter().zip(weights.iter()) {
        if r < *w {
            return Some(item.clone());
        }
        r -= w;
    }
    items.first().cloned()
}

fn pick_firstname(profile: &CountryProfile, male: bool, rng: &mut StdRng) -> String {
    let (names, ws): (Vec<_>, Vec<_>) = if male {
        profile
            .firstnames_male
            .iter()
            .map(|w| (w.name.clone(), w.weight))
            .unzip()
    } else {
        profile
            .firstnames_female
            .iter()
            .map(|w| (w.name.clone(), w.weight))
            .unzip()
    };
    weighted_pick(&names, &ws, rng).unwrap_or_else(|| "Alex".into())
}

fn pick_lastname(profile: &CountryProfile, rng: &mut StdRng) -> String {
    let names: Vec<_> = profile.lastnames.iter().map(|w| w.name.clone()).collect();
    let ws: Vec<_> = profile.lastnames.iter().map(|w| w.weight).collect();
    weighted_pick(&names, &ws, rng).unwrap_or_else(|| "Dupont".into())
}

/// Deterministic first/last name parts for a seed (same seed → same names).
pub fn synthetic_person_parts(profile: &CountryProfile, seed: u64) -> (String, String, bool) {
    let mut rng = rng_from(sub_seed(seed, 1));
    let male = rng_from(sub_seed(seed, 0)).gen_bool(0.5);
    (pick_firstname(profile, male, &mut rng), pick_lastname(profile, &mut rng), male)
}

fn latin_email_slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => out.push('a'),
            'æ' => {
                out.push('a');
                out.push('e');
            }
            'ç' => out.push('c'),
            'è' | 'é' | 'ê' | 'ë' => out.push('e'),
            'ì' | 'í' | 'î' | 'ï' => out.push('i'),
            'ñ' => out.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => out.push('o'),
            'ù' | 'ú' | 'û' | 'ü' => out.push('u'),
            'ý' | 'ÿ' => out.push('y'),
            'ß' => out.push('s'),
            ' ' => {}
            c if c.is_ascii_alphanumeric() => out.push(c.to_ascii_lowercase()),
            _ => {}
        }
    }
    out
}

fn synthetic_email(profile: &CountryProfile, seed: u64) -> String {
    let (first, last, _) = synthetic_person_parts(profile, seed);
    let a = latin_email_slug(&first);
    let b = latin_email_slug(&last);
    let local = if b.is_empty() {
        a
    } else if a.is_empty() {
        b
    } else {
        format!("{a}.{b}")
    };
    format!("{local}@{}", profile.email_domain)
}

fn pick_city(profile: &CountryProfile, rng: &mut StdRng) -> (String, u32) {
    let idx = rng.gen_range(0..profile.cities.len().max(1));
    let c = profile.cities.get(idx).cloned().unwrap_or(CityPostal {
        name: "Ville".into(),
        postal_min: 1000,
        postal_max: 9999,
    });
    let p = rng.gen_range(c.postal_min..=c.postal_max);
    (c.name, p)
}

fn format_postal(n: u32, width: u8) -> String {
    format!("{:0width$}", n, width = width as usize)
}

fn synthetic_address(profile: &CountryProfile, seed: u64) -> String {
    let mut rng = rng_from(sub_seed(seed, 3));
    let street = profile
        .streets
        .get(rng.gen_range(0..profile.streets.len().max(1)))
        .cloned()
        .unwrap_or_else(|| "Rue Exemple".into());
    let num = rng.gen_range(1..=120u32);
    let (city, postal_n) = pick_city(profile, &mut rng);
    let postal = format_postal(postal_n, profile.postal_width);
    profile
        .address_template
        .replace("{num}", &num.to_string())
        .replace("{street}", &street)
        .replace("{postal}", &postal)
        .replace("{city}", &city)
}

fn synthetic_phone(profile: &CountryProfile, seed: u64) -> String {
    let mut rng = rng_from(sub_seed(seed, 2));
    let prefixes = &profile.phone_mobile_leading_digits;
    let lead = prefixes
        .get(rng.gen_range(0..prefixes.len().max(1)))
        .cloned()
        .unwrap_or_else(|| "6".into());
    let rem = profile.phone_national_digits.saturating_sub(lead.len() as u8) as usize;
    let mut national = lead;
    for _ in 0..rem {
        national.push(char::from_digit(rng.gen_range(0..10), 10).unwrap());
    }
    let sizes = &profile.phone_group_sizes;
    let sum: usize = sizes.iter().map(|&x| x as usize).sum();
    if sum != national.len() && !sizes.is_empty() {
        // Tolerance: even split if config does not match (profile evolution).
        let _ = sum;
    }
    let mut parts: Vec<String> = Vec::new();
    let mut off = 0;
    let sum_sz: usize = sizes.iter().map(|&x| x as usize).sum();
    if sum_sz == national.len() && !sizes.is_empty() {
        for &sz in sizes {
            let sz = sz as usize;
            parts.push(national[off..off + sz].to_string());
            off += sz;
        }
    } else {
        parts.push(national.clone());
    }
    let mut s = profile.phone_display_format.replace("{cc}", &profile.phone_country_prefix);
    for (i, ch) in ('a'..='z').enumerate() {
        let token = format!("{{{ch}}}");
        if s.contains(&token) {
            let rep = parts.get(i).cloned().unwrap_or_default();
            s = s.replace(&token, &rep);
        }
    }
    s
}

fn synthetic_birth_date(profile: &CountryProfile, seed: u64) -> String {
    let mut rng = rng_from(sub_seed(seed, 5));
    let buckets = &profile.age_distribution;
    let (min_age, max_age) = if buckets.is_empty() {
        (25u32, 55u32)
    } else {
        let ws: Vec<_> = buckets.iter().map(|b| b.weight).collect();
        let labels: Vec<usize> = (0..buckets.len()).collect();
        let idx = weighted_pick(&labels, &ws, &mut rng).unwrap_or(0);
        let b = &buckets[idx];
        (b.min_age, b.max_age)
    };
    let age = rng.gen_range(min_age..=max_age);
    let year = 2026i32 - age as i32;
    let month = rng.gen_range(1..=12u32);
    let day = rng.gen_range(1..=28u32);
    profile
        .date_format
        .replace("{yyyy}", &format!("{year:04}"))
        .replace("{mm}", &format!("{month:02}"))
        .replace("{dd}", &format!("{day:02}"))
}

fn expand_iban_numeric(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        if c.is_ascii_digit() {
            out.push(c);
        } else if c.is_ascii_uppercase() {
            let v = (c as u8 - b'A' + 10) as u32;
            out.push_str(&v.to_string());
        } else if c.is_ascii_lowercase() {
            let c = c.to_ascii_uppercase();
            let v = (c as u8 - b'A' + 10) as u32;
            out.push_str(&v.to_string());
        }
    }
    out
}

fn iban_check_digits(country: &[u8; 2], bban: &str) -> String {
    let mut rearranged = String::new();
    rearranged.push_str(bban);
    rearranged.push(country[0] as char);
    rearranged.push(country[1] as char);
    rearranged.push_str("00");
    let expanded = expand_iban_numeric(&rearranged);
    let mut rem = 0u32;
    for chunk in expanded.as_bytes().chunks(9) {
        let mut part = rem.to_string();
        part.push_str(std::str::from_utf8(chunk).unwrap());
        rem = (part.parse::<u64>().unwrap() % 97) as u32;
    }
    let cd = 98 - rem;
    format!("{cd:02}")
}

/// IBAN valide (mod 97 = 1) pour le pays du profil.
pub fn synthetic_iban(profile: &CountryProfile, seed: u64) -> String {
    let mut rng = rng_from(sub_seed(seed, 6));
    let cc = profile.country_code.to_uppercase();
    let ccb: [u8; 2] = [cc.as_bytes()[0], cc.as_bytes()[1]];
    let mut bban = String::new();
    if profile.iban_bban_numeric {
        for _ in 0..profile.iban_bban_length {
            bban.push(char::from_digit(rng.gen_range(0..10), 10).unwrap());
        }
    } else {
        for _ in 0..profile.iban_bban_length {
            bban.push(rng.gen_range(b'A'..=b'Z') as char);
        }
    }
    let cd = iban_check_digits(&ccb, &bban);
    format!("{}{}{}", cc, cd, bban)
}

fn mod97_from_digits(s: &str) -> u32 {
    let mut r = 0u32;
    for ch in s.chars() {
        if let Some(d) = ch.to_digit(10) {
            r = (r * 10 + d) % 97;
        }
    }
    r
}

fn synthetic_fr_nir(seed: u64) -> String {
    let mut rng = rng_from(sub_seed(seed, 7));
    let sex = rng.gen_range(1..=2);
    let yy = rng.gen_range(50..=99);
    let mm = rng.gen_range(1..=12);
    let dept = rng.gen_range(1..=95);
    let commune = rng.gen_range(1..=999);
    let order = rng.gen_range(1..=999);
    let base = format!(
        "{}{:02}{:02}{:02}{:03}{:03}",
        sex, yy, mm, dept, commune, order
    );
    let r = mod97_from_digits(&base);
    let key = 97 - r;
    let key = if key == 97 { 97 } else { key };
    format!("{}{:02}", base, key)
}

const DNI_TABLE: &str = "TRWAGMYFPDXBNJZSQVHLCKE";

fn synthetic_es_dni(seed: u64) -> String {
    let mut rng = rng_from(sub_seed(seed, 7));
    let n = rng.gen_range(10_000_000..=99_999_999u32);
    let letter = DNI_TABLE
        .chars()
        .nth((n % 23) as usize)
        .unwrap_or('A');
    format!("{n:08}{letter}")
}

fn synthetic_de_rvnr(seed: u64) -> String {
    let mut rng = rng_from(sub_seed(seed, 7));
    let mut d = [0u8; 11];
    for x in &mut d {
        *x = rng.gen_range(0..10);
    }
    let sum: u32 = d
        .iter()
        .enumerate()
        .map(|(i, &x)| x as u32 * ((i % 9) as u32 + 1))
        .sum();
    let c = (10 - (sum % 10)) % 10;
    let mut s = String::with_capacity(12);
    for x in d {
        s.push(char::from_digit(x as u32, 10).unwrap());
    }
    s.push(char::from_digit(c, 10).unwrap());
    s
}

const CF_MONTH: [char; 12] = ['A', 'B', 'C', 'D', 'E', 'H', 'L', 'M', 'P', 'R', 'S', 'T'];

fn cf_odd_value(c: char) -> u32 {
    match c {
        '0' | 'A' => 1,
        '1' | 'B' => 0,
        '2' | 'C' => 5,
        '3' | 'D' => 7,
        '4' | 'E' => 9,
        '5' | 'F' => 13,
        '6' | 'G' => 15,
        '7' | 'H' => 17,
        '8' | 'I' => 19,
        '9' | 'J' => 21,
        'K' => 2,
        'L' => 4,
        'M' => 18,
        'N' => 20,
        'O' => 11,
        'P' => 3,
        'Q' => 6,
        'R' => 8,
        'S' => 12,
        'T' => 14,
        'U' => 16,
        'V' => 10,
        'W' => 22,
        'X' => 25,
        'Y' => 24,
        'Z' => 23,
        _ => 0,
    }
}

fn cf_even_value(c: char) -> u32 {
    match c {
        '0'..='9' => c.to_digit(10).unwrap(),
        'A'..='Z' => c as u32 - 'A' as u32,
        _ => 0,
    }
}

fn synthetic_it_cf(seed: u64) -> String {
    let mut rng = rng_from(sub_seed(seed, 7));
    let mut body = String::with_capacity(15);
    for _ in 0..6 {
        body.push(rng.gen_range(b'A'..=b'Z') as char);
    }
    let yy = rng.gen_range(50..=99u32);
    body.push_str(&format!("{yy:02}"));
    let m = CF_MONTH[rng.gen_range(0..12)];
    body.push(m);
    let day = rng.gen_range(1..=28u32);
    body.push_str(&format!("{day:02}"));
    body.push(rng.gen_range(b'A'..=b'Z') as char);
    for _ in 0..3 {
        body.push(char::from_digit(rng.gen_range(0..10), 10).unwrap());
    }
    debug_assert_eq!(body.len(), 15);
    let mut sum = 0u32;
    for (i, c) in body.chars().enumerate() {
        let odd = (i % 2) == 0;
        sum += if odd {
            cf_odd_value(c)
        } else {
            cf_even_value(c)
        };
    }
    let check = (b'A' + (sum % 26) as u8) as char;
    format!("{body}{check}")
}

fn synthetic_nl_bsn(seed: u64) -> String {
    let mut rng = rng_from(sub_seed(seed, 7));
    for _ in 0..512 {
        let mut d = [0u32; 9];
        for i in 0..8 {
            d[i] = rng.gen_range(0..10);
        }
        let s: i32 = d
            .iter()
            .take(8)
            .enumerate()
            .map(|(i, &x)| x as i32 * (9 - i as i32))
            .sum();
        for d9 in 0..=9u32 {
            if (s + d9 as i32) % 11 == 0 {
                d[8] = d9;
                let mut out = String::with_capacity(9);
                for x in d {
                    out.push(char::from_digit(x, 10).unwrap());
                }
                return out;
            }
        }
    }
    "123456789".into()
}

fn synthetic_national_id(profile: &CountryProfile, seed: u64) -> String {
    match profile.national_id_kind {
        NationalIdKind::FrNir => synthetic_fr_nir(seed),
        NationalIdKind::EsDni => synthetic_es_dni(seed),
        NationalIdKind::DeRvnr => synthetic_de_rvnr(seed),
        NationalIdKind::ItCf => synthetic_it_cf(seed),
        NationalIdKind::NlBsn => synthetic_nl_bsn(seed),
    }
}

fn luhn_digit(body: &[u8]) -> u8 {
    let mut sum = 0u32;
    for (i, &d) in body.iter().enumerate() {
        let mut v = d as u32;
        if i % 2 == 1 {
            v *= 2;
            if v > 9 {
                v -= 9;
            }
        }
        sum += v;
    }
    ((10 - (sum % 10)) % 10) as u8
}

fn synthetic_card(seed: u64) -> String {
    let mut rng = rng_from(sub_seed(seed, 11));
    let mut body = [0u8; 15];
    body[0] = 4;
    for x in body.iter_mut().skip(1) {
        *x = rng.gen_range(0..10) as u8;
    }
    let c = luhn_digit(&body);
    let mut s = String::with_capacity(16);
    for x in body {
        s.push(char::from_digit(x as u32, 10).unwrap());
    }
    s.push(char::from_digit(c as u32, 10).unwrap());
    s
}

/// Replaces a PII entity with a **deterministic** synthetic value for the given `seed`.
///
/// Use the **same seed** for every entity belonging to one person to keep
/// coherence (name, email, phone, address, birth date, etc.).
pub fn generate_synthetic(entity: &Entity, profile: &CountryProfile, seed: u64) -> String {
    match entity.entity_type {
        EntityType::Person => {
            let (f, l, _) = synthetic_person_parts(profile, seed);
            format!("{f} {l}")
        }
        EntityType::Email => synthetic_email(profile, seed),
        EntityType::Phone => synthetic_phone(profile, seed),
        EntityType::Address => synthetic_address(profile, seed),
        EntityType::Location => {
            let mut rng = rng_from(sub_seed(seed, 3));
            let (city, _) = pick_city(profile, &mut rng);
            city
        }
        EntityType::Date => synthetic_birth_date(profile, seed),
        EntityType::Iban => synthetic_iban(profile, seed),
        EntityType::Ssn | EntityType::NationalId => synthetic_national_id(profile, seed),
        EntityType::TaxId => synthetic_national_id(profile, seed),
        EntityType::CreditCard => synthetic_card(seed),
        EntityType::Organization => {
            let mut rng = rng_from(sub_seed(seed, 8));
            let names = [
                "Acme Europe SAS",
                "BlueRiver GmbH",
                "Nordic Data SpA",
                "Iberia Labs SL",
                "Delta Services BV",
            ];
            names[rng.gen_range(0..names.len())].to_string()
        }
        EntityType::IpAddress => {
            let mut rng = rng_from(sub_seed(seed, 9));
            format!(
                "{}.{}.{}.{}",
                rng.gen_range(10..=192),
                rng.gen_range(0..255),
                rng.gen_range(0..255),
                rng.gen_range(1..254)
            )
        }
        EntityType::Url => {
            let dom = synthetic_email(profile, seed);
            if let Some(a) = dom.split('@').next() {
                format!("https://{a}.{}", profile.email_domain)
            } else {
                format!("https://www.{}", profile.email_domain)
            }
        }
        EntityType::Passport
        | EntityType::DriverLicense
        | EntityType::MedicalRecord
        | EntityType::BankAccount
        | EntityType::CryptoWallet
        | EntityType::VehiclePlate
        | EntityType::Custom(_) => {
            format!("[SYNTH:{}:{}]", entity.entity_type.config_key(), sub_seed(seed, 99))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_core::entity::EntityType;
    use std::collections::HashMap;

    fn fr_profile() -> CountryProfile {
        SyntheticDataGenerator::new().profile("FR").unwrap().clone()
    }

    fn entity(t: EntityType, text: &str) -> Entity {
        Entity {
            entity_type: t,
            start: 0,
            end: text.len(),
            text: text.into(),
            score: 1.0,
            recognizer_name: "t".into(),
            metadata: HashMap::new(),
            decision_trace: None,
        }
    }

    #[test]
    fn determinism_same_seed() {
        let p = fr_profile();
        let e = entity(EntityType::Person, "Jean Dupont");
        let a = generate_synthetic(&e, &p, 42);
        let b = generate_synthetic(&e, &p, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn person_email_coherent_same_seed() {
        let p = fr_profile();
        let seed = 999u64;
        let person = entity(EntityType::Person, "X");
        let mail = entity(EntityType::Email, "old@x.com");
        let name = generate_synthetic(&person, &p, seed);
        let email = generate_synthetic(&mail, &p, seed);
        let mut sp = name.split_whitespace();
        let f = sp.next().unwrap();
        let l = sp.next().unwrap();
        let local_expected = format!(
            "{}.{}@{}",
            latin_email_slug(f),
            latin_email_slug(l),
            p.email_domain
        );
        assert_eq!(email, local_expected, "{email} vs {name}");
    }

    #[test]
    fn iban_mod97() {
        let p = fr_profile();
        let iban = synthetic_iban(&p, 12345);
        assert!(iban.starts_with("FR"));
        let rearr = format!("{}{}", &iban[4..], &iban[..4]);
        let exp = expand_iban_numeric(&rearr);
        let mut rem = 0u32;
        for chunk in exp.as_bytes().chunks(9) {
            let mut part = rem.to_string();
            part.push_str(std::str::from_utf8(chunk).unwrap());
            rem = (part.parse::<u64>().unwrap() % 97) as u32;
        }
        assert_eq!(rem, 1, "IBAN {iban}");
    }

    #[test]
    fn seed_for_entity_uses_metadata() {
        let mut m = HashMap::new();
        m.insert("synthetic_subject_key".into(), "patient-A".into());
        let e = Entity {
            entity_type: EntityType::Phone,
            start: 0,
            end: 3,
            text: "different".into(),
            score: 1.0,
            recognizer_name: "t".into(),
            metadata: m,
            decision_trace: None,
        };
        let s1 = seed_for_entity(1, &e);
        let s2 = seed_for_entity(1, &e);
        assert_eq!(s1, s2);
    }

    #[test]
    fn nl_bsn_11_proof() {
        let p = SyntheticDataGenerator::new().profile("NL").unwrap();
        let s = synthetic_nl_bsn(777);
        assert_eq!(s.len(), 9);
        let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
        assert_eq!(d.len(), 9);
        let sum: i32 = d
            .iter()
            .enumerate()
            .map(|(i, &x)| x as i32 * (9 - i as i32))
            .sum();
        assert_eq!(sum % 11, 0);
    }

    #[test]
    fn es_dni_letter() {
        let s = synthetic_es_dni(42);
        assert_eq!(s.len(), 9);
        assert!(s[..8].chars().all(|c| c.is_ascii_digit()));
        assert!(s.as_bytes()[8].is_ascii_uppercase());
    }
}
