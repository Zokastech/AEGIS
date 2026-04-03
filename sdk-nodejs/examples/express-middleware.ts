// AEGIS — zokastech.fr — Apache 2.0 / MIT

/**
 * Exemple : middleware Express qui analyse le corps JSON `{ "text": "..." }` via le **moteur natif Rust** (NAPI).
 *
 * Prérequis : depuis `sdk-nodejs/` exécuter `npm run build` (binaire `.node` + TypeScript).
 * Variables optionnelles : `AEGIS_ENGINE_CONFIG` (chemin fichier config moteur), `AEGIS_ENGINE_LANGUAGES` (ex. `fr,en`).
 *
 * Lancer : `npx tsx examples/express-middleware.ts`
 */

import type { Request, Response, NextFunction } from 'express'
import express from 'express'
import { AegisEngine, nativeAddonVersion, type AnalysisResult } from '../dist/index.js'

declare global {
  namespace Express {
    interface Request {
      aegisAnalysis?: AnalysisResult
    }
  }
}

export function aegisBodyAnalyzerMiddleware(engine: AegisEngine) {
  return async (req: Request, res: Response, next: NextFunction) => {
    try {
      const text = typeof req.body?.text === 'string' ? req.body.text : null
      if (text) {
        req.aegisAnalysis = await engine.analyzeFull(text)
      }
      next()
    } catch (e) {
      next(e)
    }
  }
}

function engineFromEnv(): AegisEngine {
  const configPath = process.env.AEGIS_ENGINE_CONFIG?.trim() || undefined
  const langsRaw = process.env.AEGIS_ENGINE_LANGUAGES?.trim()
  const languages = langsRaw
    ? langsRaw
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean)
    : undefined
  return new AegisEngine(configPath ?? null, languages ?? null)
}

async function main() {
  const engine = engineFromEnv()
  console.info('[aegis] native addon', nativeAddonVersion())
  const app = express()
  app.use(express.json())
  app.use(aegisBodyAnalyzerMiddleware(engine))

  app.post('/scan', (req, res) => {
    res.json({ ok: true, analysis: req.aegisAnalysis ?? null })
  })

  app.listen(3000, () => {
    console.log('http://localhost:3000 — POST /scan {"text":"..."}')
  })
}

try {
  await main()
} catch (e) {
  console.error(e)
}
