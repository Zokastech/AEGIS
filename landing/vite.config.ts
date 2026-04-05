// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { landingSecurityHeaders } from "./src/lib/securityHeaders";

export default defineConfig({
  plugins: [react()],
  server: {
    headers: landingSecurityHeaders,
  },
  preview: {
    headers: landingSecurityHeaders,
  },
  build: {
    cssMinify: true,
    minify: "esbuild",
    rollupOptions: {
      output: {
        manualChunks: undefined,
      },
    },
  },
});
