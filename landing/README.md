# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Landing page — aegis.zokastech.fr

Site vitrine statique (Vite + React + TypeScript), optimisé pour **Lighthouse** : pas de polices externes, bundle minimal, `prefers-reduced-motion` respecté.

## Développement

```bash
cd landing
npm install
npm run dev
```

## Build production

```bash
npm run build
```

Sortie : `landing/dist/` — à servir derrière **nginx**, **Cloud CDN**, **GitHub Pages** ou **Cloud Run** (fichiers statiques).

## Déploiement (exemple)

- **GitHub Pages** : `npm run build` puis publier le contenu de `dist/` sur la branche `gh-pages` ou via action dédiée.
- **Sous-domaine** : pointer `aegis.zokastech.fr` vers l’hébergeur avec TLS (Let’s Encrypt).

## Contenu

- Hero avec bouclier animé (CSS)
- Comparaison Presidio / AEGIS + 4 cartes
- Pipeline 3 niveaux (révélation au scroll)
- Carte UE stylisée + liste formats
- Snippet Python copiable + terminal animé
- Étoiles GitHub via `api.github.com` (sans token, rate limit public)

## Personnalisation

Variables visuelles dans `src/index.css` (`:root`).  
Liens : `DOCS_URL`, `GITHUB_REPO` dans `src/App.tsx`.
