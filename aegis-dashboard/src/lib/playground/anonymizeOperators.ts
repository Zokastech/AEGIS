// AEGIS — zokastech.fr — Apache 2.0 / MIT
// Schema aligned with `FfiAnonymizeConfig` (crates/aegis-ffi): analysis + operators_by_entity / default_operator.

/** AES-256 / FPE key: 32 bytes as hex (64 chars). Playground demo only — do not reuse in production. */
export const PLAYGROUND_DEMO_KEY_HEX =
  "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

export const PLAYGROUND_ANONYMIZE_OPERATOR_IDS = [
  "redact",
  "replace",
  "mask",
  "hash",
  "encrypt",
  "fpe",
  "pseudonymize",
] as const;

export type PlaygroundAnonymizeOperatorId = (typeof PLAYGROUND_ANONYMIZE_OPERATOR_IDS)[number];

export const DEFAULT_PLAYGROUND_ANONYMIZE_OPERATOR: PlaygroundAnonymizeOperatorId = "replace";

/** Sample text (person, phone, email, IBAN, id) to exercise L1/L2/L3. */
export const PLAYGROUND_SAMPLE_TEXT: Record<string, string> = {
  fr: "Contact : Jean Dupont — tél. +33 6 12 34 56 78 — marie.durand@example.com — IBAN FR7630006000011234567890189 — NIR 1 85 05 75 806 043 75",
  en: "Contact: Jane Doe — phone +1 415 555 0199 — jane@example.com — IBAN GB82WEST12345698765432 — SSN 078-05-1120",
  de: "Kontakt: Max Mustermann — Tel. +49 170 1234567 — max@beispiel.de — IBAN DE89370400440532013000",
};

type OperatorPayload = { operator_type: string; params: Record<string, string> };

function operatorForWildcard(id: PlaygroundAnonymizeOperatorId): OperatorPayload {
  switch (id) {
    case "redact":
      return { operator_type: "redact", params: { replacement: "[REDACTED]" } };
    case "replace":
      return { operator_type: "replace", params: { numbered: "true" } };
    case "mask":
      return {
        operator_type: "mask",
        params: { visible_prefix: "1", visible_suffix: "2", mask_char: "*" },
      };
    case "hash":
      return {
        operator_type: "hash",
        params: { algorithm: "sha256", salt: "playground", truncate: "20" },
      };
    case "encrypt":
      return {
        operator_type: "encrypt",
        params: { key_hex: PLAYGROUND_DEMO_KEY_HEX, key_id: "playground" },
      };
    case "fpe":
      return {
        operator_type: "fpe",
        params: { key_hex: PLAYGROUND_DEMO_KEY_HEX, key_id: "playground" },
      };
    case "pseudonymize":
      return {
        operator_type: "pseudonymize",
        params: { salt: "playground", label: "PII" },
      };
  }
}

/**
 * JSON for `POST /v1/anonymize`: same `analysis` block as analyze + single operator on `*`
 * (all entity types detected in this pass).
 */
export function buildPlaygroundAnonymizeConfigJson(
  lang: string,
  pipeline: number,
  threshold: number,
  operator: PlaygroundAnonymizeOperatorId
): string {
  return JSON.stringify({
    analysis: {
      language: lang,
      pipeline_level: pipeline,
      score_threshold: threshold,
      return_decision_process: true,
    },
    operators_by_entity: {
      "*": operatorForWildcard(operator),
    },
  });
}
