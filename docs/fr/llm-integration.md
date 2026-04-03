# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Intégration LLM (`aegis-llm-proxy`)

Le service **`aegis-llm-proxy`** se place entre vos applications et un LLM amont (compatible OpenAI, Anthropic, Ollama, …). Il peut **analyser** prompts/réponses avec le moteur AEGIS et appliquer les modes **transparent**, **anonymize**, **block** ou **alert**.

## Configuration (YAML)

Champs clés (voir `aegis-llm-proxy/internal/config/config.go`) :

| Champ | Description |
|-------|-------------|
| `listen` | Adresse d’écoute (surchargeable avec `AEGIS_LLM_LISTEN`) |
| `upstream_url` | URL de base du fournisseur |
| `mode` | `transparent` \| `anonymize` \| `block` \| `alert` |
| `score_threshold` | Transmis au moteur via `analysis_config_json` |
| `block_min_score` | En mode `block`, seuil de rejet |
| `webhook_url` | Pour le mode `alert` (POST JSON asynchrone) |
| `anonymize_config_json` | Profil d’opérateurs pour anonymisation automatique |
| `protected_entity_types` | Si défini, seuls ces types déclenchent la protection |
| `engine.type` | `http` (appel passerelle) ou `cli` (lancement `aegis`) |
| `engine.base_url` | URL passerelle si `type: http` |
| `engine.timeout_seconds` | Budget d’appel moteur |
| `inject_api_key_env` | Injecter `Authorization: Bearer` depuis l’env si le client omet |
| `dashboard` | Préfixe stats optionnel pour le dashboard AEGIS |

## Modes

| Mode | Comportement |
|------|--------------|
| `transparent` | Transfert du trafic ; journalisation / métriques optionnelles |
| `anonymize` | Réécriture des prompts (et optionnellement des réponses) après détection |
| `block` | Rejet des requêtes lorsque de la PII à haute confiance est trouvée |
| `alert` | Transfert mais notification `webhook_url` |

## Exécution locale

```bash
cd aegis-llm-proxy
go build -o aegis-llm-proxy ./cmd/aegis-llm-proxy
./aegis-llm-proxy -config config.yaml
```

Fournir un `config.yaml` valide pointant vers une **passerelle AEGIS** en marche ou un chemin CLI.

## LangChain

Utiliser le **client HTTP** de LangChain pointé vers le proxy plutôt que vers le fournisseur :

1. Régler `base_url` sur `http://localhost:<port-proxy>/v1` (ou votre chemin de montage).
2. Conserver la clé API fournisseur dans la config du proxy (`inject_api_key_env`) pour éviter de dupliquer les secrets dans l’app.

Le wrapper exact dépend de la version de LangChain ; conceptuellement le proxy est une **URL de base compatible OpenAI** lorsque les routes fournisseur OpenAI sont utilisées.

## LlamaIndex

Même schéma : configurer la classe LLM **OpenAI** ou **OpenAI-compatible** avec l’URL du proxy. Vérifier que les formes requête/réponse correspondent à ce que `aegis-llm-proxy` transmet (`internal/proxy/handler.go`).

## Notes de sécurité

- Terminer le **TLS** devant le proxy en production.
- Traiter `anonymize_config_json` et les clés API amont comme des **secrets**.
- Consulter le [modèle de menaces](security/threat-model.md) pour le risque résiduel de PII (faux négatifs).

---

## Sources

- Point d’entrée : `aegis-llm-proxy/cmd/aegis-llm-proxy/main.go`
- Handler : `aegis-llm-proxy/internal/proxy/handler.go`
