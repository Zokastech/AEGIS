// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! **Aho–Corasick** multi-pattern scanner: one linear pass over text (slice or stream).

use aho_corasick::{AhoCorasick, MatchKind};
use std::io::Read;

/// One literal-pattern occurrence from the scanner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralMatch {
    pub start: usize,
    pub end: usize,
    pub pattern_index: usize,
}

/// Single automaton for many patterns (aggregated deny-list, dictionaries, etc.).
///
/// ```
/// use aegis_regex::multi_pattern::MultiPatternScanner;
///
/// let scan = MultiPatternScanner::from_patterns(&["foo", "bar"]).unwrap();
/// let hits: Vec<_> = scan.find_iter(b"x foobar y").collect();
/// assert!(!hits.is_empty());
/// ```
pub struct MultiPatternScanner {
    ac: AhoCorasick,
}

impl MultiPatternScanner {
    /// Compile patterns (ASCII case-insensitive, overlap: leftmost-longest).
    pub fn from_patterns(patterns: &[&str]) -> Result<Self, aho_corasick::BuildError> {
        let pats: Vec<Vec<u8>> = patterns.iter().map(|p| p.as_bytes().to_vec()).collect();
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .ascii_case_insensitive(true)
            .build(pats)?;
        Ok(Self { ac })
    }

    /// Iterator over `haystack`: linear time in input size.
    pub fn find_iter<'a>(
        &'a self,
        haystack: &'a [u8],
    ) -> impl Iterator<Item = LiteralMatch> + 'a {
        self.ac.find_iter(haystack).map(|m| LiteralMatch {
            start: m.start(),
            end: m.end(),
            pattern_index: m.pattern().as_usize(),
        })
    }

    /// Streaming variant on [`Read`]: reads the whole stream then runs `find_iter`
    /// (handy for files; matches spanning two `read` calls are not merged).
    pub fn scan_reader<R: Read>(&self, mut r: R) -> std::io::Result<Vec<LiteralMatch>> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        Ok(self.find_iter(&buf).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leftmost_longest() {
        let s = MultiPatternScanner::from_patterns(&["abcd", "bc"]).unwrap();
        let m: Vec<_> = s.find_iter(b"xxabcdyy").collect();
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].start, m[0].end), (2, 6));
    }

    #[test]
    fn case_insensitive() {
        let s = MultiPatternScanner::from_patterns(&["NaN", "INF"]).unwrap();
        assert!(s.find_iter(b"value is nan").next().is_some());
    }
}
