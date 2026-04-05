# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Conformité RGPD et traitement des données — AEGIS

**AEGIS** — [zokastech.fr](https://zokastech.fr)

Ce document résume les **mesures techniques et d’organisation** que le projet vise à faciliter, sans constituer un avis juridique. Validez toute mise en production avec votre DPO ou conseil.

## Fondements (Règlement UE 2016/679)

| Article     | Thème                                              | Application dans AEGIS                                                                                                                            |
| ----------- | -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Art. 5**  | Principes (minimisation, limitation des finalités) | Politiques YAML (`gdpr-strict`, `gdpr-analytics`) : actions par type d’entité, TTL des mappings                                                   |
| **Art. 25** | Privacy by design & by default                     | `data_minimization` dans les politiques ; passerelle : en-têtes sécurité, limitation de débit, option de ne pas journaliser le corps des requêtes |
| **Art. 32** | Sécurité du traitement                             | TLS, contrôle d’accès (clé API), en-têtes OWASP (voir ci-dessous)                                                                                 |
| **Art. 17** | Droit à l’effacement                               | Drapeau `erasure_endpoint_enabled` — endpoint complet à brancher sur vos magasins (Redis/PostgreSQL)                                              |
| **Art. 35** | DPIA                                               | Drapeau `dpia_auto_report` — rapport d’amorce à compléter par le responsable de traitement                                                        |

## Alignement OWASP (API / REST)

La passerelle `aegis-gateway` applique des pratiques tirées de l’**OWASP REST Security Cheat Sheet** (consultation via [Context7](https://context7.com) / OWASP Cheat Sheet Series) :

- **HTTPS** et **HSTS** (`Strict-Transport-Security`)
- **`X-Content-Type-Options: nosniff`**, **`X-Frame-Options: DENY`**, **`Cache-Control: no-store`**
- **Authentification** : clé API en en-tête (éviter les secrets dans l’URL — recommandation OWASP sur les informations sensibles dans les requêtes HTTP)
- **Rate limiting** pour limiter abus et déni de service

## Minimisation des données

1. Ne stocker que les **métadonnées** nécessaires (scores, types d’entités, latences) dans les journaux d’audit.
2. Préférer **masquage / pseudonymisation** avant envoi vers des LLM tiers (module proxy prévu dans le cahier de prompts).
3. Définir une **durée de rétention** explicite pour toute table de correspondance pseudonyme.

## Données sensibles (Art. 9)

La détection heuristique des catégories sensibles (santé, opinions, etc.) est **probabiliste**. Toute alerte doit être **validée humainement** avant toute décision automatisée.

## Prochaines étapes côté projet

- Journal d’audit immuable chaîné (hash) comme décrit dans le cahier de prompts module 6.
- Endpoint `DELETE /v1/subjects/{id}` branché sur les backends réels.
- Génération DPIA export PDF/Markdown.

---

_Document technique — ZokasTech / AEGIS._
