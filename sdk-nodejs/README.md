# @aegis-pii/core

**AEGIS by [zokastech.fr](https://zokastech.fr)** — SDK Node.js / TypeScript avec liaisons **NAPI-RS** vers le moteur Rust (analyse / anonymisation).

## Build

```bash
cd sdk-nodejs
npm install
npm run build
```

`build` exécute `napi build` puis `tsc` : le fichier `aegis_pii_native*.node` doit être présent à côté de `dist/` pour charger le **vrai moteur**.

## Utilisation

```typescript
import { AegisEngine, nativeAddonVersion } from '@aegis-pii/core'

console.log(nativeAddonVersion())
const engine = new AegisEngine(configPathOrNull, ['fr', 'en'])
const analysis = await engine.analyzeFull('contact@example.com')
```

Variables d’environnement supportées par les exemples : `AEGIS_ENGINE_CONFIG`, `AEGIS_ENGINE_LANGUAGES`.

Pour consommer AEGIS **via HTTP** (gateway) plutôt qu’en in-process, voir `examples/nodejs/` et `examples/python/quickstart.py` à la racine du dépôt.

## Exemples

- `examples/express-middleware.ts`
- `examples/fastify-plugin.ts`
- `examples/next-api-route.ts`

```bash
npx tsx examples/express-middleware.ts
```

## Licence

Apache-2.0 et MIT — voir [LICENSE](../LICENSE).
