# AEGIS — guide de marque (zokastech.fr)

L’identité visuelle à jour (**Innovate · Connect · Grow**) est livrée dans le dossier **`aegis-branding/`** :

- Synthèse : [aegis-branding/brand-guide.md](../aegis-branding/brand-guide.md)
- Règles d’usage : [aegis-branding/guidelines.md](../aegis-branding/guidelines.md)
- Exports PNG, favicon `.ico` / `.webp`, bannières sociales : `cd aegis-branding && npm install && npm run export`

## Helm (annotations Kubernetes)

Les annotations `zokastech.fr/brand-primary`, `zokastech.fr/brand-accent`, etc. sont définies dans `deploy/helm/aegis/values.yaml` et `Chart.yaml`. Pour les aligner sur la palette officielle (`#0066ff`, `#ff6b35`, …), mettre à jour ces fichiers en même temps qu’une release chart.

## Mention recommandée

**AEGIS** · [zokastech.fr](https://zokastech.fr)
