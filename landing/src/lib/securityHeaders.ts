// AEGIS — zokastech.fr — Apache 2.0 / MIT

/**
 * HTTP response headers for the static site (Vite dev / preview).
 * @see https://vitejs.dev/config/server-options.html#server-headers
 */
export const landingSecurityHeaders: Record<string, string> = {
  "X-Content-Type-Options": "nosniff",
  "X-Frame-Options": "DENY",
  "Referrer-Policy": "strict-origin-when-cross-origin",
  "Permissions-Policy": "camera=(), microphone=(), geolocation=(), interest-cohort=()",
  "Cross-Origin-Opener-Policy": "same-origin",
};
