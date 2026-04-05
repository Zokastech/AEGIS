// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Groups national-ID recognizers (EU) and filters by language.

use super::be::be_national_id_recognizer;
use super::de::de_national_id_recognizer;
use super::es::es_national_id_recognizer;
use super::fr::fr_national_id_recognizer;
use super::it::it_national_id_recognizer;
use super::nl::nl_national_id_recognizer;
use super::pl::pl_national_id_recognizer;
use super::pt::pt_national_id_recognizer;
use aegis_core::recognizer::Recognizer;
use std::collections::HashSet;
use std::sync::Arc;

/// All per-country national-ID recognizers (one composite per state).
///
/// - Empty `languages` → no filter (everything is returned).
/// - Otherwise, a recognizer is kept if one of its declared languages is in `languages` (case-insensitive) or equals `"*"`.
pub fn all_eu_national_id_recognizers(languages: &[&str]) -> Vec<Arc<dyn Recognizer>> {
    let all: Vec<Arc<dyn Recognizer>> = vec![
        Arc::new(fr_national_id_recognizer()),
        Arc::new(de_national_id_recognizer()),
        Arc::new(it_national_id_recognizer()),
        Arc::new(es_national_id_recognizer()),
        Arc::new(nl_national_id_recognizer()),
        Arc::new(be_national_id_recognizer()),
        Arc::new(pl_national_id_recognizer()),
        Arc::new(pt_national_id_recognizer()),
    ];
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
    fn empty_langs_returns_all_eight() {
        assert_eq!(all_eu_national_id_recognizers(&[]).len(), 8);
    }

    #[test]
    fn filters_unknown_lang() {
        assert!(all_eu_national_id_recognizers(&["xx"]).is_empty());
    }

    #[test]
    fn keeps_fr() {
        assert_eq!(all_eu_national_id_recognizers(&["fr"]).len(), 1);
    }

    #[test]
    fn keeps_de_and_it() {
        let v = all_eu_national_id_recognizers(&["de", "it"]);
        assert_eq!(v.len(), 2);
    }
}
