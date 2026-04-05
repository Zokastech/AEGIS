// AEGIS — zokastech.fr — Apache 2.0 / MIT

/**
 * Fictional YAML snippet for the UI (demo / dev). The API does not expose real policy YAML.
 * Do not confuse with policies deployed on the gateway.
 */
export const POLICY_YAML_DEMO_SNIPPET = `# Illustrative sample only (dashboard demo) — not from your gateway.
policy:
  id: gdpr-strict-example
  version: "1"
  description: Synthetic policy shape for UI preview
  entity_actions:
    EMAIL:
      collect: true
      default_operator: redact
    PERSON:
      collect: true
      default_operator: mask
  retention_days: 30
  dpia_auto_report: true
`;

/** Local dev or explicit demo build (`VITE_SHOW_POLICY_YAML_DEMO=true`). */
export function showPolicyYamlDemo(): boolean {
  return import.meta.env.DEV || import.meta.env.VITE_SHOW_POLICY_YAML_DEMO === "true";
}
