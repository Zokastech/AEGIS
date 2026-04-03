// AEGIS — zokastech.fr — Apache 2.0 / MIT

/**
 * Recommended HTTP response headers for the SPA (dev / Vite preview).
 * See [Vite `server.headers` / `preview.headers`](https://vitejs.dev/config/server-options.html#server-headers).
 */
export const dashboardSecurityHeaders: Record<string, string> = {
  "X-Content-Type-Options": "nosniff",
  "X-Frame-Options": "DENY",
  "Referrer-Policy": "strict-origin-when-cross-origin",
  "Permissions-Policy": "camera=(), microphone=(), geolocation=(), interest-cohort=()",
  "Cross-Origin-Opener-Policy": "same-origin",
};
