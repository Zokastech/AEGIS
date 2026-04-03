# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Documentation AEGIS

**AEGIS** (*Advanced European Guardian for Information Security*) est un moteur open source de **détection et d’anonymisation des données personnelles (PII)** avec une posture **Europe d’abord**, développé par **[zokastech.fr](https://zokastech.fr)**.

## Ce que vous pouvez faire avec AEGIS

- Exécuter **regex + score contextuel + NER ONNX optionnel** sur du texte libre.
- **Anonymiser** les segments (redaction, masquage, hachage, remplacement, chiffrement, FPE, pseudonymisation).
- Exposer les capacités via une **passerelle HTTP durcie** (Go), une **CLI** (Rust) ou des **SDK** (Python, Node, Java — maturité variable ; voir le dépôt).

## Par où commencer

| Je veux… | Lire |
|----------|------|
| Comprendre **pourquoi AEGIS** vs Presidio, Macie et outils comparables | [Pourquoi AEGIS — paysage concurrentiel](why-aegis.md) |
| Essayer AEGIS en 5 minutes | [Démarrage](getting-started.md) |
| Lancer des exemples Python, Node ou notebooks | [Exemples](examples.md) |
| Comprendre le pipeline à 3 niveaux | [Architecture](architecture.md) |
| Appeler l’API REST | [Référence API](api-reference.md) |
| Régler `aegis-config.yaml` | [Configuration](configuration.md) |
| Utiliser le Playground du dashboard (seuil de confiance) | [Dashboard — Playground](dashboard-playground.md) |
| Voir les détecteurs intégrés | [Recognizers](recognizers.md) |
| Déployer en production | [Déploiement](deployment.md) + [Sécurité — vue d’ensemble](security/index.md) |
| Configurer le backend Zokastech / contexte site | [Plateforme Zokastech](zokastech.md) |
| Déployer AEGIS sur AWS, GCP, Azure, OVH | [Fournisseurs cloud](cloud-providers.md) |

## Sur le site ZokasTech

La documentation complète est également disponible sur **[zokastech.fr/aegis/docs](https://zokastech.fr/aegis/docs)** dans l’application ZokasTech, avec le contenu source en anglais et l’identité visuelle ZokasTech.

## Langues

Ce site est disponible en **anglais** (par défaut), **français**, **allemand**, **espagnol** et **italien** via le sélecteur de langue. Les pages sans traduction **repassent automatiquement sur l’anglais** (*fallback*).

## Licence

Le projet est distribué sous **Apache 2.0** et **MIT** (double licence) ; voir le fichier `LICENSE` du dépôt.

## Contribuer aux traductions

Les pages françaises vivent sous `docs/fr/` avec la même arborescence que `docs/en/`. Le plugin **mkdocs-static-i18n** les associe à la locale **Français**. Pour le workflow général du dépôt, voir [Contribuer](contributing.md).

## Charte graphique

Pour l’identité visuelle Zokastech (couleurs, typo, composants), voir [Charte graphique](brand-guidelines.md).
