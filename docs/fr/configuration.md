# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Configuration (`aegis-config.yaml`)

Le moteur charge **`AegisEngineConfig`** depuis YAML (chemin souvent passé en `--config` ou monté `/etc/aegis/aegis.yaml` dans les conteneurs). Ce fichier contrôle les recognizers, la profondeur du pipeline, le NER, le score contextuel et les paramètres d’analyse par défaut.

!!! tip "Nom de fichier"
    L’exemple du dépôt s’appelle `aegis-config.yaml` ; dans les images Docker le même contenu peut être monté en `aegis.yaml`. Le schéma est identique.

---

## Clés de premier niveau (vue d’ensemble)

| Clé | Rôle |
|-----|------|
| `recognizers` | Activer/désactiver le pack regex par défaut et liste de désactivation par recognizer |
| `entity_thresholds` | Score minimal par type d’entité après le pipeline |
| `pipeline_level` | Raccourci : `1`, `2` ou `3` |
| `pipeline` | Réglage fin du pipeline multi-niveaux |
| `context_scorer` | Niveau 2 : bonus contextuels, combinaisons, quasi-identifiants |
| `ner` | Chemin modèle ONNX et options d’exécution |
| `analysis` | `AnalysisConfig` par défaut pour tous les passages |

---

## `recognizers`

### `recognizers.default_regex`

| Champ | Type | Défaut | Description |
|-------|------|--------|-------------|
| `enabled` | bool | `true` | Charger le pack regex intégré |
| `languages` | string[] | `["en","fr"]` | Filtrer les recognizers par langues déclarées (`*` = toutes) |

Les recognizers dont `supported_languages` n’intersecte pas la liste sont **omis**.

### `recognizers.disabled`

Liste des **noms** de recognizers (insensible à la casse) à ignorer après chargement, ex. :

```yaml
recognizers:
  default_regex:
    enabled: true
    languages: [en, fr, de, es, it, nl, pt, pl]
  disabled:
    - phone_e164_eu
```

---

## `entity_thresholds`

Carte **clé de type d’entité** → score **final** minimal (après fusion). Les clés suivent `EntityType::config_key` (ex. `EMAIL`, `CREDIT_CARD`, `PERSON`).

```yaml
entity_thresholds:
  EMAIL: 0.82
  CREDIT_CARD: 0.72
  PERSON: 0.55
```

---

## `pipeline_level`

Entier **`1`**, **`2`** ou **`3`** :

| Valeur | Effet |
|--------|--------|
| 1 | Regex / niveau 1 uniquement |
| 2 | Niveau 1 + scoreur contextuel |
| 3 | Niveaux 1 + 2 + NER ONNX (si modèle configuré) |

Fusionné dans l’énum détaillée `pipeline.levels` : `l1_only`, `l1_l2`, `l1_l2_l3`.

---

## `pipeline`

| Champ | Type | Défaut (moteur) | Description |
|-------|------|-----------------|-------------|
| `levels` | enum | `l1_l2_l3` | `l1_only` / `l1_l2` / `l1_l2_l3` |
| `output_score_threshold` | float | `0.5` | Seuil global sur scores fusionnés |
| `ner_invocation_score_threshold` | float | `0.75` | Quand envisager d’invoquer le NER |
| `short_circuit_l1_score` | float | `0.95` | Score L1 fort : sauter les niveaux plus profonds pour ce span |
| `weight_level1` | float | `0.45` | Poids de fusion L1 |
| `weight_level2` | float | `0.30` | Poids de fusion L2 |
| `weight_level3` | float | `0.25` | Poids de fusion L3 / NER |
| `timeout_level1_ms` | u64 | `2` | Budget temps L1 (0 = illimité) |
| `timeout_level2_ms` | u64 | `8` | Budget temps L2 |
| `timeout_level3_ms` | u64 | `60` | Budget temps NER |
| `adjacent_merge_gap_chars` | usize | `1` | Fusion des spans adjacents même type |
| `overlap_iou_min` | float | `0.35` | IoU min pour chevauchements = même entité |
| `record_decision_trace` | bool | `false` | Trace supplémentaire (éviter en prod si verbeux) |
| `analysis` | object | défauts | `AnalysisConfig` imbriqué pour passages internes du pipeline |

