# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Opérateurs d’anonymisation

Les opérateurs transforment les segments détectés en **chaînes de remplacement**. La configuration utilise [`OperatorConfig`](https://github.com/zokastech/aegis/blob/main/crates/aegis-core/src/anonymizer.rs) : `operator_type` + carte `params` en chaînes.

## Types d’opérateurs

| `operator_type` | Réversible | Usage typique |
|-----------------|------------|---------------|
| `redact` | Non | Suppression ou jeton fixe |
| `replace` | Non* | Placeholder type `<EMAIL_1>` |
| `mask` | Non | Masquage partiel (derniers chiffres visibles, etc.) |
| `hash` | Non | Empreinte unidirectionnelle (salée) |
| `encrypt` | Oui | Blob AES-GCM + métadonnées pour `deanonymize` |
| `fpe` | Oui | Chiffrement préservant le format (FF3-1) |
| `pseudonymize` | Via registre | Jeton opaque stable par valeur / sujet |

\*Replace n’est pas réversible cryptographiquement sans table de correspondance externe.

---

## `redact`

**Params**

| Clé | Signification |
|-----|---------------|
| `replacement` | Si défini, sortir cette chaîne au lieu d’une rédaction vide |

**Exemple**

```json
{
  "operator_type": "redact",
  "params": { "replacement": "[REDACTED]" }
}
```

---

## `replace`

Modèle par entité avec indices incrémentaux (`<TYPE_n>`).

**Params**

| Clé | Signification |
|-----|---------------|
| `template` | ex. `<EMAIL_{}>` — le moteur remplit l’index |

---

## `mask`

**Params**

| Clé | Signification |
|-----|---------------|
| `mask_char` | Caractère de masquage (défaut `*`) |
| `keep_last` | Nombre de caractères visibles en fin |
| `keep_first` | Nombre de caractères visibles en début |

Utile pour IBAN, téléphones, noms.

---

## `hash`

**Params**

| Clé | Signification |
|-----|---------------|
| `salt_hex` | Sel encodé hex combiné à la valeur |
| `algorithm` | Identifiant supporté par le moteur (voir `hash_op.rs`) |

Les hachages **ne sont pas réversibles** — adaptés à l’analytique pseudonyme.

---

## `encrypt`

Réversible avec la **même matière de clé** fournie à `deanonymize`.

**Params**

| Clé | Signification |
|-----|---------------|
| `key_hex` | Clé AES 32 octets en hex |
| `key_id` | Identifiant logique de clé pour rotation multi-clés |

Traiter les clés comme des secrets **très sensibles**.

---

## `fpe` (FF3-1)

Préserve longueur/format des chaînes numériques ou alphanumériques.

**Params**

| Clé | Signification |
|-----|---------------|
| `key_hex` | Clé 32 octets |
| `key_id` | Identifiant de clé |
| `tweak` | Chaîne de tweak optionnelle |

---

## `pseudonymize`

Émet des jetons opaques stables ; peut exiger un **registre** pour la cohérence entre documents (intégration passerelle / politique).

**Params**

| Clé | Signification |
|-----|---------------|
| `prefix` | Préfixe de jeton optionnel |
| `namespace` | Espaces de pseudonymes séparés |

---

## Charge utile API (`config_json`)

Le champ REST `config_json` sérialise un **profil d’anonymisation** compatible avec `aegis_anonymize::AnonymizationConfig` : `operators_by_entity` indexé par clés de config d’entité (`EMAIL`, `PHONE_NUMBER`, …) et `default_operator` optionnel.

Pseudo-structure (clés entité = `EntityType::config_key`, ex. `EMAIL`, `PHONE`, `IBAN`) :

```json
{
  "operators_by_entity": {
    "EMAIL": { "operator_type": "mask", "params": { "keep_last": "4", "mask_char": "*" } },
    "PHONE": { "operator_type": "redact", "params": {} }
  },
  "default_operator": { "operator_type": "replace", "params": {} }
}
```

Les noms de champs JSON exacts suivent les annotations `serde` sur `AnonymizationConfig` dans `crates/aegis-anonymize/src/types.rs`.

---

## Dé-anonymisation

`POST /v1/deanonymize` accepte `anonymized_result_json` + `key_material_json` pour les opérateurs avec métadonnées réversibles (`encrypt`, `fpe`). **Restreindre aux rôles admin de dernier recours.**

---

## Références code

- Moteur : `crates/aegis-anonymize/src/engine.rs`
- Types cœur : `crates/aegis-core/src/anonymizer.rs`
