// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Compiles **static** [`regex::Regex`] patterns (known at compile time).
//!
//! A panic here indicates a development bug (invalid pattern), not bad user input.
//! [`track_caller`] improves panic location when a pattern is wrong.

use regex::Regex;

/// Compiles a static pattern. Panics if the pattern is syntactically invalid.
#[must_use]
#[track_caller]
pub(crate) fn compile(pattern: &str) -> Regex {
    Regex::new(pattern).expect("aegis-regex: motif statique invalide (bug interne)")
}
