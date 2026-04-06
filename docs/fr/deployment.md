# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Déploiement

Pour la **configuration et le lancement** du backend Zokastech (`zokastech/backend`), voir [Plateforme Zokastech](zokastech.md). Pour le **Terraform par cloud** (AWS, GCP, Azure, OVHcloud), voir [Fournisseurs cloud](cloud-providers.md).

## Docker Compose (développement / petite prod)

1. `cp .env.example .env` et définir les secrets.
2. `docker compose up -d --build`

Services (typiques) :

| Service | Rôle |
|---------|------|
| `aegis-core` | Moteur Rust + volume ONNX optionnel |
| `aegis-gateway` | API HTTP |
| `aegis-dashboard` | Interface web |
| `postgres` | Métadonnées / politiques / audit (selon intégration) |
| `redis` | Cache / files |
| `onnx-init` | Job optionnel : bundle **ZOKA-SENTINEL** (`.tgz` → `ner.onnx` + `tokenizer.json`) ou `NER_ONNX_URL` — voir `.env.example` |

Les réseaux **`frontend`** et **`backend`** isolent le dashboard des services internes — reproduire cela avec des groupes de sécurité en cloud.

Voir [`docker-compose.yml`](https://github.com/zokastech/aegis/blob/main/docker-compose.yml).

---

## Kubernetes

Chart officiel : **`deploy/helm/aegis/`** — installation avec Helm 3+.

```bash
helm upgrade --install aegis ./deploy/helm/aegis \
  --namespace aegis --create-namespace \
  -f your-values.yaml
```

Surcharger tags d’images, TLS ingress, limites de ressources et secrets via `values.yaml`.

---

## Terraform de référence (cloud)

Le détail des modules est sur **[Fournisseurs cloud (AWS, GCP, Azure, OVH)](cloud-providers.md)**. Résumé :

| Répertoire | Fournisseur |
|------------|-------------|
| `deploy/terraform/aws/` | AWS |
| `deploy/terraform/gcp/` | Google Cloud |
| `deploy/terraform/azure/` | Azure |
| `deploy/terraform/ovh/` | OVHcloud |

Ces modules illustrent le réseau type VPC, bases managées et contraintes régionales — **à revoir avant usage production**.

---

## Configuration et secrets

- Monter **`aegis-config.yaml`** en lecture seule.
- Stocker clés API et mots de passe DB dans **Kubernetes Secrets**, **GCP Secret Manager**, **AWS Secrets Manager**, ou équivalent.
- Activer TLS à l’ingress ; voir [Durcissement](security/hardening.md).

---

## Observabilité

- Prometheus : `GET /metrics` sur la passerelle
- Santé : `/v1/health`, `/livez`, `/readyz` (selon flags de build)

Si Prometheus et Grafana sont **déjà** déployés chez vous, voir **[Prometheus & Grafana (stack déjà en place)](monitoring-prometheus-grafana.md)** (scrape, RBAC sur `/metrics`, import du tableau Grafana AEGIS).

---

## Voir aussi

- [Démarrage](getting-started.md)
- [Sécurité](security/index.md)
