# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Sécurité

La documentation sécurité d’AEGIS couvre le **durcissement du déploiement**, la **modélisation des menaces**, les **mesures techniques orientées RGPD** et les **artefacts de chaîne d’approvisionnement** (SBOM, signatures).

| Document | Description |
|----------|-------------|
| [Dépendances & licences tierces](../../../THIRD_PARTY_LICENSES.md) | Stacks (Rust, Go, npm, Python, Docker), Prometheus/Grafana, commandes d’inventaire |
| [Durcissement](hardening.md) | Checklist production, politiques réseau, rotation des secrets |
| [Modèle de menaces (STRIDE)](threat-model.md) | Flux de données, analyse STRIDE, matrice de risques (lisible DPO) |
| [Alignement RGPD](rgpd-compliance.md) | Mesures techniques reliées aux articles du RGPD (résumé non juridique) |

Signalement de vulnérabilités : voir [`SECURITY.md` sur GitHub](https://github.com/zokastech/aegis/blob/main/SECURITY.md).

Génération SBOM : `scripts/generate-sbom.sh` — décrit dans `SECURITY.md`.
