# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Migration depuis Microsoft Presidio

Ce guide met en correspondance les concepts Presidio avec AEGIS. Pour le **positionnement et le contexte concurrentiel**, voir [Pourquoi AEGIS](why-aegis.md). Validez avec votre **DPO** et des **données de test** avant de basculer le trafic de production.

## Correspondance générale

| Presidio | AEGIS |
|----------|-------|
| `AnalyzerEngine` | `AnalyzerEngine` (`aegis-core`) |
| Recognizers / registre | Trait `Recognizer` + `RecognizerRegistry` |
| `AnonymizerEngine` | `AnonymizerEngine` (`aegis-anonymize`) |
| Service REST analyzer | `aegis-gateway` `/v1/analyze` |
| Config YAML recognizers | `aegis-config.yaml` + `recognizers` / `pipeline` |
| Decision process | `return_decision_process`, `record_decision_trace` (avec parcimonie) |

## Étape 1 — Inventaire des artefacts Presidio

Lister recognizers activés, packs de langue, listes de refus et opérateurs d’anonymisation. Capturer des exemples de charges `analyze` et `anonymize`.

## Étape 2 — Cartographier les opérateurs

| Style Presidio | `operator_type` AEGIS |
|----------------|----------------------|
| Replace / redact | `replace`, `redact` |
| Mask | `mask` |
| Hash | `hash` |
| Encrypt / FPE | `encrypt`, `fpe` |

Vérifier que les flux **réversibles** utilisent une gestion de clés compatible (params `encrypt`/`fpe`).

## Étape 3 — Recréer l’intention de politique

Reporter les seuils **analyzer** Presidio vers `entity_thresholds` et les poids `pipeline`. Reporter les mots de **contexte** vers `context_scorer.languages`.

## Étape 4 — Migration API

Remplacer les appels HTTP Presidio par **`/v1/analyze`** et **`/v1/anonymize`** AEGIS. Les noms de champs diffèrent — voir [Référence API](api-reference.md).

## Étape 5 — Évaluation

Lancer des benchmarks côte à côte sur des jeux **sanitisés** : comparer comptages, spans et faux positifs. Voir [Évaluation](evaluation.md).

## Étape 6 — Déploiement progressif

- Blue/green ou canary sur la passerelle.
- Surveiller la latence (le NER ajoute de la latence de queue).
- Conserver un retour Presidio jusqu’à stabilisation des métriques.

---

AEGIS **n’est pas** compatible API avec Presidio sans couche d’adaptation — prévoir un middleware si vous devez conserver des chemins d’URL identiques.
