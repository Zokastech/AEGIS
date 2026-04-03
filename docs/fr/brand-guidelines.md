# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Charte graphique (Zokastech)

Conventions visuelles et UX pour les surfaces **AEGIS** (dashboard, landing, documentation) lorsqu’elles suivent l’identité **Zokastech**.

## Palette couleur

| Jeton | Hex | Usage |
|-------|-----|--------|
| **Orange** | `#ff8a00` | Primaire, CTA, accents (`brand.orange` en Tailwind) |
| **Rose / magenta** | `#e52e71` | Secondaire, liens type confidentialité, hovers alternatifs (`brand.pink`) |
| **Bleu** | `#4361ee` | Troisième couleur, dégradés, bandeaux (`brand.blue`) |

**Dégradé signature :** `linear-gradient(135deg, #ff8a00 → #e52e71 → #4361ee)`.

**Hover CTA (ex. boutons de démarrage) :** variante plus claire orange → rose.

### Neutres

- Fond de page : slate-50 (~`#f8fafc`) sur le `body`.
- Texte principal : slate-900 / `--zokastech-dark` `#1a1d24`.
- Gris de support : `--zokastech-gray` `#64748b` ; labels de formulaires `#475569` ; placeholders `#94a3b8`.

### Legacy / extension (Tailwind)

- `primary.200` : `#15151e` (noir bleuté).
- Secondaire étendu : violet / rose (`#912BBC`, `#D875C7`).

### Pied de page

- Fond : `#1a1d24`, texte blanc, liens `#f1f5f9`, survol lien orange `#ff8a00`.

### Formulaires

- Fond des champs blanc, bordure `#e2e8f0`, focus orange avec halo `rgba(255, 138, 0, 0.2)`.

### Statuts

- Succès : verts type `#2f855a` avec fond léger.
- Erreur : rouges avec lien visuel vers le rose marque (ex. bordure `#e52e71`).

### Ombres

- `shadow-brand` / `shadow-brand-lg` : ombres teintées orange `rgba(255, 138, 0, …)`.

## Typographie

- **Police :** Plus Jakarta Sans (Google Fonts).
- Graisses : 400, 500, 600, 700 + italiques 400 et 500.
- Pile : `"Plus Jakarta Sans", system-ui, sans-serif`.
- `antialiased` sur le `body`.

## Mise en page

- Conteneur centré : padding horizontal **20px** par défaut, **40px** à partir du breakpoint `md`.
- Variable CSS : `--header-height: 4rem` le cas échéant.

## Mouvement

- `scroll-behavior: smooth` sur `html` ; respect de `prefers-reduced-motion: reduce` (désactiver ou réduire les animations).
- Durées : **150ms**, **200ms**, **300ms** ; courbe `cubic-bezier(0.4, 0, 0.2, 1)`.
- Animations type Tailwind optionnelles : `fade-in` (8px), `slide-up` (16px), `gradient` (6s) — neutralisées si mouvement réduit.

## Patterns UI

- **Boutons forts :** dégradé orange → rose (→ bleu), texte blanc, hover léger `translateY(-2px)` et ombre rose/orange.
- **Hero / bandeaux :** cohérence avec l’orange marque (ex. `from-orange-500 to-orange-600` en Tailwind).
- **Cartes :** fond blanc, bordures gris clair, ombre modérée, coins arrondis `lg` / `xl` / `2xl` selon l’écran.

## Langue

- Langue du document HTML des sites publics : **fr** lorsque la cible est francophone ; la documentation technique reste multilingue via MkDocs.

## PDF / sceau

- Le sceau de signature reprend orange, rose, bleu sur fond très clair.

---

*Cette page résume les jetons de design pour les contributeurs ; l’implémentation se trouve dans le CSS/Tailwind de `aegis-dashboard` et `landing`.*
