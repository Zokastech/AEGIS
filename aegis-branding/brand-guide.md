# Zokastech AEGIS — Brand guide (condensé)

**Produit** : **Zokastech AEGIS** (moteur open-source de détection et d’anonymisation de PII) · **Éditeur** : [Zokastech](https://zokastech.fr) · **Site projet** : [aegis.zokastech.fr](https://aegis.zokastech.fr) (landing) · **Corporate** : [zokastech.fr](https://zokastech.fr)

**Nom d’usage** : écrire **« Zokastech AEGIS »** (titres, README, slides) ou **« AEGIS »** seul quand le contexte Zokastech est déjà clair (UI compacte, favicon).

---

## 1. Positionnement

**Zokastech AEGIS** incarne une alternative européenne orientée **conformité** (RGPD, packs policy) et **performance** (cœur Rust, passerelle durcie). L’identité visuelle traduit **protection** (bouclier géométrique), **clarté** (formes simples) et **innovation** (bleu primaire + cyan tech).

---

## 2. Logo

| Fichier | Usage |
|---------|--------|
| `logo/aegis-full.svg` | Lockup horizontal (bouclier + ZOKASTECH + AEGIS) |
| `logo/aegis-lockup-square.svg` | Variante carrée (icône + Zokastech + AEGIS) |
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

- **README** : `badges/powered-by.svg` / `.png` ; URL future : `https://aegis.zokastech.fr/badges/powered-by.svg` (« Powered by Zokastech AEGIS »)
- **Certification recognizers** : `badges/aegis-certified.svg` / `.png` (Zokastech AEGIS Certified)
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

Assets et documentation branding : dépôt **Zokastech/AEGIS** (Apache 2.0 / MIT selon modules). Crédit : **Zokastech** — produit **Zokastech AEGIS**.
