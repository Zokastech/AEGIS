// AEGIS — zokastech.fr — Apache 2.0 / MIT
import { describe, expect, it } from "vitest";
import {
  analyzeResultUsesMockEngine,
  anonymizedTextFromResponse,
  buildAnalysisConfigJson,
} from "@/lib/playground/analysis";
import {
  buildPlaygroundAnonymizeConfigJson,
  PLAYGROUND_ANONYMIZE_OPERATOR_IDS,
} from "@/lib/playground/anonymizeOperators";
import { dashboardSecurityHeaders } from "@/lib/securityHeaders";

describe("playground", () => {
  it("construit une config d’analyse valide", () => {
    const c = JSON.parse(buildAnalysisConfigJson("fr", 2, 0.6));
    expect(c).toMatchObject({ language: "fr", pipeline_level: 2, score_threshold: 0.6, return_decision_process: true });
  });

  it("chaque opérateur produit un JSON valide", () => {
    for (const op of PLAYGROUND_ANONYMIZE_OPERATOR_IDS) {
      const c = JSON.parse(buildPlaygroundAnonymizeConfigJson("fr", 1, 0.5, op));
      expect(c.operators_by_entity["*"]).toBeTruthy();
    }
  });

  it("détecte le moteur mock et extrait le texte anonymisé", () => {
    expect(analyzeResultUsesMockEngine({ result: { entities: [{ recognizer_name: "mock" }] } })).toBe(true);
    expect(analyzeResultUsesMockEngine({ result: { entities: [{ recognizer_name: "regex" }] } })).toBe(false);
    expect(analyzeResultUsesMockEngine(null)).toBe(false);
    expect(anonymizedTextFromResponse({ result: { anonymized: { text: "ok" } } })).toBe("ok");
  });
});

describe("en-têtes de sécurité", () => {
  it("contient les protections de base", () => {
    expect(dashboardSecurityHeaders["X-Content-Type-Options"]).toBe("nosniff");
    expect(dashboardSecurityHeaders["X-Frame-Options"]).toBe("DENY");
  });
});
