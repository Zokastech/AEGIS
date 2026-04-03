# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Contribuer

Merci d’aider à améliorer **AEGIS** ([zokastech.fr](https://zokastech.fr)).

Le dépôt contient aussi un [`CONTRIBUTING.md`](https://github.com/zokastech/aegis/blob/main/CONTRIBUTING.md) à la racine — gardez les deux alignés lorsque le processus évolue.

## Workflow

1. **Fork** et branche fonctionnelle depuis `main`.
2. Lancer les **tests** en local (`cargo test`, `go test ./...` dans les modules modifiés).
3. Commits ciblés ; respecter le **style** existant (rustfmt, gofmt).
4. Ouvrir une **Pull Request** avec description claire, issues liées, notes de migration si le comportement change.

## Sécurité

**Ne pas** ouvrir de PR publique pour des vulnérabilités non divulguées — suivre [`SECURITY.md`](https://github.com/zokastech/aegis/blob/main/SECURITY.md).

## Juridique

En contribuant, vous acceptez que vos contributions soient sous les termes de la double licence du projet **Apache 2.0** et **MIT**.

## Documentation

- Mettre à jour les pages **MkDocs** sous `docs/en/` (et `docs/fr/` si vous traduisez) lorsque le comportement utilisateur change.
- Lancer `mkdocs serve` en local après `pip install -r requirements-docs.txt`.

## Recognizers et langues

Les nouveaux recognizers regex doivent inclure des **tests unitaires** et déclarer correctement `supported_languages`.

## Questions

Utiliser **GitHub Discussions** (si activé) ou les issues pour la conception ; garder les issues actionnables.
