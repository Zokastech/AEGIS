// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useAuthStore } from "@/stores/authStore";

/**
 * Credential sent to the gateway (API key, JWT, or `null` for dev bypass / no secret).
 * Single entry point for data hooks — matches `queryKeys` that include this credential.
 */
export function useAuthCredential(): string | null {
  return useAuthStore((s) => (s.devBypass ? null : s.token));
}
