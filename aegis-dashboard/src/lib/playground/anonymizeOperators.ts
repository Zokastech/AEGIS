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

/**
 * Texte long multi-entités pour le pipeline L3 (NER ONNX **ZOKA-SENTINEL**).
 * Aligné sur les jeux de régression internes (personnes, dates, lieux, emails, téléphone, IBAN, org).
 */
export const ZOKA_SENTINEL_DEMO_TEXT: Record<string, string> = {
  fr: [
    "Objet : Dossier composite — client Yacine Ben Salah / Bensaleh, né le 03-11-1991 (11/03/91) à Tourcoing.",
    "Coordonnées : y.bensalah91@gmail.com, yacine_bs@protonmail.com, +33 (0)7 44 91 23 88.",
    "Employeur : Sarl DataXpert — 22 rue Nationale, Lille — TJ-LILLE-2024/008771.",
    "Paiement : IBAN FR14 2004 1010 0505 0001 3M02 606, BIC PSSTFRPPXXX — carte Mastercard 5326 **** **** 9087.",
    "Identifiants : passeport 98FR76X12345, permis BSYAC91110359, référence dossier EU-RGPD-TEST-009X.",
  ].join("\n"),
  en: [
    "Subject: Composite file — client Alex Morgan, DOB 14-08-1990 (08/14/90) in Manchester.",
    "Contact: alex.morgan@protonmail.com, backup alex_m@data-io.example, +44 7700 900555.",
    "Employer: Northwind Data Ltd — 10 Baker Street, London — ref NW-2024-8891.",
    "Payment: IBAN GB82WEST12345698765432, BIC NWBKGB2L — Visa 4242 **** **** 4242.",
    "IDs: passport 123456789, driving license AB12CDE3456789, case ref GDPR-UK-TEST-001A.",
  ].join("\n"),
  de: [
    "Akte: Person Anna Weber / Weber-Schmidt, geb. 22-04-1988 in München.",
    "Kontakt: anna.weber@firma.de, privat anna_w@posteo.de, +49 176 12345678.",
    "Arbeitgeber: Daten GmbH — Hauptstraße 5, Berlin — Steuer-ID DE123456789.",
    "Zahlung: IBAN DE89370400440532013000, BIC COBADEFFXXX.",
    "Ausweis: Reisepass L01X00T47, Führerschein M-AB 123 456.",
  ].join("\n"),
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
