# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Anonymization operators

Operators transform detected spans into **replacement strings**. Configuration uses [`OperatorConfig`](https://github.com/zokastech/aegis/blob/main/crates/aegis-core/src/anonymizer.rs): `operator_type` + string `params` map.

## Operator types

| `operator_type` | Reversible | Typical use |
|-----------------|------------|-------------|
| `redact` | No | Remove or replace with fixed token |
| `replace` | No* | Placeholder like `<EMAIL_1>` |
| `mask` | No | Partial masking (keep last digits, etc.) |
| `hash` | No | One-way digest (salted) |
| `encrypt` | Yes | AES-GCM style blob + metadata for `deanonymize` |
| `fpe` | Yes | Format-preserving encryption (FF3-1) |
| `pseudonymize` | Via ledger | Stable random token per value / subject |

\*Replace is not cryptographically reversible unless you keep external mapping.

---

## `redact`

**Params**

| Key | Meaning |
|-----|---------|
| `replacement` | If set, output this string instead of empty redaction |

**Example**

```json
{
  "operator_type": "redact",
  "params": { "replacement": "[REDACTED]" }
}
```

---

## `replace`

Template per entity with incremental indices (`<TYPE_n>`).

**Params**

| Key | Meaning |
|-----|---------|
| `template` | e.g. `<EMAIL_{}>` — engine fills index |

Default behavior uses `<ENTITY_{}>` style labels when template omitted (see implementation).

---

## `mask`

**Params**

| Key | Meaning |
|-----|---------|
| `mask_char` | Character used for masking (default `*`) |
| `keep_last` | Number of characters to leave visible at end |
| `keep_first` | Number of characters to leave visible at start |

Useful for IBANs, phones, names.

---

## `hash`

**Params**

| Key | Meaning |
|-----|---------|
| `salt_hex` | Hex-encoded salt combined with value |
| `algorithm` | Engine-supported identifier (see `hash_op.rs`) |

Hashes are **not reversible** — suitable for pseudonymous analytics.

---

## `encrypt`

Reversible with the **same key material** supplied to `deanonymize`.

**Params**

| Key | Meaning |
|-----|---------|
| `key_hex` | 32-byte AES key as hex |
| `key_id` | Logical key id for multi-key rotation |

Treat keys as **highly sensitive** secrets.

---

## `fpe` (FF3-1)

Preserves length/format of numeric or alphanumeric strings.

**Params**

| Key | Meaning |
|-----|---------|
| `key_hex` | 32-byte key |
| `key_id` | Key identifier |
| `tweak` | Optional tweak string |

---

## `pseudonymize`

Emits stable opaque tokens; may require **ledger** storage for consistency across documents (gateway / policy integration).

**Params**

| Key | Meaning |
|-----|---------|
| `prefix` | Optional token prefix |
| `namespace` | Separate pseudonym spaces |

---

## API payload (`config_json`)

The REST field `config_json` serializes an **anonymization profile** compatible with `aegis_anonymize::AnonymizationConfig`: `operators_by_entity` keyed by entity config keys (`EMAIL`, `PHONE_NUMBER`, …) and optional `default_operator`.

Pseudo-structure (entity keys = `EntityType::config_key`, e.g. `EMAIL`, `PHONE`, `IBAN`):

```json
{
  "operators_by_entity": {
    "EMAIL": { "operator_type": "mask", "params": { "keep_last": "4", "mask_char": "*" } },
    "PHONE": { "operator_type": "redact", "params": {} }
  },
  "default_operator": { "operator_type": "replace", "params": {} }
}
```

Exact JSON field names follow Rust `serde` annotations on `AnonymizationConfig` in `crates/aegis-anonymize/src/types.rs`.

---

## Deanonymization

`POST /v1/deanonymize` accepts `anonymized_result_json` + `key_material_json` for operators that retain reversible metadata (`encrypt`, `fpe`). **Restrict to break-glass admin roles.**

---

## Code reference

- Engine: `crates/aegis-anonymize/src/engine.rs`
- Core types: `crates/aegis-core/src/anonymizer.rs`
