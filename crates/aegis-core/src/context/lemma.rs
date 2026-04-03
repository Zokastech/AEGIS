// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Light per-language lemmatization (common suffixes), no ML model.

/// Light lemmatizer to align text with keyword lists (e.g. "patients" → "patient").
#[derive(Debug, Clone, Default)]
pub struct LemmaAnalyzer;

impl LemmaAnalyzer {
    /// Normalize: lowercase NFC, strip edge punctuation, apply per-language suffix stripping.
    pub fn normalize_token(word: &str, lang: &str) -> String {
        let w = word.trim().trim_matches(|c: char| {
            matches!(
                c,
                '.' | ',' | ':' | ';' | '!' | '?' | '"' | '\'' | '(' | ')' | '«' | '»' | '„' | '“'
            )
        });
        if w.is_empty() {
            return String::new();
        }
        let lower = w.to_lowercase();
        Self::strip_suffixes(&lower, lang)
    }

    fn strip_suffixes(w: &str, lang: &str) -> String {
        let l = lang.to_ascii_lowercase();
        let suffixes: &[&str] = match l.as_str() {
            "fr" => &[
                "ements", "ement", "euses", "euse", "eaux", "eau", "ions", "iez", "ées", "ée",
                "ées", "èrent", "asses", "asse", "isses", "isse", "îmes", "îtes", "âtes", "âmes",
                "ants", "antes", "ant", "ent", "ons", "ez", "és", "ées", "ée", "é", "s", "x",
            ],
            "de" => &[
                "ungen", "ungen", "ungen", "lichkeiten", "lichkeit", "heiten", "heit", "ungen",
                "ungen", "innen", "in", "chen", "ern", "er", "en", "em", "es", "e", "s",
            ],
            "es" | "it" | "pt" => &[
                "aciones", "ación", "aciones", "mente", "adoras", "adores", "adora", "ador",
                "mente", "issimo", "issima", "issimi", "issime", "zioni", "zione", "amenti",
                "amento", "ei", "ai", "ano", "ate", "ava", "ivi", "ere", "ire", "oso", "osa",
                "os", "as", "es", "s",
            ],
            "nl" => &[
                "heden", "heid", "tjes", "tje", "ingen", "ing", "eren", "er", "en", "e", "s",
            ],
            "en" => &[
                "fulness", "ness", "ments", "ment", "ingly", "edly", "ions", "ion", "ies", "ied",
                "ing", "ed", "es", "s",
            ],
            _ => &["s", "x", "en", "e"],
        };

        let mut cur = w.to_string();
        for _ in 0..3 {
            let mut shortened = false;
            for suf in suffixes {
                if suf.len() < cur.len() && cur.ends_with(suf) {
                    let next_len = cur.len() - suf.len();
                    if next_len >= 2 {
                        cur.truncate(next_len);
                        shortened = true;
                        break;
                    }
                }
            }
            if !shortened {
                break;
            }
        }
        cur
    }

    /// True if the token’s lemma matches the expected lemma (both normalized).
    pub fn lemma_matches(token: &str, expected: &str, lang: &str) -> bool {
        let a = Self::normalize_token(token, lang);
        let b = Self::normalize_token(expected, lang);
        !a.is_empty() && a == b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patients_to_patient_fr() {
        assert!(LemmaAnalyzer::lemma_matches("patients", "patient", "fr"));
        assert!(LemmaAnalyzer::lemma_matches("employés", "employé", "fr"));
    }
}
