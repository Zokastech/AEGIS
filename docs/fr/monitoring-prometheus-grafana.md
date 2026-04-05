# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Prometheus & Grafana (stack déjà en place)

Ce guide s’applique lorsque **Prometheus et Grafana existent déjà** chez vous (offre managée, plateforme partagée ou installation maison). Il suffit de **scraper** la passerelle AEGIS et de **brancher Grafana** sur votre Prometheus (ou d’importer le JSON du tableau de bord).

## Ce que la passerelle expose

| Point d’accès | Contenu | Remarques |
|----------------|---------|-----------|
| `GET /metrics` | Exposition texte Prometheus (métriques `aegis_*`) | **Texte brut** (gzip désactivé sur ce chemin pour un parsing fiable par les scrapers). |
| `GET /livez`, `GET /readyz` | Sondes HTTP | Optionnel ; pas scrapées par défaut. |

Voir aussi [Référence API](api-reference.md) et [Déploiement](deployment.md) (observabilité).

### Authentification sur `/metrics`

- **Développement** (`development.disable_auth` / compose dev) : scrape **sans** identifiants.
- **Production** (sécurité passerelle active) : le scraper doit disposer du droit **`metrics:view`** (RBAC). Prometheus doit envoyer le même type d’identifiants que l’API, par ex. **`X-API-Key`** ou **`Authorization: Bearer <JWT>`**.

Préférez un fichier secret monté dans Prometheus (ne jamais committer les jetons).

---

## 1. Configurer le scrape Prometheus

Ajoutez un job dans `scrape_configs` (noms et adresses sont des exemples — adaptez réseau / DNS).

### Passerelle en HTTP (mesh interne, `AEGIS_INSECURE_HTTP=1`)

```yaml
scrape_configs:
  - job_name: aegis-gateway
    scrape_interval: 15s
    metrics_path: /metrics
    scheme: http
    static_configs:
      - targets: ["aegis-gateway.votre-namespace.svc:8080"]
        labels:
          service: gateway
          environment: production
```

### Passerelle en HTTPS (style prod par défaut)

Exemple aligné sur le fichier dépôt [`docker/monitoring/prometheus.yml`](https://github.com/zokastech/aegis/blob/main/docker/monitoring/prometheus.yml) :

```yaml
scrape_configs:
  - job_name: aegis-gateway
    scrape_interval: 15s
    metrics_path: /metrics
    scheme: https
    tls_config:
      insecure_skip_verify: true   # idéalement remplacer par une CA maîtrisée
    static_configs:
      - targets: ["aegis-gateway.example.com:8443"]
        labels:
          service: gateway
```

### Scrape avec clé API (RBAC prod)

Prometheus 2.26+ autorise des en-têtes d’autorisation, par ex. jeton dans un fichier :

```yaml
  - job_name: aegis-gateway
    metrics_path: /metrics
    scheme: https
    authorization:
      type: Bearer
      credentials_file: /etc/prometheus/secrets/aegis-api-key.txt
    tls_config:
      ca_file: /etc/prometheus/tls/votre-ca.pem
    static_configs:
      - targets: ["aegis-gateway.interne:8080"]
```

Si la passerelle attend **`X-API-Key`** plutôt que Bearer, utilisez un **reverse proxy** devant `/metrics` qui ajoute l’en-tête, ou un petit sidecar nginx — Prometheus ne propose pas `X-API-Key` nativement sur le scrape.

!!! tip "Label `job` et Grafana"
    Le tableau Grafana fourni filtre sur `job="aegis-gateway"`. Gardez `job_name: aegis-gateway` **ou** adaptez les requêtes des panneaux à votre nom de job.

Rechargez Prometheus (`/-/reload` si l’API lifecycle est activée, ou redémarrage).

---

## 2. Vérifier la cible

1. UI Prometheus → **Status → Targets** : état **UP** pour `aegis-gateway`.
2. **Graph** → ex. `aegis_active_connections` ou `aegis_analyze_requests_total`.
3. Erreur de parse avec `\x1f` : réponse gzip — utilisez une version récente de la passerelle (gzip exclu pour `/metrics`) ou désactivez la compression sur ce chemin côté proxy.

---

## 3. Grafana (instance existante)

### Source de données

1. **Connections → Data sources → Add data source → Prometheus**.
2. **URL** : point d’accès Prometheus (ex. `http://prometheus:9090` en cluster, ou URL managée).
3. Enregistrer et **Test**.

### Tableau de bord

1. **Dashboards → New → Import**.
2. Importer le fichier **`docker/monitoring/grafana/dashboards/aegis-gateway-overview.json`** du dépôt AEGIS (ou coller le JSON).
3. Choisir la datasource Prometheus créée.

Les panneaux supposent `job="aegis-gateway"`. Si votre job diffère, ouvrez **Explore** avec par ex. `sum by (endpoint) (rate(aegis_analyze_requests_total[5m]))` puis ajustez les requêtes des panneaux.

---

## 4. Kubernetes (optionnel)

Avec **Prometheus Operator**, ajoutez un `ServiceMonitor` (ou `PodMonitor`) qui cible le Service de la passerelle, `path: /metrics`, `scheme` et TLS adaptés. Secrets de scrape : `basicAuth` / `bearerTokenSecret` selon la doc de l’opérateur.

---

## 5. Lien avec les compose du dépôt

| Fichier | Cas d’usage |
|---------|-------------|
| `docker-compose.dev.yml` | Dev local : Prometheus + Grafana + scrape HTTP (`docker/monitoring/prometheus.dev.yml`). |
| `docker-compose.yml` + `docker-compose.monitoring.yml` | Stack complète, scrape TLS d’exemple dans `docker/monitoring/prometheus.yml`. |

Cette page vise un **Prometheus / Grafana que vous possédez déjà** : les services du compose ne sont pas obligatoires.

---

## Voir aussi

- [THIRD_PARTY_LICENSES.md](https://github.com/zokastech/aegis/blob/main/THIRD_PARTY_LICENSES.md) (licences Prometheus / Grafana)
- [Sécurité — vue d’ensemble](security/index.md)
- [Durcissement](security/hardening.md)
