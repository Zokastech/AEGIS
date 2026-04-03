// AEGIS — zokastech.fr — Apache 2.0 / MIT

/**
 * Exemple : route API Next.js (App Router) — analyse via le **moteur natif** `@aegis-pii/core`.
 *
 * Copiez vers `app/api/aegis/analyze/route.ts`, installez le package publié ou `file:../sdk-nodejs`,
 * et exécutez `npm run build` dans le SDK pour générer `aegis_pii_native*.node`.
 *
 * Singleton : évite de recharger le binaire NAPI à chaque requête (recommandé en prod).
 * Configuration : `AEGIS_ENGINE_CONFIG`, `AEGIS_ENGINE_LANGUAGES` (lus au premier appel).
 */

import { AegisEngine, nativeAddonVersion, type AnalysisResult } from '@aegis-pii/core'

let engineSingleton: AegisEngine | null = null

function getEngine(): AegisEngine {
  if (!engineSingleton) {
    const configPath = process.env.AEGIS_ENGINE_CONFIG?.trim() || undefined
    const langsRaw = process.env.AEGIS_ENGINE_LANGUAGES?.trim()
    const languages = langsRaw
      ? langsRaw
          .split(',')
          .map((s) => s.trim())
          .filter(Boolean)
      : undefined
    engineSingleton = new AegisEngine(configPath ?? null, languages ?? null)
    console.info('[aegis] native addon', nativeAddonVersion())
  }
  return engineSingleton
}

export async function POST(
  request: Request,
): Promise<Response> {
  try {
    const body = (await request.json()) as { text?: string }
    const text = typeof body.text === 'string' ? body.text : ''
    if (!text) {
      return Response.json({ error: 'missing text' }, { status: 400 })
    }
    const analysis: AnalysisResult = await getEngine().analyzeFull(text)
    return Response.json({ analysis })
  } catch (e) {
    const message = e instanceof Error ? e.message : 'analyze failed'
    return Response.json({ error: message }, { status: 500 })
  }
}
