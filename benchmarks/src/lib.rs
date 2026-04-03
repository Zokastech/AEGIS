// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Corpus et tailles communes pour les benchmarks Criterion.

/// Bloc texte riche en signaux PII (répété pour atteindre la taille cible).
pub const BENCH_BLOCK: &str = "Contact: alice@company.co.uk tel +33 6 12 34 56 78 \
    card 4532015112830366 ip 192.168.0.1 url https://zokastech.fr/x \
    date 2024-06-01 IBAN FR7630006000011234567890189 BIC BNPAFRPPXXX \
    patient M. Dupont à Paris NIR 186022A123456 78 \n";

/// Tailles standard : 1 KiB … 10 MiB.
pub const SIZES: [usize; 5] = [1024, 10_240, 102_400, 1_048_576, 10_485_760];

/// Texte d’exactement `n` octets UTF-8 (tronqué proprement sur le bloc).
pub fn corpus_n_bytes(n: usize) -> String {
    let mut s = String::with_capacity(n);
    while s.len() < n {
        s.push_str(BENCH_BLOCK);
    }
    s.truncate(n);
    s
}
