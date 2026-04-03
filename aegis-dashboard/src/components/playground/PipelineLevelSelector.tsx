// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";

export type PipelineLevel = 1 | 2 | 3;

const LEVELS: Array<{
  level: PipelineLevel;
  i18nKey: "l1" | "l2" | "l3";
  latencyMs: { min: number; max: number };
}> = [
  { level: 1, i18nKey: "l1", latencyMs: { min: 2, max: 15 } },
  { level: 2, i18nKey: "l2", latencyMs: { min: 15, max: 45 } },
  { level: 3, i18nKey: "l3", latencyMs: { min: 40, max: 180 } },
];

export type PipelineLevelSelectorProps = {
  value: PipelineLevel;
  onChange: (level: PipelineLevel) => void;
  className?: string;
  disabled?: boolean;
};

/**
 * Visual selector for the three pipeline levels with indicative latency hints for operators.
 */
export function PipelineLevelSelector({ value, onChange, className, disabled }: PipelineLevelSelectorProps) {
  const { t } = useTranslation("common");

  return (
    <div className={cn("space-y-3", className)}>
      <p className="text-sm font-medium leading-snug text-foreground/85">{t("playground.pipeline.hint")}</p>
      <div className="grid gap-3 sm:grid-cols-3">
        {LEVELS.map((L) => {
          const selected = value === L.level;
          const pk = `playground.pipeline.${L.i18nKey}` as const;
          const bullets = [t(`${pk}.b0`), t(`${pk}.b1`), t(`${pk}.b2`)];
          return (
            <button
              key={L.level}
              type="button"
              disabled={disabled}
              onClick={() => onChange(L.level)}
              aria-pressed={selected}
              className={cn(
                "relative flex min-h-0 flex-col overflow-hidden rounded-xl border p-4 text-left transition-colors duration-200",
                "outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 focus-visible:ring-offset-background",
                selected
                  ? "border-2 border-primary bg-primary/[0.09] shadow-sm"
                  : "border border-border bg-card hover:border-primary/35 hover:bg-muted/50",
                disabled && "pointer-events-none opacity-50"
              )}
            >
              <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
                <span className="text-lg font-bold tabular-nums text-primary">L{L.level}</span>
                <Badge
                  variant={selected ? "outline" : "secondary"}
                  className={cn(
                    "shrink-0 text-xs font-semibold tabular-nums",
                    selected &&
                      "border-primary/50 bg-orange-50 text-zokastech-dark dark:bg-primary/15 dark:text-foreground"
                  )}
                >
                  ~{L.latencyMs.min}–{L.latencyMs.max} ms
                </Badge>
              </div>
              <div className="text-sm font-semibold leading-snug text-foreground">{t(`${pk}.title`)}</div>
              <p className="mt-2 text-sm font-normal leading-relaxed text-foreground/88">{t(`${pk}.description`)}</p>
              <ul className="mt-3 space-y-2.5 border-t border-border/80 pt-3">
                {bullets.map((b, idx) => (
                  <li key={`${L.level}-b-${idx}`} className="flex gap-2.5 text-sm font-normal leading-relaxed text-foreground/82">
                    <span className="mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-primary" aria-hidden />
                    <span>{b}</span>
                  </li>
                ))}
              </ul>
            </button>
          );
        })}
      </div>
    </div>
  );
}