Les champs imbriqués `pipeline.analysis` reflètent le bloc racine `analysis` (`language`, `score_threshold`, `return_decision_process`, `context_window_size`).

---

## `context_scorer`

Pilote le **niveau 2**. Le schéma supporte un style **legacy** (`context_window_chars` + `languages`) et un jeu de règles **moderne** (`rules`, `tokens_before` / `tokens_after`, bloc `scorer`). Voir `crates/aegis-core/src/context/rules.rs` pour l’arbre complet.

Champs courants :

| Champ | Description |
|-------|-------------|
| `context_window_chars` | Fenêtre caractères legacy orientée PERSON |
| `languages` | Carte code locale → listes `person_boost`, `person_penalty`, `boost_delta`, `penalty_delta` |
| `combinations` | Règles qui augmentent le score quand plusieurs types d’entité apparaissent dans `within_chars` |
| `quasi_identifiers` | Combinaisons quasi-ID étendues, `base_risk_score`, `cap_risk_at` |
| `tokens_before` / `tokens_after` | Fenêtre en tokens avec règles modernes |
| `rules` | Règles contextuelles typées (avancé) |

Exemple (extrait d’échantillon dépôt) :

```yaml
context_scorer:
  context_window_chars: 96
  languages:
    fr:
      person_boost: [patient, client, M., Mme, monsieur]
      person_penalty: [ville de, pays, région]
      boost_delta: 0.08
      penalty_delta: 0.12
  combinations:
    - within_chars: 120
      require_entity_types: [DATE, LOCATION]
      alert_score_boost: 0.15
      note: quasi_id_date_location
```

---

## `ner`

| Champ | Type | Description |
|-------|------|-------------|
| `model_path` | string | Chemin fichier `.onnx` dans le conteneur / hôte |
| `device` | string | ex. `cpu` |
| `batch_size` | usize | Taille de batch d’inférence |
| `thread_pool_size` | usize (optionnel) | Pool de threads dédié ORT |

```yaml
ner:
  model_path: models/ner-mini.onnx
  device: cpu
  batch_size: 8
  thread_pool_size: 2
```

---

## `analysis` (racine)

Défaut [`AnalysisConfig`](https://github.com/zokastech/aegis/blob/main/crates/aegis-core/src/config.rs) :

| Champ | Type | Défaut | Description |
|-------|------|--------|-------------|
| `language` | string? | `None` | Indication BCP-47 (`fr`, `en`, …) |
| `score_threshold` | float | `0.5` | Score minimal pour émettre une entité dans ce passage (libellé Playground : *Seuil de confiance* — voir [Dashboard — Playground](dashboard-playground.md)) |
| `pipeline_level` | int? | `None` | Surcharge par requête : `1` = L1 seul, `2` = L1+L2, `3` = L1+L2+L3 (ex. `analysis_config_json` sur `POST /v1/analyze`) |
| `entities_to_analyze` | list? | `None` | Restreindre à certains types |
| `return_decision_process` | bool | `false` | Si `true`, trace des étapes pipeline et champ `decision_trace` sur chaque entité dans le JSON |
| `context_window_size` | usize | `5` | Fenêtre de contexte pour l’analyse |

---

## Opérateurs d’anonymisation (sujet séparé)

Les cartes d’opérateurs par type d’entité vivent dans **`AegisConfig`** (`operators_by_entity`) — souvent fournies via `config_json` API ou un YAML complémentaire de profils. Voir [Anonymisation](anonymization.md).

---

## Rechargement à chaud via API

Les administrateurs peuvent `PUT /v1/config` avec un **fragment** YAML fusionné dans le moteur en cours (passerelle → FFI). Prudence en production (audit + RBAC).

---

## Implémentation de référence

- Structs YAML : `crates/aegis-core/src/config/engine_yaml.rs`, `crates/aegis-core/src/context/rules.rs`, `crates/aegis-core/src/pipeline/config.rs`
- Fichier d’exemple : `aegis-config.yaml` à la racine du dépôt
