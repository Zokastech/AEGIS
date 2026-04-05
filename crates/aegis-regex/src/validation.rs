// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Reusable validators (Luhn, pragmatic RFC 5322 email, etc.).

/// Luhn algorithm on a digit string (non-digits ignored).
pub fn luhn_valid(s: &str) -> bool {
    let digits: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() < 2 {
        return false;
    }
    let mut sum = 0u32;
    let mut alt = false;
    for d in digits.iter().rev() {
        let mut v = *d;
        if alt {
            v *= 2;
            if v > 9 {
                v -= 9;
            }
        }
        sum += v;
        alt = !alt;
    }
    sum % 10 == 0
}

/// Pragmatic check (lengths, consecutive dots, common ASCII character set).
pub fn email_rfc5322_pragmatic(s: &str) -> bool {
    if s.len() > 254 {
        return false;
    }
    let Some((local, domain)) = s.split_once('@') else {
        return false;
    };
    if local.is_empty() || local.len() > 64 || domain.is_empty() || domain.len() > 253 {
        return false;
    }
    if local.starts_with('.') || local.ends_with('.') || local.contains("..") {
        return false;
    }
    if domain.starts_with('.') || domain.ends_with('.') || domain.contains("..") {
        return false;
    }
    if !domain.contains('.') {
        return false;
    }
    let tld = domain.rsplit('.').next().unwrap_or("");
    if tld.len() < 2 || !tld.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    let local_ok = local
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-'));
    let domain_ok = domain
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'));
    local_ok && domain_ok
}

/// Keep digits only (for cards with spaces / dashes).
pub fn digits_only(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Typical lengths per network (after digit extraction).
pub fn credit_card_network_ok(digits: &str) -> bool {
    let len = digits.len();
    if !(13..=19).contains(&len) {
        return false;
    }
    let b = digits.as_bytes();
    if b.is_empty() {
        return false;
    }
    // Visa
    if b[0] == b'4' {
        return len == 13 || len == 16 || len == 19;
    }
    // MasterCard 2-series (2221–2720), 16 digits
    if b[0] == b'2' && len == 16 && b.len() >= 4 {
        let p4: u32 = (0..4).fold(0u32, |acc, i| acc * 10 + (b[i] - b'0') as u32);
        if (2221..=2720).contains(&p4) {
            return true;
        }
    }
    // MasterCard 51–55
    if b[0] == b'5' && len == 16 && b.len() >= 2 {
        let p2 = (b[0] - b'0') * 10 + (b[1] - b'0');
        if (51..=55).contains(&p2) {
            return true;
        }
    }
    // AmEx 34 / 37
    if b[0] == b'3' && len == 15 && b.len() >= 2 {
        return matches!(b[1], b'4' | b'7');
    }
    // Discover / other 6xxx
    if b[0] == b'6' && (len == 16 || len == 19) {
        return true;
    }
    false
}

pub fn validate_credit_card_match(s: &str) -> bool {
    let d = digits_only(s);
    credit_card_network_ok(&d) && luhn_valid(&d)
}

/// Bitcoin base58 alphabet (excludes 0, O, I, l).
pub fn is_btc_base58(s: &str) -> bool {
    s.chars()
        .all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c))
}

pub fn validate_btc_p2pkh_p2sh(s: &str) -> bool {
    let len = s.len();
    if !(26..=35).contains(&len) {
        return false;
    }
    matches!(s.as_bytes().first(), Some(b'1' | b'3')) && is_btc_base58(s)
}

pub fn validate_btc_bech32(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    if !lower.starts_with("bc1") {
        return false;
    }
    let skip = if lower.starts_with("bc1p") { 4 } else { 3 };
    let payload = lower.get(skip..).unwrap_or("");
    if payload.is_empty() || payload.len() > 87 {
        return false;
    }
    payload
        .chars()
        .all(|c| "qpzry9x8gf2tvdw0s3jn54khce6mua7l".contains(c))
}

pub fn validate_ethereum_address(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    let Some(hex) = lower.strip_prefix("0x") else {
        return false;
    };
    if hex.len() != 40 {
        return false;
    }
    hex.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luhn_known() {
        assert!(luhn_valid("79927398713"));
        assert!(!luhn_valid("79927398714"));
    }

    #[test]
    fn email_pragmatic() {
        assert!(email_rfc5322_pragmatic("a@b.co"));
        assert!(!email_rfc5322_pragmatic("a@@b.co"));
        assert!(!email_rfc5322_pragmatic(".a@b.co"));
    }
}
