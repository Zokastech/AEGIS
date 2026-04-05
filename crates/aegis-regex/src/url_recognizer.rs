// AEGIS — zokastech.fr — Apache 2.0 / MIT

use crate::context_lexicon::{url_negative_context, url_positive_context};
use crate::pattern::PatternRecognizer;
use crate::static_regex::compile;
use aegis_core::entity::EntityType;
use std::sync::Arc;

/// Common TLDs: reduces false positives like "firstname.lastname" before `@` (email).
fn bare_host_two_labels_common_tld(host: &str) -> bool {
    let h = host.trim_end_matches('.').to_ascii_lowercase();
    let labels: Vec<&str> = h.split('.').filter(|p| !p.is_empty()).collect();
    if labels.len() != 2 {
        return true;
    }
    let tld = labels[1];
    const TLDS: &[&str] = &[
        "com", "net", "org", "fr", "de", "uk", "io", "co", "eu", "be", "ch", "es", "it", "nl",
        "pl", "pt", "at", "cz", "se", "no", "fi", "dk", "ie", "ro", "gr", "hu", "ru", "ua", "tr",
        "in", "au", "nz", "jp", "cn", "us", "ca", "br", "mx", "edu", "gov", "mil", "int", "info",
        "biz", "name", "pro", "app", "dev", "ai", "tv", "me", "cc", "ly", "so", "to", "fm", "is",
        "gg", "je", "im", "lt", "lu", "lv", "sk", "si", "hr", "bg", "ee", "mt", "cy", "gi", "fo",
        "gl", "ad", "mc", "sm", "va", "cat", "gal", "eus", "bar", "cloud", "tech", "online",
        "site", "store", "shop", "blog", "news", "email", "mail", "wtf", "xyz", "id", "sh", "vc",
        "ms", "ac", "ngo", "ong",
    ];
    TLDS.contains(&tld)
}

fn plausible_url(s: &str) -> bool {
    let t = s.trim();
    if t.len() < 4 {
        return false;
    }
    let lower = t.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return t.contains('.') || lower.contains("localhost");
    }
    if lower.starts_with("www.") {
        return t.contains('.');
    }
    if !t.contains('.') {
        return false;
    }
    let host = t.split('/').next().unwrap_or("");
    if !bare_host_two_labels_common_tld(host) {
        return false;
    }
    host.rsplit('.')
        .next()
        .map(|tld| tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()))
        .unwrap_or(false)
}

/// URLs with scheme (`http`/`https`) or `www.`, and common `domain.tld/path` shapes.
pub fn url_recognizer() -> PatternRecognizer {
    let re = compile(
        r#"(?xi)
        \bhttps?://[^\s<>\[\]{}|\\^`"']+
        | \bwww\.[^\s<>\[\]{}|\\^`"']+
        | \b[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?
          (?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)+
          (?:/[^\s<>\[\]{}|\\^`"']*)?
        "#,
    );
    let pos: Vec<&str> = url_positive_context();
    let neg: Vec<&str> = url_negative_context();
    PatternRecognizer::new(
        "url",
        re,
        EntityType::Url,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl"],
        0.7,
    )
    .with_validator(Arc::new(plausible_url))
    .with_min_score(0.38)
    .with_deny_substrings(&["example.com", "example.org", "test.com"])
    .with_context_boost_words(&pos, 0.07)
    .with_context_penalty_words(&neg, 0.12)
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
    fn https_url() {
        let r = url_recognizer();
        let v = r.analyze("see https://zokastech.fr/path?q=1", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn www_url() {
        let r = url_recognizer();
        let v = r.analyze("open www.example.net/x", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn bare_domain() {
        let r = url_recognizer();
        let v = r.analyze("visit docs.rust-lang.org/stable/", &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn rejects_example_com_deny() {
        let r = url_recognizer();
        assert!(r.analyze("https://example.com/x", &cfg()).is_empty());
    }

    #[test]
    fn rejects_two_label_person_name_not_url() {
        let r = url_recognizer();
        assert!(r
            .analyze("contact marie.durand (pas une URL)", &cfg())
            .is_empty());
    }

    #[test]
    fn two_label_known_tld_still_url() {
        let r = url_recognizer();
        assert!(!r.analyze("site zokastech.fr ok", &cfg()).is_empty());
    }

    #[test]
    fn multilingual_context() {
        let r = url_recognizer();
        assert!(!r.analyze("enlace https://a.b/c", &cfg()).is_empty());
        assert!(!r.analyze("lien web www.foo.bar", &cfg()).is_empty());
    }
}
