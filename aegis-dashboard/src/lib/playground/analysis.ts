// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { playgroundEntityFromApi, type PlaygroundEntity, type ApiEntityLike } from "@/components/playground";

/** Gateway built without CGO returns stub entities (`recognizer_name: "mock"`). */
export function analyzeResultUsesMockEngine(result: unknown): boolean {
  if (!result || typeof result !== "object") return false;
  const o = result as Record<string, unknown>;
  const inner = o.result ?? o;
  if (typeof inner !== "object" || inner === null) return false;
  const raw = (inner as { entities?: { recognizer_name?: string }[] }).entities;
  if (!Array.isArray(raw)) return false;
  return raw.some((e) => e && String(e.recognizer_name ?? "").toLowerCase() === "mock");
}

export function extractEntitiesFromAnalyzeResult(result: unknown): PlaygroundEntity[] {
  if (!result || typeof result !== "object") return [];
  const o = result as Record<string, unknown>;
  const inner = o.result ?? o;
  if (typeof inner !== "object" || inner === null) return [];
  const raw = (inner as { entities?: ApiEntityLike[] }).entities;
  if (!Array.isArray(raw)) return [];
  return raw.map((e, i) => playgroundEntityFromApi(e, i));
}

/** Sets `return_decision_process` to populate `decision_trace` on each entity (L1/L2/L3). */
export function buildAnalysisConfigJson(lang: string, pipeline: number, threshold: number): string {
  return JSON.stringify({
    language: lang,
    pipeline_level: pipeline,
    score_threshold: threshold,
    return_decision_process: true,
  });
}

export function anonymizedTextFromResponse(data: unknown): string {
  const o = data as { result?: { anonymized?: { text?: string } } };
  return o?.result?.anonymized?.text ?? JSON.stringify(data, null, 2);
}
