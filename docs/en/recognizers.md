# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Recognizers

Recognizers implement the [`Recognizer`](https://github.com/zokastech/aegis/blob/main/crates/aegis-core/src/recognizer.rs) trait and produce scored **entities** with spans.

## Default regex pack (`default_regex_recognizers`)

When `recognizers.default_regex.enabled` is true, the engine loads **15** built-in detectors (before language filtering):

| Name | Typical entity type | Notes |
|------|---------------------|-------|
| `email_rfc5322_like` | Email | Pragmatic RFC 5322 validation + deny lists |
| `phone_e164_eu` | Phone | EU + NANP-style patterns |
| `credit_card_luhn` | Credit card | Luhn validation |
| `ipv4` | IP address | |
| `ipv6` | IP address | |
| `url` | URL | |
| `date_eu_iso` | Date | EU / ISO oriented |
| `crypto_wallet` | Crypto wallet | Common address patterns |
| `iban_iso13616` | IBAN | Mod-97 + format checks |
| `bic_swift` | SWIFT / BIC | |
| `eu_vat_intracom` | Tax / VAT ID | Multi-country VAT rules |
| `eu_credit_card_bins` | Credit card | BIN-oriented EU patterns |
| `fr_siren` | National / org ID | France SIREN |
| `fr_siret` | National / org ID | France SIRET |
| `fr_nir` | National ID | France NIR |

Language tags are attached per recognizer (e.g. `en`, `fr`, `de`, …). The config list `recognizers.default_regex.languages` **filters** which of these are registered.

Use `GET /v1/recognizers` on a running gateway to see the **effective** catalog (includes disabled flags).

---

## EU extension pack (`all_eu_recognizers`)

Additional EU-oriented recognizers (license plates, extended phones, addresses, health hints, GDPR art. 9 markers, quasi-identifiers, **national ID formats per country**) live in `aegis_regex::recognizers::eu::all_eu_recognizers`.

They are **not** part of the default 15 unless you integrate them in a **custom Rust binary** (add to `AnalyzerEngineBuilder` after loading defaults).

---

## Disabling recognizers

```yaml
recognizers:
  disabled:
    - iban_iso13616
```

Names are matched **case-insensitively** to each recognizer’s `name()`.

---

## Examples

```text
alice@company.com          → email_rfc5322_like
+33 6 12 34 56 78          → phone_e164_eu
FR76 3000 6000 0112 3456 7890 189 → iban_iso13616
4532015112830367           → credit_card_luhn
```

---

## Source layout

- Default registry: `crates/aegis-regex/src/defaults.rs`
- Financial: `crates/aegis-regex/src/recognizers/financial/`
- EU pack: `crates/aegis-regex/src/recognizers/eu/`
