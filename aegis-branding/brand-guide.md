# AEGIS — Brand guide (condensé)

**Projet** : AEGIS Framework · **Client** : Zokastech · **Site** : [zokastech.fr](https://zokastech.fr)

---

## 1. Positionnement

Framework **cybersécurité & cloud** : accessible, moderne, orienté développeurs et équipes infra. L’identité visuelle traduit **protection** (bouclier géométrique), **clarté** (formes simples) et **innovation** (bleu primaire + cyan tech).

---

## 2. Logo

| Fichier | Usage |
|---------|--------|
| `logo/aegis-full.svg` | Lockup horizontal (bouclier + AEGIS) |
| `logo/aegis-lockup-square.svg` | Variante carrée (icône + mot dessous) |
| `logo/aegis-icon.svg` | Icône seule (favicon, app, watermark) |
| `logo/aegis-mono.svg` | Noir / blanc cassé (print, contraintes 1 couleur) |
| `logo/aegis-inverted.svg` | Fonds sombres (`#1a1f3a`, etc.) |
| `logo/aegis-icon-mono.svg` | Icône monochrome sur fond transparent |

Exports PNG & favicons : voir `guidelines.md` (`npm run export`).

---

## 3. Couleurs (résumé)

| Token | Hex | Rôle |
|-------|-----|------|
| Primaire | `#0066ff` | CTA, liens forts, logo |
| Secondaire | `#1a1f3a` | Surfaces sombres |
| Accent chaud | `#ff6b35` | Alertes, performance |
| Accent tech | `#00d9ff` | Données, réseau, code |
| Succès | `#10b981` | Validation |
| Texte | `#0f172a` | Corps principal |

Détail : `colors/palette.md`.

---

## 4. Typographie

- **Titres (H1–H3)** : Inter Tight, 700–900  
- **Titres (H4–H6) & corps** : Inter, 400–700  
- **Code** : JetBrains Mono, 400–600  

Imports CSS : `typography/typography.css`.

---

## 5. Badges & social

- **README** : `badges/powered-by.svg` / `.png` ; URL future : `https://aegis.zokastech.fr/badges/powered-by.svg`
- **Certification recognizers** : `badges/aegis-certified.svg` / `.png`
- **Shields.io** : `badges/shields-io.md`
- **Bannières** : `social-media/*.svg` (+ `.png` après export) — GitHub 1280×640, Twitter 1200×675, LinkedIn 1500×500, OG 1200×630

---

## 6. PDF (1–2 pages)

Ce document sert de **source unique** pour une version PDF : ouvrir `brand-guide.md` dans un éditeur Markdown (VS Code, Typora, etc.) et **Exporter / Imprimer en PDF**, ou utiliser Pandoc :

```bash
pandoc brand-guide.md -o aegis-brand-guide.pdf --pdf-engine=wkhtmltopdf
```

*(Adapter selon l’outil disponible sur votre poste.)*

---

## Licence

Assets et documentation branding : alignés sur le dépôt AEGIS (Apache 2.0 / MIT selon modules). Crédit : **Zokastech**.
