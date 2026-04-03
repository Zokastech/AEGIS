# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Architecture

## Composants (vue d’ensemble)

| Composant | Rôle |
|-----------|------|
| **aegis-gateway** (Go) | API HTTP, RBAC optionnel, réceptacle d’audit, hooks politiques |
| **aegis-core** (Rust) | Moteur d’analyse : registre + pipeline à 3 niveaux |
| **aegis-regex** (Rust) | Recognizers niveau 1 (regex, validateurs, lexiques) |
| **aegis-ner** (Rust) | Backend NER ONNX niveau 3 (optionnel) |
| **aegis-anonymize** (Rust) | Opérateurs d’anonymisation (redact, mask, FPE, …) |
| **aegis-policy** (Go) | Paquets de politiques YAML (orientés RGPD) |
| **aegis-dashboard** (React) | UI admin / playground (maturité variable) |
| **PostgreSQL / Redis** | Persistance et cache (déploiement type) |

## Pipeline de détection à trois niveaux

```mermaid
flowchart LR
  subgraph L1["Niveau 1 — Regex / règles"]
    R[Recognizers]
  end
  subgraph L2["Niveau 2 — Contexte"]
    C[Scoreur contexte\nquasi-identifiants]
  end
  subgraph L3["Niveau 3 — NER"]
    N[Modèle ONNX\noptionnel]
  end
  T[Texte] --> R
  R --> C
  C --> N
  N --> M[Fusion / scores / seuils]
  M --> O[Entités + scores]
```

- **L1** : détecteurs rapides par motifs (e-mail, téléphone, IBAN, …).
- **L2** : bonus / pénalités contextuels (ex. « patient », « M. ») et règles de combinaison.
- **L3** : NER ML lorsque configuré (`ner.model_path`) et invoqué selon seuils et timeouts du pipeline.

Les niveaux se choisissent avec `pipeline_level` et le bloc détaillé `pipeline` dans [`aegis-config.yaml`](configuration.md).

## Flux de données (chemin requête)

```mermaid
sequenceDiagram
  participant Client
  participant Gateway as aegis-gateway
  participant Core as aegis-core
  participant ONNX as NER ONNX
  Client->>Gateway: POST /v1/analyze
  Gateway->>Core: analyze (FFI / process)
  Core->>Core: L1 → L2 → L3 ?
  Core->>ONNX: inférence (si activé)
  ONNX-->>Core: spans
  Core-->>Gateway: AnalysisResult JSON
  Gateway-->>Client: 200 + JSON
```

Les données personnelles **transitent par la passerelle et le moteur** à chaque appel analyze/anonymize. **Ne pas journaliser les corps de requête bruts** en production sauf cadre DPA explicite.

## Documentation associée

- [Modèle de menaces](security/threat-model.md) — STRIDE et flux de données pour les DPO
- [Déploiement](deployment.md) — réseaux et secrets
