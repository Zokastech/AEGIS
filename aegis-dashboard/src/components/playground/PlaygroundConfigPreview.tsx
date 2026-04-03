// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

function prettyJson(raw: string): string {
  const t = raw.trim();
  if (!t) return "";
  try {
    return JSON.stringify(JSON.parse(t), null, 2);
  } catch {
    return raw;
  }
}

export type PlaygroundConfigPreviewProps = {
  analysisConfigJson: string;
  anonymizeConfigJson: string;
  className?: string;
};

export function PlaygroundConfigPreview({ analysisConfigJson, anonymizeConfigJson, className }: PlaygroundConfigPreviewProps) {
  const { t } = useTranslation("common");

  return (
    <details className={cn("rounded-lg border border-border bg-muted/20", className)}>
      <summary className="cursor-pointer select-none px-4 py-3 text-sm font-semibold text-foreground hover:bg-muted/40">
        {t("playground.configPreview.title")}
      </summary>
      <div className="space-y-4 border-t border-border px-4 pb-4 pt-3">
        <p className="text-xs leading-relaxed text-foreground/80">{t("playground.configPreview.intro")}</p>
        <div>
          <h3 className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            {t("playground.configPreview.analysisLabel")}
          </h3>
          <p className="mb-2 text-xs text-muted-foreground">{t("playground.configPreview.analysisHint")}</p>
          <pre className="max-h-48 overflow-auto rounded-md border border-border bg-card p-3 font-mono text-[11px] leading-relaxed text-foreground">
            {prettyJson(analysisConfigJson)}
          </pre>
        </div>
        <div>
          <h3 className="mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            {t("playground.configPreview.anonymizeLabel")}
          </h3>
          <p className="mb-2 text-xs text-muted-foreground">{t("playground.configPreview.anonymizeHint")}</p>
          <pre className="max-h-64 overflow-auto rounded-md border border-border bg-card p-3 font-mono text-[11px] leading-relaxed text-foreground">
            {prettyJson(anonymizeConfigJson)}
          </pre>
        </div>
      </div>
    </details>
  );
}
