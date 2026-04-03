// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { createRequire } from 'node:module'
import { basename, dirname, join } from 'node:path'
import { Readable } from 'node:stream'
import { fileURLToPath } from 'node:url'

/** Entité PII détectée. */
export interface Entity {
  entityType: string
  start: number
  end: number
  text: string
  score: number
  recognizerName: string
  metadata: Record<string, string>
}

/** Enregistrement d’une transformation d’anonymisation. */
export interface TransformationRecord {
  entityStart: number
  entityEnd: number
  originalText: string
  replacement: string
  operator: string
  entityType: string
}

/** Résultat d’anonymisation. */
export interface AnonymizedResult {
  text: string
  transformations: TransformationRecord[]
  keyIdsUsed: string[]
  mappingHints: Record<string, string>
}

/** Résultat d’analyse (métadonnées + entités). */
export interface AnalysisResult {
  entities: Entity[]
  processingTimeMs: number
  languageDetected: string | null | undefined
  textLength: number
}

/** Options passées à `analyze` / `analyzeFull`. */
export interface AnalyzeOptions {
  language?: string | null
  entities?: string[] | null
  scoreThreshold?: number | null
}

/**
 * Configuration d’anonymisation (JSON).
 * Peut être soit la forme FFI complète (`analysis`, `operators_by_entity`, `default_operator`),
 * soit un objet `{ [ENTITY_TYPE]: { operator_type, ... } }` raccourci.
 */
export type AnonymizeOperatorsConfig = Record<string, unknown>

/** Un chunk de texte et son analyse (streaming). */
export interface StreamAnalysisChunk {
  chunk: string
  analysis: AnalysisResult
}

interface NativeEntity {
  entity_type: string
  start: number
  end: number
  text: string
  score: number
  recognizer_name: string
  metadata: Record<string, string>
}

interface NativeTransformationRecord {
  entity_start: number
  entity_end: number
  original_text: string
  replacement: string
  operator: string
  entity_type: string
}

interface NativeAnonymizedResult {
  text: string
  transformations: NativeTransformationRecord[]
  key_ids_used: string[]
  mapping_hints: Record<string, string>
}

interface NativeAnalysisResult {
  entities: NativeEntity[]
  processing_time_ms: number
  language_detected?: string | null
  text_length: number
}

interface NativeAnalyzeOptions {
  language?: string | null
  entities?: string[] | null
  score_threshold?: number | null
}

interface NativeEngine {
  analyze(text: string, options?: NativeAnalyzeOptions | null): Promise<NativeEntity[]>
  analyzeFull(text: string, options?: NativeAnalyzeOptions | null): Promise<NativeAnalysisResult>
  anonymize(text: string, operatorsJson?: string | null): Promise<NativeAnonymizedResult>
  analyzeBatch(texts: string[]): Promise<NativeEntity[][]>
  close(): void
}

interface NativeBinding {
  NativeAegisEngine: new (configPath?: string | null, languages?: string[] | null) => NativeEngine
  nativeAddonVersion: () => string
}

function mapEntity(n: NativeEntity): Entity {
  return {
    entityType: n.entity_type,
    start: n.start,
    end: n.end,
    text: n.text,
    score: n.score,
    recognizerName: n.recognizer_name,
    metadata: n.metadata ?? {},
  }
}

function mapAnalysis(n: NativeAnalysisResult): AnalysisResult {
  return {
    entities: (n.entities ?? []).map(mapEntity),
    processingTimeMs: n.processing_time_ms,
    languageDetected: n.language_detected ?? null,
    textLength: n.text_length,
  }
}

function mapAnonymized(n: NativeAnonymizedResult): AnonymizedResult {
  return {
    text: n.text,
    transformations: (n.transformations ?? []).map((t) => ({
      entityStart: t.entity_start,
      entityEnd: t.entity_end,
      originalText: t.original_text,
      replacement: t.replacement,
      operator: t.operator,
      entityType: t.entity_type,
    })),
    keyIdsUsed: n.key_ids_used ?? [],
    mappingHints: n.mapping_hints ?? {},
  }
}

function toNativeOptions(o?: AnalyzeOptions | null): NativeAnalyzeOptions | undefined {
  if (o == null) return undefined
  return {
    language: o.language ?? undefined,
    entities: o.entities ?? undefined,
    score_threshold: o.scoreThreshold ?? undefined,
  }
}

