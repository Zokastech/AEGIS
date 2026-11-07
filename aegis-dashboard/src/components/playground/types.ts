// AEGIS — zokastech.fr — Apache 2.0 / MIT

/**
 * One step in the decision trace (AEGIS pipeline) shown in tooltips and the sidebar.
 */
export type DecisionTraceStep = {
  /** Technical step name (e.g. `regex_pre`, `score_gate`). */
  name: string;
  /** Whether this step validated the detection. */
  passed: boolean;
  /** Optional detail (e.g. regex pattern, context window). */
  detail?: string;
};

/**
 * Aggregated decision trace for a detected entity.
 */
export type DecisionTrace = {
  steps: DecisionTraceStep[];
  /** Pipeline level used during analysis. */
  pipelineLevel?: 1 | 2 | 3;
};

/**
 * Normalized entity for the playground UI. Prefer `playgroundEntityFromApi` to map gateway snake_case.
 */
export type PlaygroundEntity = {
  /** Stable id for React keys and anonymization links. */
  id: string;
  entityType: string;
  start: number;
  end: number;
  /** Source substring; if missing, derived from parent text via [start,end). */
  text?: string;
  /** Confidence score ∈ [0, 1]. */
  score: number;
  /** Recognizer or family (e.g. `eu_iban`, `email_v2`). */
  recognizer?: string;
  decisionTrace?: DecisionTrace;
};

/**
 * Link between an original segment and its anonymized span.
 */
export type AnonymizationLink = {
  id: string;
  /** For highlight color (e.g. `PERSON`, `EMAIL`). */
  entityType?: string;
  originalStart: number;
  originalEnd: number;
  anonymizedStart: number;
  anonymizedEnd: number;
};

/** Minimal API shape (gateway / analyze). */
export type ApiEntityLike = {
  /** String (ex. `EMAIL`) ou objet serde pour `Custom` : `{ "CUSTOM": "B-ORG" }`. */
  entity_type: string | Record<string, unknown>;
  start: number;
  end: number;
  text?: string;
  score?: number;
  recognizer?: string;
  decision_trace?: { steps?: Array<{ name: string; passed: boolean; detail?: string }>; pipeline_level?: number };
};

/**
 * Normalise `entity_type` JSON (Rust `EntityType`) en chaîne stable pour l’UI.
 * Les variantes `Custom` arrivent en externe-tag serde : `{ "CUSTOM": "label" }`.
 */
export function normalizeEntityType(raw: unknown): string {
  if (typeof raw === "string") return raw;
  if (raw != null && typeof raw === "object" && !Array.isArray(raw)) {
    const o = raw as Record<string, unknown>;
    const custom = o.CUSTOM ?? o.Custom ?? o.custom;
    if (typeof custom === "string" && custom.length > 0) {
      return `CUSTOM:${custom}`;
    }
    const keys = Object.keys(o);
    if (keys.length === 1) {
      const v = o[keys[0]];
      if (typeof v === "string") {
        return `${keys[0]}:${v}`;
      }
    }
  }
  if (raw === null || raw === undefined) return "UNKNOWN";
  try {
    return JSON.stringify(raw);
  } catch {
    return String(raw);
  }
}

/**
 * Maps an API entity to the playground model (camelCase + id).
 */
export function playgroundEntityFromApi(raw: ApiEntityLike, index: number): PlaygroundEntity {
  const steps = raw.decision_trace?.steps?.map((s) => ({ name: s.name, passed: s.passed, detail: s.detail })) ?? [];
  const pl = raw.decision_trace?.pipeline_level;
  const decisionTrace: DecisionTrace | undefined =
    steps.length > 0 || pl != null
      ? {
          steps,
          pipelineLevel: pl === 1 || pl === 2 || pl === 3 ? pl : undefined,
        }
      : undefined;

  const entityType = normalizeEntityType(raw.entity_type);
  return {
    id: `e-${index}-${raw.start}-${raw.end}`,
    entityType,
    start: raw.start,
    end: raw.end,
    text: raw.text,
    score: raw.score ?? 0,
    recognizer: raw.recognizer ?? entityType,
    decisionTrace,
  };
}
