# AEGIS — zokastech.fr — Apache 2.0 / MIT

# FAQ

## AEGIS remplace-t-il Microsoft Presidio sans adaptation ?

Conceptuellement oui pour les flux **analyze + anonymize**, mais les API et le YAML diffèrent. Voir [Migration depuis Presidio](migration-presidio.md).

## AEGIS appelle-t-il des services externes par défaut ?

Non. La détection s’exécute **sur votre infrastructure**. Les composants optionnels (téléchargement d’URL de modèle ONNX, proxy LLM amont) sont **explicitement configurés**.

## Quelle est la précision de la détection ?

Cela dépend de la langue du texte, du domaine et du niveau du pipeline. Les regex sont rapides mais peuvent manquer des paraphrases ; le NER améliore le rappel au prix du CPU. Lancez toujours une **évaluation métier** ([Évaluation](evaluation.md)).

## Où sont stockées les données personnelles ?

Par défaut, **en mémoire transitoire** pour analyze/anonymize. Le stockage persistant (Postgres, Redis, volume d’audit) apparaît lorsque vous activez des fonctions passerelle — adaptez votre DPIA en conséquence.

## Comment désactiver le NER ?

Régler `pipeline_level` à `1` ou `2`, ou omettre / décharger `ner.model_path`.

## Puis-je utiliser uniquement la bibliothèque Rust sans HTTP ?

Oui — dépendre de `aegis-core` + `aegis-regex` (+ `aegis-ner` si besoin) depuis le workspace et construire avec `AnalyzerEngineBuilder`.

## Comment signaler un problème de sécurité ?

Voir [`SECURITY.md`](https://github.com/zokastech/aegis/blob/main/SECURITY.md) — **ne pas** ouvrir d’issue publique pour des vulnérabilités non divulguées.

## Licences ?

Double licence **Apache 2.0** et **MIT** — voir `LICENSE` du dépôt.
