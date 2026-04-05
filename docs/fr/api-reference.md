# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Référence API (REST)

L’URL de base dépend du déploiement. Les exemples utilisent `http://localhost:8080`. La passerelle expose **`/v1/*`** plus les routes d’observabilité.

!!! note "Authentification"
    Lorsque la **sécurité passerelle / RBAC** est activée, la plupart des routes exigent un en-tête **clé API** (nom par défaut configurable ; OpenAPI documente `X-API-Key`). Les routes admin (config, deanonymize, effacement sujet) exigent des clés **admin** ou des permissions dédiées.

## En-têtes courants

| En-tête | Quand |
|---------|--------|
| `Content-Type: application/json` | Tous les `POST`/`PUT` avec corps JSON |
| `X-API-Key: <secret>` | Passerelles sécurisées |
| `Authorization: Bearer <token>` | Si configuré pour votre environnement |

---

## `GET /v1/health`

Santé type liveness / readiness pour la passerelle et la liaison moteur.

```bash
curl -sS "http://localhost:8080/v1/health"
```

---

## `GET /v1/openapi.yaml`

Retourne la spécification **OpenAPI 3.0** (YAML).

```bash
curl -sS "http://localhost:8080/v1/openapi.yaml" -o openapi.yaml
```

---

## `GET /v1/recognizers`

Catalogue des recognizers (nom, type, activé).

```bash
curl -sS "http://localhost:8080/v1/recognizers"
```

---

## `GET /v1/entities`

Clés des **types d’entité** supportés (alignement config / politique).

```bash
curl -sS "http://localhost:8080/v1/entities"
```

---

## `POST /v1/analyze`

Analyser un texte unique.

**Champs du corps**

| Champ | Type | Description |
|-------|------|-------------|
| `text` | string | **Requis.** Texte d’entrée |
| `analysis_config_json` | string | JSON optionnel surchargeant la config d’analyse (schéma moteur) |
| `policy` | string | Nom optionnel de pack politique (ex. `gdpr-strict`) |

```bash
curl -sS -X POST "http://localhost:8080/v1/analyze" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Patient Jane Doe — nir 1 85 08 75 123 456 78",
    "policy": "gdpr-strict"
  }'
```

---

## `POST /v1/analyze/batch`

Analyse par lots paginée.

| Champ | Type | Description |
|-------|------|-------------|
| `texts` | string[] | Morceaux à analyser |
| `page` | int | Index de page (optionnel) |
| `page_size` | int | Taille de page (optionnel) |
| `policy` | string | Politique optionnelle |

```bash
curl -sS -X POST "http://localhost:8080/v1/analyze/batch" \
  -H "Content-Type: application/json" \
  -d '{"texts":["a@b.com","+33123456789"],"page":1,"page_size":10}'
```

---

## `POST /v1/anonymize`

Détection + anonymisation. `config_json` est la **config d’anonymisation** du moteur (opérateurs par entité).

| Champ | Type | Description |
|-------|------|-------------|
| `text` | string | Entrée |
| `config_json` | string | JSON : `operators_by_entity`, `default_operator`, … |
| `policy` | string | Politique optionnelle |
| `subject_id` | string | Id sujet optionnel pour registre pseudonyme / effacement |

```bash
curl -sS -X POST "http://localhost:8080/v1/anonymize" \
  -H "Content-Type: application/json" \
  -d '{"text":"Reach me at x@y.com","config_json":"{}"}'
```

---

## `POST /v1/feedback/false-positive`

Soumettre un retour faux positif (forme définie par l’implémentation passerelle).

```bash
curl -sS -X POST "http://localhost:8080/v1/feedback/false-positive" \
  -H "Content-Type: application/json" \
  -d '{}'
```

---

## `POST /v1/deanonymize` (admin)

Inverser les transformations réversibles lorsque la matière de clé est disponible. **Très sensible.**

```bash
curl -sS -X POST "http://localhost:8080/v1/deanonymize" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: ADMIN_KEY" \
  -d '{"anonymized_result_json":"{}","key_material_json":"{}"}'
```

---

## `PUT /v1/config` (admin)

Rechargement à chaud **partiel ou complet** du YAML moteur.

```bash
curl -sS -X PUT "http://localhost:8080/v1/config" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: ADMIN_KEY" \
  -d '{"yaml":"pipeline_level: 2\n"}'
```

---

## Routes politique (lorsque le service politique est branché)

### `GET /v1/policies`

```bash
curl -sS "http://localhost:8080/v1/policies"
```

### `GET /v1/policy/dpia`

Ébauche de rapport orienté DPIA (à compléter par votre revue juridique).

```bash
curl -sS "http://localhost:8080/v1/policy/dpia"
```

### `DELETE /v1/subjects/:id` (admin)

Crochet d’effacement pour un sujet (implémentation dépendante des backends de stockage).

```bash
curl -sS -X DELETE "http://localhost:8080/v1/subjects/sub-123" \
  -H "X-API-Key: ADMIN_KEY"
```

---

## `GET /v1/audit/export` (passerelle sécurisée)

Exporter les enregistrements d’audit (permission `PermAuditExport`).

```bash
curl -sS "http://localhost:8080/v1/audit/export" -H "X-API-Key: KEY"
```

---

## Observabilité

| Méthode | Chemin | Description |
|---------|--------|-------------|
| GET | `/metrics` | Métriques Prometheus |
| GET | `/livez` | Liveness (mode sécurisé) |
| GET | `/readyz` | Readiness (mode sécurisé) |
| GET | `/health/live`, `/health/ready`, `/health/startup` | Variantes health (si activées au build) |

```bash
curl -sS "http://localhost:8080/metrics" | head
```

---

## Erreurs

Les erreurs JSON incluent typiquement `code`, `message`, et `request_id` optionnel (`ErrorBody` dans le code passerelle).

---

## Source de vérité

Table des routes : `aegis-gateway/api/rest/server.go` — DTO : `aegis-gateway/api/rest/dto.go` — OpenAPI : `aegis-gateway/api/rest/openapi.yaml`.
