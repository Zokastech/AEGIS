// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useAuthStore } from "@/stores/authStore";

const HYDRATE_FALLBACK_MS = 3000;

/**
 * Waits until persist rehydration (sessionStorage) finishes. Otherwise, on first load or F5,
 * `getState().token` is still `null` while the key exists in storage → spurious login redirects
 * and API calls without `X-API-Key`.
 *
 * Fallback timeout: if storage is unreadable, Zustand may never call `onFinishHydration` while
 * `hasHydrated` stays false.
 */
export function ensureAuthHydrated(): Promise<void> {
  return new Promise((resolve) => {
    if (useAuthStore.persist.hasHydrated()) {
      resolve();
      return;
    }
    let done = false;
    const finish = () => {
      if (done) return;
      done = true;
      window.clearTimeout(fallback);
      unsub();
      resolve();
    };
    const unsub = useAuthStore.persist.onFinishHydration(finish);
    const fallback = window.setTimeout(finish, HYDRATE_FALLBACK_MS);
  });
}

/**
 * Whether the user may access protected routes.
 *
 * - Non-empty token: normal auth (API key / Bearer).
 * - `devBypass`: Vite dev build only when `VITE_ENABLE_DEV_LOGIN_BYPASS=true` (never persisted — see `authStore`).
 */
export function isSessionValid(): boolean {
  const s = useAuthStore.getState();
  if (import.meta.env.DEV && import.meta.env.VITE_ENABLE_DEV_LOGIN_BYPASS === "true" && s.devBypass) {
    return true;
  }
  const t = s.token;
  return t != null && t.trim().length > 0;
}
