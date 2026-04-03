// AEGIS — zokastech.fr — Apache 2.0 / MIT

/**
 * Exemple : plugin Fastify + **moteur natif** AEGIS (`fastify.aegis`) et route POST `/scan`.
 *
 * Prérequis : `npm run build` dans `sdk-nodejs/`.
 * Env : `AEGIS_ENGINE_CONFIG`, `AEGIS_ENGINE_LANGUAGES` (voir express-middleware.ts).
 *
 * Lancer : `npx tsx examples/fastify-plugin.ts`
 */

import type { FastifyPluginAsync, FastifyRequest } from 'fastify'
import Fastify from 'fastify'
import { AegisEngine, nativeAddonVersion, type AnalysisResult } from '../dist/index.js'

declare module 'fastify' {
  interface FastifyInstance {
    aegis: AegisEngine
  }
  interface FastifyRequest {
    aegisAnalysis?: AnalysisResult
  }
}

const aegisPlugin: FastifyPluginAsync<{ engine: AegisEngine }> = async (fastify, opts) => {
  fastify.decorate('aegis', opts.engine)

  fastify.addHook('preHandler', async (request: FastifyRequest) => {
    const body = request.body as { text?: string } | undefined
    const text = typeof body?.text === 'string' ? body.text : null
    if (text) {
      request.aegisAnalysis = await fastify.aegis.analyzeFull(text)
    }
  })
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
  const app = Fastify({ logger: true })

  await app.register(aegisPlugin, { engine })
  app.post('/scan', async (request) => {
    return { ok: true, analysis: request.aegisAnalysis ?? null }
  })

  await app.listen({ port: 3001, host: '0.0.0.0' })
  console.log('http://localhost:3001 — POST /scan {"text":"..."}')
}

try {
  await main()
} catch (e) {
  console.error(e)
}
