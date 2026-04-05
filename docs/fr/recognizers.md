# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Recognizers

Les recognizers implémentent le trait [`Recognizer`](https://github.com/zokastech/aegis/blob/main/crates/aegis-core/src/recognizer.rs) et produisent des **entités** notées avec positions.

## Pack regex par défaut (`default_regex_recognizers`)

Lorsque `recognizers.default_regex.enabled` est vrai, le moteur charge **15** détecteurs intégrés (avant filtrage par langue) :

| Nom | Type d’entité typique | Notes |
|-----|----------------------|-------|
| `email_rfc5322_like` | Email | Validation pragmatique RFC 5322 + listes de refus |
| `phone_e164_eu` | Téléphone | Motifs UE + style NANP |
| `credit_card_luhn` | Carte bancaire | Validation Luhn |
| `ipv4` | Adresse IP | |
| `ipv6` | Adresse IP | |
| `url` | URL | |
| `date_eu_iso` | Date | Orienté UE / ISO |
| `crypto_wallet` | Portefeuille crypto | Motifs d’adresse courants |
| `iban_iso13616` | IBAN | Mod-97 + contrôles de format |
| `bic_swift` | SWIFT / BIC | |
| `eu_vat_intracom` | Fiscal / TVA | Règles TVA multi-pays |
| `eu_credit_card_bins` | Carte bancaire | Motifs BIN orientés UE |
| `fr_siren` | ID national / org | France SIREN |
| `fr_siret` | ID national / org | France SIRET |
| `fr_nir` | ID national | France NIR |

Des balises de langue sont attachées par recognizer (ex. `en`, `fr`, `de`, …). La liste `recognizers.default_regex.languages` dans la config **filtre** ceux qui sont enregistrés.

Utiliser `GET /v1/recognizers` sur une passerelle en marche pour voir le **catalogue effectif** (drapeaux `disabled` inclus).

---

## Pack d’extension UE (`all_eu_recognizers`)

Des recognizers supplémentaires orientés UE (plates d’immatriculation, téléphones étendus, adresses, indices santé, marqueurs art. 9 RGPD, quasi-identifiants, **formats d’ID nationaux par pays**) vivent dans `aegis_regex::recognizers::eu::all_eu_recognizers`.

Ils ne font **pas** partie des 15 par défaut sauf intégration dans un **binaire Rust personnalisé** (ajout à `AnalyzerEngineBuilder` après chargement des défauts).

---

## Désactiver des recognizers

```yaml
recognizers:
  disabled:
    - iban_iso13616
```

Les noms sont comparés **sans tenir compte de la casse** au `name()` de chaque recognizer.

---

## Exemples

```text
alice@company.com          → email_rfc5322_like
+33 6 12 34 56 78          → phone_e164_eu
FR76 3000 6000 0112 3456 7890 189 → iban_iso13616
4532015112830367           → credit_card_luhn
```

---

## Organisation des sources

- Registre par défaut : `crates/aegis-regex/src/defaults.rs`
- Financier : `crates/aegis-regex/src/recognizers/financial/`
- Pack UE : `crates/aegis-regex/src/recognizers/eu/`
