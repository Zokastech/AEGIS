# Shields.io — badges personnalisés Zokastech AEGIS

Badges dynamiques pour README et documentation (produit **Zokastech AEGIS**). Remplacez `USER` / `REPO` si besoin.

## Exemples (style flat-square)

```markdown
![Security](https://img.shields.io/badge/security--level-high-blue?style=flat-square)
![Zokastech AEGIS](https://img.shields.io/badge/Zokastech%20AEGIS-1.0.0-blue?style=flat-square)
![Compatible](https://img.shields.io/badge/compatible-rust%20%7C%20go%20%7C%20python-orange?style=flat-square)
```

### URLs encodées

- `security-level=high` → label `security-level`, message `high`, couleur `blue`
- `Zokastech AEGIS=1.0.0` (label/message encodés) → version produit, couleur `blue`
- `compatible=rust | go | python` → encoder les `|` en `%7C` dans l’URL

### Liens vers les assets officiels

```markdown
[![Powered by Zokastech AEGIS](https://aegis.zokastech.fr/badges/powered-by.svg)](https://aegis.zokastech.fr)
```

*(À activer quand le domaine sert les fichiers statiques.)*

## Hébergement local / repo

```markdown
[![Powered by Zokastech AEGIS](../badges/powered-by.svg)](https://github.com/Zokastech/AEGIS)
```

Pour PNG (README GitHub sans SVG inline) : générer via `npm run export` dans `aegis-branding/`.
