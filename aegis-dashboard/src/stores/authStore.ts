// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

type AuthState = {
  token: string | null;
  email: string | null;
  /**
   * "Gateway without auth" bypass: **not persisted**, only in dev when
   * `VITE_ENABLE_DEV_LOGIN_BYPASS=true` (see `LoginPage` / `authSession`).
   */
  devBypass: boolean;
  setAuth: (token: string, email: string) => void;
  /** Sign-in without a secret (local demo only). */
  setDevBypass: (email: string) => void;
  logout: () => void;
};

/**
 * `sessionStorage` persistence (smaller XSS blast radius than `localStorage`: tab closed = session ends).
 * `devBypass` is omitted from serialization.
 */
export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      token: null,
      email: null,
      devBypass: false,
      setAuth: (token, email) => set({ token, email, devBypass: false }),
      setDevBypass: (email) => set({ token: null, email, devBypass: true }),
      logout: () => set({ token: null, email: null, devBypass: false }),
    }),
    {
      name: "aegis-auth",
      storage: createJSONStorage(() => sessionStorage),
      partialize: (state) => ({ token: state.token, email: state.email }),
    }
  )
);
