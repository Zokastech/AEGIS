# AEGIS — Guidelines d’usage (Zokastech)

**Valeurs** : Innovate · Connect · Grow  
**Ton** : moderne, sécurisé, accessible, tech-forward.

## Principes

- **Minimalisme fonctionnel** : chaque élément a un rôle (pas de décor gratuit).
- **Géométrie nette** : bouclier, traits épais, angles marqués = solidité / sécurité.
- **Contraste** : viser WCAG AA minimum sur tout texte UI ; tester fond clair et fond charcoal.
- **Mouvement** : micro-interactions courtes (150–300 ms), easing doux ; pas d’animations agressives.
- **Gradients** : bleu → cyan discrets (headers, highlights), jamais saturés à l’excès.

## Composants UI

| Élément | Recommandation |
|--------|----------------|
| **CTA primaire** | Fond `#0066ff`, texte blanc ou `#f8f9fa`, `border-radius: 6px`, ombre légère |
| **Cartes** | Fond `#f8f9fa` (clair) ou `#1a1f3a` (sombre), bordure `#e5e7eb` si besoin |
| **Alertes** | Warning `#ff6b35`, succès `#10b981`, erreur `#ef4444`, texte lisible sur fond neutre |
| **Code** | Fond `#1a1f3a` ou `#0f172a`, syntaxe accentuée en `#00d9ff` / primaire |

## Logo

- **Marges de protection** : zone vide ≥ hauteur du « A » du mot AEGIS autour du lockup.
- **Ne pas** : étirer, rotation non 90°, changer les couleurs hors palette, ajouter des effets (neon, 3D cheap).
- **Fond sombre** : utiliser `aegis-inverted.svg` ou équivalent monochrome clair sur charcoal.

## Fichiers de référence

- Couleurs : `colors/palette.css`, `colors/palette.json`
- Typo : `typography/typography.css`
- Synthèse : `brand-guide.md`

## Export raster & favicon

```bash
cd aegis-branding && npm install && npm run export
```

Génère `logo/png/**`, `favicon.ico`, `favicon.webp`, PNG réseaux sociaux et badges.
