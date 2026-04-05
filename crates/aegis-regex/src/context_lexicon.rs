// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Mots de contexte multilingues (EN, FR, DE, ES, IT) pour ajuster les scores.

/// Indices « positifs » autour d’une adresse e-mail.
pub fn email_positive_context() -> Vec<&'static str> {
    vec![
        // EN
        "email",
        "e-mail",
        "mail",
        "mailto",
        "contact",
        "reach",
        "reply",
        // FR
        "courriel",
        "mel",
        "adresse",
        "contacter",
        "écrire",
        "ecrire",
        // DE
        "e-mail",
        "email",
        "kontakt",
        "ansprechpartner",
        "schreiben",
        // ES
        "correo",
        "email",
        "contacto",
        "escribir",
        // IT
        "email",
        "posta",
        "contatto",
        "scrivere",
    ]
}

/// Negative cues (false positives / fictional data) — 5 languages.
pub fn email_negative_context() -> Vec<&'static str> {
    vec![
        "example", "sample", "test", "fake", "dummy", "invalid", "exemple", "faux", "beispiel",
        "muster", "ejemplo", "prueba", "esempio", "fittizio",
    ]
}

pub fn phone_positive_context() -> Vec<&'static str> {
    vec![
        "phone",
        "tel",
        "telephone",
        "mobile",
        "cell",
        "call",
        "téléphone",
        "telephone",
        "tél",
        "portable",
        "gsm",
        "telefono",
        "teléfono",
        "móvil",
        "movil",
        "handy",
        "rufnummer",
        "numero",
        "numéro",
        "numero di telefono",
    ]
}

pub fn phone_negative_context() -> Vec<&'static str> {
    vec![
        "example", "test", "sample", "fake", "exemple", "beispiel", "ejemplo", "esempio", "iban",
        "credit", "card",
    ]
}

pub fn card_positive_context() -> Vec<&'static str> {
    vec![
        "card",
        "credit",
        "debit",
        "visa",
        "mastercard",
        "amex",
        "payment",
        "cb",
        "carte",
        "bancaire",
        "karte",
        "zahlung",
        "tarjeta",
        "pago",
        "carta",
        "pagamento",
    ]
}

pub fn card_negative_context() -> Vec<&'static str> {
    vec![
        "example", "test", "sample", "invalid", "fake", "exemple", "beispiel", "ejemplo",
        "esempio", "phone", "iban",
    ]
}

pub fn ip_positive_context() -> Vec<&'static str> {
    vec![
        "ip",
        "ipv4",
        "ipv6",
        "address",
        "host",
        "server",
        "gateway",
        "dns",
        "adresse",
        "serveur",
        "rechner",
        "direccion",
        "servidor",
        "indirizzo",
    ]
}

pub fn ip_negative_context() -> Vec<&'static str> {
    vec!["version", "example", "test", "fake", "exemple", "beispiel"]
}

pub fn url_positive_context() -> Vec<&'static str> {
    vec![
        "url",
        "link",
        "website",
        "http",
        "https",
        "www",
        "site",
        "lien",
        "web",
        "enlace",
        "sitio",
        "collegamento",
        "sito",
    ]
}

pub fn url_negative_context() -> Vec<&'static str> {
    vec![
        "example.com",
        "example.org",
        "test.com",
        "localhost",
        "invalid",
        "fake",
    ]
}

pub fn date_positive_context() -> Vec<&'static str> {
    vec![
        "date",
        "born",
        "birth",
        "due",
        "expires",
        "datum",
        "geburt",
        "fecha",
        "nacimiento",
        "data",
        "nascita",
        "naissance",
    ]
}

pub fn date_negative_context() -> Vec<&'static str> {
    vec!["version", "v.", "release", "build"]
}

pub fn crypto_positive_context() -> Vec<&'static str> {
    vec![
        "bitcoin",
        "btc",
        "ethereum",
        "eth",
        "wallet",
        "address",
        "send",
        "payment",
        "crypto",
        "portefeuille",
        "monedero",
        "billetera",
        "portafoglio",
        "zahlung",
    ]
}

pub fn crypto_negative_context() -> Vec<&'static str> {
    vec!["example", "test", "fake", "invalid", "exemple", "beispiel"]
}
