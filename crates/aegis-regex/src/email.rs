// AEGIS — zokastech.fr — Apache 2.0 / MIT

use crate::context_lexicon::{email_negative_context, email_positive_context};
use crate::pattern::PatternRecognizer;
use crate::static_regex::compile;
use crate::validation::email_rfc5322_pragmatic;
use aegis_core::entity::EntityType;
use std::sync::Arc;

/// Email detection with pragmatic validation close to RFC 5322.
pub fn email_recognizer() -> PatternRecognizer {
    let re = compile(
        r"(?i)\b[a-z0-9](?:[a-z0-9._%+\-]*[a-z0-9])?@[a-z0-9](?:[a-z0-9.\-]*[a-z0-9])?\.[a-z]{2,63}\b",
    );
    let pos: Vec<&str> = email_positive_context();
    let neg: Vec<&str> = email_negative_context();
    PatternRecognizer::new(
        "email_rfc5322_like",
        re,
        EntityType::Email,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl"],
        0.88,
    )
    .with_validator(Arc::new(email_rfc5322_pragmatic))
    .with_min_score(0.42)
    .with_deny_substrings(&[
        "example.com",
        "example.org",
        "example.net",
        "test.com",
        "invalid",
        "localhost",
    ])
    .with_context_boost_words(&pos, 0.07)
    .with_context_penalty_words(&neg, 0.11)
    .with_invalidate_words(&["noreply", "no-reply", "donotreply"])
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
    fn detects_simple_email() {
        let r = email_recognizer();
        let v = r.analyze("Contact: alice.smith@company.co.uk today", &cfg());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].text, "alice.smith@company.co.uk");
    }

    #[test]
    fn rejects_double_at() {
        let r = email_recognizer();
        assert!(r.analyze("bad@@x.com", &cfg()).is_empty());
    }

    #[test]
    fn rejects_leading_dot_local() {
        let r = email_recognizer();
        assert!(r.analyze(".a@b.co", &cfg()).is_empty());
    }

    #[test]
    fn rejects_example_domain() {
        let r = email_recognizer();
        assert!(r.analyze("x@example.com", &cfg()).is_empty());
    }

    #[test]
    fn plus_in_local_part() {
        let r = email_recognizer();
        let v = r.analyze("tag+list@domain.org", &cfg());
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn context_boost_multilingual() {
        let r = email_recognizer();
        let fr = r.analyze("mon courriel : user@mail.fr", &cfg());
        assert!(!fr.is_empty());
        let es = r.analyze("mi correo es a@b.es", &cfg());
        assert!(!es.is_empty());
        let de = r.analyze("E-Mail: k@d.de", &cfg());
        assert!(!de.is_empty());
    }
}
