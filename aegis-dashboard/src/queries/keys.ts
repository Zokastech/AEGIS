// AEGIS — zokastech.fr — Apache 2.0 / MIT
/** Unique prefix + stable segments for invalidation / devtools (TanStack Query). */

export const queryKeys = {
  root: ["aegis"] as const,

  gatewayMetrics: (credential: string | null) =>
    [...queryKeys.root, "gateway-metrics", credential ?? ""] as const,

  /** Data keyed by gateway credential (API key / JWT / empty dev session). */
  policies: (credential: string | null) => [...queryKeys.root, "policies", credential ?? ""] as const,

  policyDpia: (credential: string | null, policyName: string) =>
    [...queryKeys.root, "policy-dpia", credential ?? "", policyName] as const,

  recognizers: (credential: string | null) => [...queryKeys.root, "recognizers", credential ?? ""] as const,

  auditExport: (credential: string | null) => [...queryKeys.root, "audit-export", credential ?? ""] as const,
};
