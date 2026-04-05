// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { StrictMode, Suspense } from "react";
import { createRoot } from "react-dom/client";
import { RouterProvider } from "@tanstack/react-router";
import { AppProviders } from "@/providers/AppProviders";
import { router } from "./router";
import "@/i18n/config";
import "./index.css";

/** Texte neutre : le hook useTranslation n’est pas fiable ici (hors arbre i18n / chargement initial). */
function I18nLoadingFallback() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background text-sm text-zokastech-gray">
      Loading… / Chargement…
    </div>
  );
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Suspense fallback={<I18nLoadingFallback />}>
      <AppProviders>
        <RouterProvider router={router} />
      </AppProviders>
    </Suspense>
  </StrictMode>
);
