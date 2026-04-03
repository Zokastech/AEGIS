# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Guide de durcissement et déploiement sécurisé

Ce document complète la politique [`SECURITY.md` (dépôt)](https://github.com/zokastech/aegis/blob/main/SECURITY.md) avec des recommandations opérationnelles pour **production**.

## Principes

- **Minimal privilege** : comptes de service, rôles IAM, et droits réseau au plus juste.
- **Défense en profondeur** : WAF / reverse proxy, TLS, segmentation, journaux centralisés.
- **Reproductibilité** : images signées, SBOM archivés, tags de release traçables.

## Checklist — configuration production

### Moteur AEGIS (Rust / conteneur)

- [ ] Fichier `aegis-config.yaml` versionné (sans secrets en clair) ; secrets injectés par **secrets manager** (GCP Secret Manager, AWS Secrets Manager, Vault, etc.).
- [ ] `recognizers.disabled` et `entity_thresholds` revus pour le contexte métier (réduction faux positifs / données inutiles).
- [ ] NER ONNX : chemin modèle en lecture seule ; image sans shell interactif si possible (**distroless** / **minimal**).
- [ ] Logs : pas de texte brut PII dans les logs applicatifs ; niveau `info`/`warn` sans corps de requête complet.
- [ ] Métriques : exposition Prometheus derrière auth ou réseau admin uniquement.

### Passerelle Go (`aegis-gateway`)

- [ ] TLS terminé sur un proxy (ou TLS strict mTLS interne service-à-service).
- [ ] Timeouts, limites de taille de corps, rate limiting en amont (API gateway / cloud LB).
- [ ] CORS et en-têtes de sécurité (`HSTS`, `X-Content-Type-Options`, etc.) selon le front.

### Bases de données et cache

- [ ] PostgreSQL / Redis : auth forte, **pas** d’exposition publique ; chiffrement au repos activé côté cloud.
- [ ] Sauvegardes chiffrées ; tests de restauration périodiques.
- [ ] Comptes applicatifs distincts (lecture / écriture) si pertinent.

### CI / CD

- [ ] Branches protégées, revue obligatoire, statuts CI verts avant merge.
- [ ] **Dependabot** / renouvellement des dépendances ; `cargo audit`, `govulncheck`, scans image (Trivy) en CI.
- [ ] Releases : artefacts **cosign** + SBOM + provenance SLSA (voir workflows GitHub).

## Network policies (Kubernetes / équivalent)

- [ ] **Ingress** : uniquement ports 443 (et 80 → redirection HTTPS si nécessaire).
- [ ] **Egress** : restreindre les sorties du pod passerelle (DNS, APIs autorisées, registre d’images).
- [ ] **Namespace** dédié AEGIS ; `NetworkPolicy` refusant tout le trafic par défaut puis autorisation explicite vers :
  - service base de données / Redis ;
  - endpoint ONNX si hors cluster ;
  - observabilité (OTLP, si utilisé).
- [ ] **Service mesh** (optionnel) : mTLS entre services, politiques L7.

## Rotation des secrets et certificats

| Élément | Fréquence indicative | Action |
|--------|----------------------|--------|
| Secrets API / DB | 90 jours ou après incident | Rotation + mise à jour du secret manager ; redéploiement sans downtime (double clé si supporté). |
| Certificats TLS publics | Avant expiration (ACME ~ 60 j) | Automatiser (cert-manager, LB managé). |
| Clés de chiffrement anonymisation (FPE / AES) | Selon politique interne | Ré-encrypt ou rotation avec `key_id` ; documenter la procédure métier. |
| Tokens CI (crates.io, npm, PyPI) | Révoquer si fuite ; rotation annuelle | Stockage en secrets GitHub uniquement ; accès minimal. |
| Cosign / OIDC | Géré par GitHub Actions | Pas de clé longue durée locale si keyless. |

## Déploiement « secure by default »

- Préférer **private endpoints** pour l’administration (bastion, VPN, IAP).
- Désactiver les endpoints de debug et les profils Rust `debug` en prod.
- Surveiller les **CVE** des images de base ; reconstruire les images lors des correctifs upstream.

## Références internes au dépôt

- Workflows : `.github/workflows/security.yml`, `.github/workflows/release.yml`
- SBOM : `scripts/generate-sbom.sh`
- Signature CLI : `scripts/release-sign-cosign.sh`, `scripts/cosign-verify-aegis-cli.sh`

---

*Adapter les durées et outils au cadre légal et contractuel de votre organisation.*
