// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Default registry of regex recognizers.

use crate::credit_card::{credit_card_recognizer, masked_credit_card_recognizer};
use crate::crypto_wallet::crypto_wallet_recognizer;
use crate::date::date_recognizer;
use crate::email::email_recognizer;
use crate::ip_address::{ipv4_recognizer, ipv6_recognizer};
use crate::phone::phone_recognizer;
use crate::recognizers::eu::eu_address::eu_address_recognizer;
use crate::recognizers::financial::financial_recognizers;
use crate::url_recognizer::url_recognizer;
use aegis_core::recognizer::Recognizer;
use std::collections::HashSet;
use std::sync::Arc;

/// All **AEGIS** regex recognizers enabled for the requested languages.
///
/// - If `languages` is empty, no filter: everything is returned.
/// - A recognizer is kept if it declares `"*"` or a language present in `languages` (case-insensitive).
///
/// ```
/// use aegis_regex::default_regex_recognizers;
///
/// let v = default_regex_recognizers(&["fr", "de"]);
/// assert!(!v.is_empty());
/// ```
pub fn default_regex_recognizers(languages: &[&str]) -> Vec<Arc<dyn Recognizer>> {
    let mut all: Vec<Arc<dyn Recognizer>> = vec![
        Arc::new(email_recognizer()),
        Arc::new(phone_recognizer()),
        Arc::new(credit_card_recognizer()),
        Arc::new(masked_credit_card_recognizer()),
        Arc::new(ipv4_recognizer()),
        Arc::new(ipv6_recognizer()),
        Arc::new(url_recognizer()),
        Arc::new(date_recognizer()),
        Arc::new(crypto_wallet_recognizer()),
        Arc::new(eu_address_recognizer()),
    ];
    all.extend(financial_recognizers());
    filter_by_languages(all, languages)
}

fn filter_by_languages(
    all: Vec<Arc<dyn Recognizer>>,
    languages: &[&str],
) -> Vec<Arc<dyn Recognizer>> {
    if languages.is_empty() {
        return all;
    }
    let set: HashSet<String> = languages.iter().map(|s| s.to_lowercase()).collect();
    all.into_iter()
        .filter(|r| {
            r.supported_languages().iter().any(|l| {
                let l = l.to_lowercase();
                l == "*" || set.contains(&l)
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_langs_returns_all() {
        assert_eq!(default_regex_recognizers(&[]).len(), 17);
    }

    #[test]
    fn filters_unknown_lang() {
        let v = default_regex_recognizers(&["xx"]);
        assert!(v.is_empty());
    }

    #[test]
    fn keeps_fr() {
        let v = default_regex_recognizers(&["FR"]);
        assert_eq!(v.len(), 17);
    }
}