function loadNative(): NativeBinding {
  const require = createRequire(import.meta.url)
  const here = dirname(fileURLToPath(import.meta.url))
  const isDist = basename(here) === 'dist'
  const packageRoot = isDist ? join(here, '..') : here
  const triple = `${process.platform}-${process.arch}`
  const candidates = [
    join(packageRoot, 'aegis_pii_native.node'),
    join(packageRoot, `aegis_pii_native.${triple}.node`),
    join(here, 'aegis_pii_native.node'),
  ]
  for (const p of candidates) {
    try {
      return require(p) as NativeBinding
    } catch {
      /* essai suivant */
    }
  }
  throw new Error(
    `@aegis-pii/core: addon natif introuvable (essayé ${candidates.join(', ')}). Exécutez « npm run build:native » depuis sdk-nodejs/.`,
  )
}

const native = loadNative()

/** Version du module natif NAPI (Cargo). */
export function nativeAddonVersion(): string {
  return native.nativeAddonVersion()
}

/**
 * Moteur AEGIS : analyse et anonymisation PII via le cœur Rust.
 */
export class AegisEngine {
  readonly #engine: NativeEngine

  constructor(configPath?: string | null, languages?: string[] | null) {
    this.#engine = new native.NativeAegisEngine(configPath ?? undefined, languages ?? undefined)
  }

  /** Détecte les entités PII dans `text`. */
  async analyze(text: string, options?: AnalyzeOptions | null): Promise<Entity[]> {
    const raw = await this.#engine.analyze(text, toNativeOptions(options))
    return raw.map(mapEntity)
  }

  /** Analyse complète (latence, langue, longueur). */
  async analyzeFull(text: string, options?: AnalyzeOptions | null): Promise<AnalysisResult> {
    const raw = await this.#engine.analyzeFull(text, toNativeOptions(options))
    return mapAnalysis(raw)
  }

  /** Anonymise le texte selon `operators` (objet ou chaîne JSON). */
  async anonymize(
    text: string,
    operators?: AnonymizeOperatorsConfig | string | null,
  ): Promise<AnonymizedResult> {
    let json: string | undefined
    if (operators == null) json = undefined
    else if (typeof operators === 'string') json = operators
    else json = JSON.stringify(operators)
    const raw = await this.#engine.anonymize(text, json)
    return mapAnonymized(raw)
  }

  /** Analyse plusieurs textes (configuration d’analyse par défaut pour chaque). */
  async analyzeBatch(texts: string[]): Promise<Entity[][]> {
    const batches = await this.#engine.analyzeBatch(texts)
    return batches.map((row) => row.map(mapEntity))
  }

  /**
   * Analyse un flux Node.js : chaque morceau UTF-8 est analysé avec `analyzeFull`.
   * Découpage optionnel par lignes si `byLine` est vrai (sinon un appel par chunk binaire).
   */
  async *analyzeStream(
    readable: Readable,
    options?: AnalyzeOptions | null,
    streamOptions?: { byLine?: boolean },
  ): AsyncGenerator<StreamAnalysisChunk, void, undefined> {
    if (streamOptions?.byLine) {
      yield* this.#analyzeStreamByLine(readable, options)
      return
    }
    for await (const chunk of readable) {
      const text = Buffer.isBuffer(chunk) ? chunk.toString('utf8') : String(chunk)
      if (text.length === 0) continue
      const analysis = await this.analyzeFull(text, options)
      yield { chunk: text, analysis }
    }
  }

  async *#analyzeStreamByLine(
    readable: Readable,
    options?: AnalyzeOptions | null,
  ): AsyncGenerator<StreamAnalysisChunk, void, undefined> {
    let buf = ''
    for await (const chunk of readable) {
      buf += Buffer.isBuffer(chunk) ? chunk.toString('utf8') : String(chunk)
      const lines = buf.split('\n')
      buf = lines.pop() ?? ''
      for (const line of lines) {
        if (line.length === 0) continue
        const analysis = await this.analyzeFull(line, options)
        yield { chunk: line, analysis }
      }
    }
    if (buf.length > 0) {
      const analysis = await this.analyzeFull(buf, options)
      yield { chunk: buf, analysis }
    }
  }

  /** Libère le moteur côté natif. */
  close(): void {
    this.#engine.close()
  }
}
