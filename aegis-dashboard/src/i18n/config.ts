// AEGIS — zokastech.fr — Apache 2.0 / MIT
// i18next + react-i18next (see official docs: initReactI18next, Suspense, resourcesToBackend, LanguageDetector).

import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import resourcesToBackend from "i18next-resources-to-backend";

/** Languages aligned with project MkDocs (en, fr, de, es, it). */
export const supportedLanguages = ["en", "fr", "de", "es", "it"] as const;
export type SupportedLanguage = (typeof supportedLanguages)[number];

function syncDocumentLanguage(lng: string) {
  if (typeof document === "undefined") return;
  const base = lng.split("-")[0] ?? "en";
  document.documentElement.lang = base;
  document.documentElement.dir = i18n.dir(base);
}

void i18n
  .use(
    resourcesToBackend((language: string, namespace: string) =>
      import(`../locales/${language}/${namespace}.json`).then((m) => m.default as Record<string, unknown>)
    )
  )
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    fallbackLng: "en",
    supportedLngs: [...supportedLanguages],
    ns: ["common"],
    defaultNS: "common",
    interpolation: { escapeValue: false },
    react: {
      useSuspense: true,
      bindI18n: "languageChanged loaded",
      transSupportBasicHtmlNodes: true,
      transKeepBasicHtmlNodesFor: ["br", "strong", "i", "span", "mono"],
    },
    detection: {
      order: ["localStorage", "navigator", "htmlTag"],
      caches: ["localStorage"],
      lookupLocalStorage: "aegis-dashboard-i18nextLng",
    },
    debug: import.meta.env.DEV,
  })
  .then(() => {
    syncDocumentLanguage(i18n.resolvedLanguage || i18n.language);
  });

i18n.on("languageChanged", (lng) => {
  syncDocumentLanguage(lng);
});

export default i18n;
