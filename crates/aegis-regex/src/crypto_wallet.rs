// AEGIS — zokastech.fr — Apache 2.0 / MIT

use crate::context_lexicon::{crypto_negative_context, crypto_positive_context};
use crate::pattern::PatternRecognizer;
use crate::static_regex::compile;
use crate::validation::{validate_btc_bech32, validate_btc_p2pkh_p2sh, validate_ethereum_address};
use aegis_core::entity::EntityType;
use std::sync::Arc;

fn validate_crypto_hit(s: &str) -> bool {
    let t = s.trim();
    if t.len() >= 26 && (t.starts_with('1') || t.starts_with('3')) {
        return validate_btc_p2pkh_p2sh(t);
    }
    if t.to_ascii_lowercase().starts_with("bc1") {
        return validate_btc_bech32(t);
    }
    validate_ethereum_address(t)
}

/// Bitcoin (P2PKH/P2SH, Bech32/SegWit, Taproot) et adresses Ethereum.
pub fn crypto_wallet_recognizer() -> PatternRecognizer {
    let re = compile(
        r"(?xi)
        \b[13][a-hj-np-zA-HJ-NP-Z1-9]{25,34}\b
        | \bbc1[a-z0-9]{11,71}\b
        | \b0x[a-f0-9]{40}\b
        ",
    );
    let pos: Vec<&str> = crypto_positive_context();
    let neg: Vec<&str> = crypto_negative_context();
    PatternRecognizer::new(
        "crypto_wallet",
        re,
        EntityType::CryptoWallet,
        vec!["en", "fr", "de", "es", "it", "nl", "pt", "pl"],
        0.68,
    )
    .with_validator(Arc::new(validate_crypto_hit))
    .with_min_score(0.42)
    .with_context_boost_words(&pos, 0.08)
    .with_context_penalty_words(&neg, 0.11)
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
    fn eth_address() {
        let r = crypto_wallet_recognizer();
        let v = r.analyze(
            "send to 0x742d35Cc6634C0532925a3b844Bc454e4438f44e please",
            &cfg(),
        );
        assert!(!v.is_empty());
    }

    #[test]
    fn btc_bech32_lowercase() {
        let r = crypto_wallet_recognizer();
        let v = r.analyze(
            "wallet bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq ok",
            &cfg(),
        );
        assert!(!v.is_empty());
    }

    #[test]
    fn invalid_eth_length() {
        let r = crypto_wallet_recognizer();
        assert!(r.analyze("0xabc", &cfg()).is_empty());
    }

    #[test]
    fn taproot_bech32m_shape() {
        let r = crypto_wallet_recognizer();
        let addr = "bc1p5cyxnuxmeuutcqydrlfm83hjt626j5tkredzwvm6xpn9vk0xtdmesg4eze";
        let v = r.analyze(addr, &cfg());
        assert!(!v.is_empty());
    }

    #[test]
    fn multilingual_context() {
        let r = crypto_wallet_recognizer();
        assert!(!r
            .analyze(
                "portafoglio 0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
                &cfg()
            )
            .is_empty());
    }
}
