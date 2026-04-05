# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Dashboard — Seuil de confiance (Playground)

Le **Playground** du dashboard AEGIS sert à expérimenter la détection. Le **seuil de confiance** est lié au paramètre moteur `score_threshold` et à un **aperçu** qui se met à jour quand vous bougez le curseur.

---

## « Au-dessus du seuil / en dessous » : signification

Le texte d’aide :

- **Au-dessus du seuil** → entité **retenue**
- **En dessous du seuil** → entité **ignorée**

concerne le **score final** de chaque segment détecté (après le pipeline : recognizers L1, contexte L2 optionnel, NER L3 optionnel, puis fusion). Pour chaque entité, le moteur applique :

`entity.score >= score_threshold`

- Si la condition est **vraie**, l’entité figure dans le JSON renvoyé par `POST /v1/analyze`.
- Si elle est **fausse**, l’entité est **écartée** et n’apparaît pas dans la réponse.

Le Playground envoie cette valeur comme **`score_threshold`** dans `analysis_config_json` (le même bloc est utilisé pour **Anonymiser**, pour garder analyse et anonymisation cohérentes).

Les scores sont toujours dans l’intervalle **[0, 1]**.

---

## La ligne : « À 0,75 : 1 détectées · 0 ignorées »

Cette ligne est un **résumé en direct à partir du dernier résultat d’analyse**, pas un second appel API caché :

1. Vous cliquez sur **Analyser** → la passerelle renvoie des entités, chacune avec un `score`.
2. Le curseur fixe un seuil, par ex. **0,75**.
3. L’interface compte :
   - **Détectées** = nombre d’entités de ce résultat avec `score ≥ 0,75` (elles seraient encore renvoyées si vous relanciez Analyser avec le seuil 0,75).
   - **Ignorées** = le reste (`score < 0,75`) ; au prochain appel avec ce seuil, le moteur les **filtre**.

**Exemple :** un e-mail avec le score **0,92** et le seuil **0,75** → **1 détectées · 0 ignorées**, car `0,92 ≥ 0,75`.

Déplacer le curseur met à jour **uniquement** cet aperçu (et le graphique) tant que vous n’avez pas relancé **Analyser**. Pour appliquer un nouveau seuil côté API, cliquez de nouveau sur **Analyser** (ou **Anonymiser**) après avoir réglé le curseur.

---

## Graphique sous le curseur

Les courbes montrent, pour plusieurs seuils possibles sur l’axe horizontal, combien d’entités issues de la **dernière** analyse compteraient comme **détectées** ou **ignorées**. Cela permet de comparer des seuils sans multiplier les appels.

---

## Lien avec `aegis-config.yaml`

- **`analysis.score_threshold`** (et le champ équivalent envoyé par le Playground) est le seuil **par requête / par défaut** pour émettre une entité dans cette passe.
- **`entity_thresholds`** et **`pipeline.output_score_threshold`** sont des règles **supplémentaires** côté serveur ; voir [Configuration](configuration.md).

Pour les noms de champs et le JSON, voir [Référence API](api-reference.md).
