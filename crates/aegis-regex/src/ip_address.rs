// AEGIS — zokastech.fr — Apache 2.0 / MIT

use crate::context_lexicon::{ip_negative_context, ip_positive_context};
use crate::pattern::PatternRecognizer;
use crate::static_regex::compile;
use aegis_core::entity::EntityType;
use std::net::Ipv6Addr;
use std::str::FromStr;
use std::sync::Arc;

/// IPv4 avec validation [`std::net::Ipv4Addr`].
pub fn ipv4_recognizer() -> PatternRecognizer {
    let re = compile(
        r"\b(?:(?:25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\.){3}(?:25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\b",
    );
    let pos: Vec<&str> = ip_positive_context();
    let neg: Vec<&str> = ip_negative_context();
    PatternRecognizer::new(
        "ipv4",
        re,
        EntityType::IpAddress,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl"],
        0.9,
    )
    .with_validator(Arc::new(|s| std::net::Ipv4Addr::from_str(s).is_ok()))
    .with_min_score(0.5)
    .with_context_boost_words(&pos, 0.05)
    .with_context_penalty_words(&neg, 0.1)
}

/// IPv6: full, compressed, IPv4-mapped, zone id; validated by parse.
pub fn ipv6_recognizer() -> PatternRecognizer {
    let re = compile(
        r"(?xi)
        \b(?:[0-9a-f]{1,4}:){7}[0-9a-f]{1,4}\b
        | \b(?:[0-9a-f]{1,4}:){1,6}:[0-9a-f:.]+(?:%[0-9A-Za-z_.\-]+)?\b
        | \b::1\b
        | \b::ffff:[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\b
        | \[[0-9a-f:.]+\](?::\d+)?
        ",
    );
    let pos: Vec<&str> = ip_positive_context();
    let neg: Vec<&str> = ip_negative_context();
    PatternRecognizer::new(
        "ipv6",
        re,
        EntityType::IpAddress,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl"],
        0.88,
    )
    .with_validator(Arc::new(|s| ipv6_ok(s)))
    .with_min_score(0.5)
    .with_context_boost_words(&pos, 0.05)
    .with_context_penalty_words(&neg, 0.1)
}

fn ipv6_ok(s: &str) -> bool {
    let mut t = s.trim();
    if t.starts_with('[') {
        let Some(end) = t.find(']') else {
            return false;
        };
        t = &t[1..end];
    }
    let core = t.split('%').next().unwrap_or(t).trim();
    if !core.contains(':') {
        return false;
    }
    Ipv6Addr::from_str(core).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Recognizer;
    use aegis_core::config::AnalysisConfig;

    fn cfg() -> AnalysisConfig {
        AnalysisConfig::default()
    }

    #[test]
    fn loopback_v4() {
        let r = ipv4_recognizer();
        let v = r.analyze("addr 127.0.0.1 ok", &cfg());
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn reject_overflow_octet() {
        let r = ipv4_recognizer();
        assert!(r.analyze("999.1.1.1", &cfg()).is_empty());
    }

    #[test]
    fn full_ipv6() {
        let r = ipv6_recognizer();
        let v = r.analyze("addr 2001:0db8:85a3:0000:0000:8a2e:0370:7334", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn loopback_compressed_ipv6() {
        let r = ipv6_recognizer();
        let v = r.analyze("ping ::1 end", &cfg());
        assert!(!v.is_empty());
    }
}
