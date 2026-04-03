// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import { dashboardSecurityHeaders } from "./src/lib/securityHeaders";

/**
 * Cible du proxy `/v1` et `/metrics` (côté Node, pas le navigateur).
 * - Local : 127.0.0.1:8080 (gateway sur la machine hôte).
 * - Docker Compose : `http://aegis-gateway:8080` (voir docker-compose.dev.yml → AEGIS_GATEWAY_PROXY_TARGET).
 */
const gatewayProxyTarget = process.env.AEGIS_GATEWAY_PROXY_TARGET?.trim() || "http://127.0.0.1:8080";

const gatewayProxy = {
  "/v1": { target: gatewayProxyTarget, changeOrigin: true },
  "/metrics": { target: gatewayProxyTarget, changeOrigin: true },
} as const;

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 5173,
    headers: dashboardSecurityHeaders,
    proxy: { ...gatewayProxy },
  },
  preview: {
    headers: dashboardSecurityHeaders,
    proxy: { ...gatewayProxy },
  },
});
