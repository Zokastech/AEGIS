# AEGIS — zokastech.fr — Apache 2.0 / MIT

## Politique de sécurité

Merci d’aider à protéger les utilisateurs d’**AEGIS** en signalant les vulnérabilités de façon responsable.

### Divulgation responsable

1. **Ne pas** ouvrir d’issue GitHub publique pour un problème de sécurité non corrigé.
2. Envoyez un rapport détaillé à l’équipe : **[security@zokastech.fr](mailto:security@zokastech.fr)** (remplacez par l’alias réel de votre organisation si différent).
3. Incluez : description, impact, étapes de reproduction, versions / commits concernés, et idéalement une proposition de correctif ou de contournement.

### Chiffrement PGP (rapports sensibles)

Pour les informations particulièrement sensibles, vous pouvez chiffrer le corps du message avec la clé PGP du projet :

- **Fingerprint** : `0000 0000 0000 0000 0000  0000 0000 0000 0000 0000` *(à remplacer par la clé réelle publiée sur [keys.openpgp.org](https://keys.openpgp.org) ou dans ce dépôt sous `docs/en/security/`)*
- **Clé publique** : fichier `docs/en/security/zokastech-security.asc` *(à ajouter lorsque la clé est créée)*

Sans clé publique publiée, privilégiez un canal privé convenu avec l’équipe.

### Engagements de réponse (SLA indicatif)

| Gravité      | Exemples                               | Première réponse cible |
|-------------|-----------------------------------------|-------------------------|
| **Critique** | RCE, fuite massive de clés, contournement auth complet | **48 h ouvrées**        |
| **Élevée**   | XSS stockée, injection majeure, déni de service facile | 5 jours ouvrés          |
| **Moyenne**  | Fuites d’info limitées, CSRF ciblée     | 10 jours ouvrés         |
| **Faible**   | Durcissements, faible impact            | Meilleur effort         |

Ces délais sont des **objectifs** ; la charge et les fuseaux peuvent les modifier. Nous accuserons réception dès que possible.

### Ce que nous faisons après réception

- Analyse et reproduction ; demande d’informations complémentaires si besoin.
- Correctifs sur les branches maintenues ; **CVE** coordonnée si pertinent.
- Crédit dans les notes de version (sauf demande d’anonymat).

### Bug bounty

Il n’y a **pas** de programme de bug bounty rémunéré public pour l’instant. Les recherches **non destructives** sur les déploiements que vous possédez ou sur les environnements de démonstration explicitement autorisés sont les bienvenues. Merci de respecter la loi et les conditions d’usage des services tiers.

**Hors scope (typiquement)** : déni de service sur infrastructure partagée, spam social engineering, fuites de données d’autres utilisateurs, tests sur des systèmes sans autorisation écrite.

### Vérification des artefacts (Sigstore / cosign)

- **Image Docker** (passerelle) :  
  `cosign verify ghcr.io/<org>/aegis-gateway@<digest_SHA256>`  
  (identité OIDC GitHub Actions selon la config du dépôt.)

- **Archive CLI** (fichier + bundle générés en release) :  
  Voir `scripts/cosign-verify-aegis-cli.sh` et définir `COSIGN_CERT_IDENTITY_REGEX` / `COSIGN_CERT_OIDC_ISSUER_REGEX` selon votre organisation.

### SBOM

Des SBOM (SPDX / CycloneDX) sont générés via `scripts/generate-sbom.sh` et publiés comme artefacts de release lorsque le pipeline CI est activé.

### Dashboard (SPA)

- **Jetons** : les identifiants de session du tableau de bord sont stockés dans `sessionStorage` (pas `localStorage`) pour limiter la persistance après fermeture d’onglet.
- **Contournement dev** : la case « sans auth » (gateway en `development.disable_auth`) n’existe qu’en build de développement et uniquement si `VITE_ENABLE_DEV_LOGIN_BYPASS=true`. **Ne jamais** activer ce flag dans une image ou un déploiement de production.
- **En-têtes** : le conteneur Nginx du dashboard et la config Vite (`server.headers` / `preview.headers`) appliquent des en-têtes de durcissement (nosniff, frame denial, COOP, etc.) alignés sur `aegis-dashboard/src/lib/securityHeaders.ts`.

### Provenance SLSA (niveau 3 — générateur générique)

Le workflow [`.github/workflows/release.yml`](.github/workflows/release.yml) appelle le générateur officiel [SLSA GitHub Generator](https://github.com/slsa-framework/slsa-github-generator) (`generator_generic_slsa3`) après publication de la release, et atteste les hachages des archives CLI, des bundles cosign associés et des artefacts SBOM. Vérification indicative avec [slsa-verifier](https://github.com/slsa-framework/slsa-verifier) sur le fichier de provenance joint à la release (nom typique `multiple.intoto.jsonl`).

---

*Dernière mise à jour : document modèle — remplacer les placeholders PGP / e-mail par les valeurs officielles zokastech.fr.*
