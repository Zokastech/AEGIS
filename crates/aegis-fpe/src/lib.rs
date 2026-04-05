// AEGIS — zokastech.fr — Apache 2.0 / MIT
//! Format-preserving encryption (FF3-1, NIST SP 800-38G) for **AEGIS**.
//! See <https://zokastech.fr>.

#![forbid(unsafe_code)]

/// Placeholder: FF3-1 implementation will live here (cahier prompt 4.2).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_non_empty() {
        assert!(!VERSION.is_empty());
    }
}
