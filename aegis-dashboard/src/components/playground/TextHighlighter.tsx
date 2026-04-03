// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useMemo, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import type { PlaygroundEntity } from "./types";
import { highlightClassForEntityType } from "./entityPalette";

export type TextHighlighterProps = {
  /** Raw source text. */
  text: string;
  /** Entities with [start, end) offsets valid in `text`. */
  entities: PlaygroundEntity[];
  className?: string;
  /** Tooltip open delay (ms). */
  tooltipDelayMs?: number;
  /** Override highlight classes per entity type. */
  typeClassNames?: Record<string, string>;
};

function sortAndClampEntities(text: string, entities: PlaygroundEntity[]): PlaygroundEntity[] {
  const len = text.length;
  return [...entities]
    .filter((e) => e.start >= 0 && e.end <= len && e.start < e.end)
    .sort((a, b) => a.start - b.start || b.end - a.end);
}

function EntityTooltipBody({ entity }: { entity: PlaygroundEntity }) {
  const { t } = useTranslation("common");
  const trace = entity.decisionTrace;
  return (
    <div className="space-y-2 text-left">
      <div className="font-semibold text-foreground">{entity.entityType}</div>
      <div className="grid gap-0.5 text-[11px] text-muted-foreground">
        <div>
          {t("playground.tooltip.score")}{" "}
          <span className="text-foreground">{entity.score.toFixed(3)}</span>
        </div>
        <div>
          {t("playground.tooltip.recognizer")}{" "}
          <span className="text-foreground">{entity.recognizer ?? "—"}</span>
        </div>
        {entity.text != null && entity.text !== "" ? (
          <div className="truncate" title={entity.text}>
            {t("playground.tooltip.value")}{" "}
            <span className="font-mono text-foreground">{entity.text}</span>
          </div>
        ) : null}
        <div>
          {t("playground.tooltip.offsets")}{" "}
          <span className="font-mono text-foreground">
            [{entity.start}:{entity.end}]
          </span>
        </div>
      </div>
      {trace != null && trace.steps.length > 0 ? (
        <div className="border-t border-border pt-2">
          <div className="mb-1 text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
            {t("playground.tooltip.decisionTrace")}
          </div>
          <ul className="space-y-1 text-[11px]">
            {trace.steps.map((s, i) => (
              <li key={i} className="flex flex-col gap-0.5">
                <span className={s.passed ? "text-emerald-400" : "text-rose-400"}>
                  {s.passed ? "✓" : "✗"} {s.name}
                </span>
                {s.detail != null && s.detail !== "" ? <span className="pl-3 text-muted-foreground">{s.detail}</span> : null}
              </li>
            ))}
          </ul>
          {trace.pipelineLevel != null ? (
            <div className="mt-1 text-[10px] text-muted-foreground">
              {t("playground.tooltip.pipeline")} L{trace.pipelineLevel}
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}

/**
 * Renders text with per-entity-type highlights and tooltip (score, recognizer, decision trace).
 */
export function TextHighlighter({
  text,
  entities,
  className,
  tooltipDelayMs = 200,
  typeClassNames,
}: TextHighlighterProps) {
  const sorted = useMemo(() => sortAndClampEntities(text, entities), [text, entities]);

  const nodes = useMemo(() => {
    const out: ReactNode[] = [];
    let cursor = 0;
    sorted.forEach((e, i) => {
      if (e.start > cursor) {
        out.push(<span key={`t-${i}-${cursor}`}>{text.slice(cursor, e.start)}</span>);
      }
      const cls = typeClassNames?.[e.entityType] ?? highlightClassForEntityType(e.entityType);
      const slice = text.slice(e.start, e.end);
      out.push(
        <Tooltip key={`e-${e.id}`} delayDuration={tooltipDelayMs}>
          <TooltipTrigger asChild>
            <mark
              className={cn(
                "cursor-help rounded border px-0.5 font-mono text-[0.95em] leading-relaxed transition-colors",
                cls
              )}
            >
              {slice}
            </mark>
          </TooltipTrigger>
          <TooltipContent side="top" className="max-w-sm">
            <EntityTooltipBody entity={e} />
          </TooltipContent>
        </Tooltip>
      );
      cursor = e.end;
    });
    if (cursor < text.length) {
      out.push(<span key="tail">{text.slice(cursor)}</span>);
    }
    return out.length > 0 ? out : text;
  }, [sorted, text, tooltipDelayMs, typeClassNames]);

  return (
    <TooltipProvider delayDuration={tooltipDelayMs}>
      <div className={cn("whitespace-pre-wrap break-words font-mono text-sm leading-relaxed", className)}>{nodes}</div>
    </TooltipProvider>
  );
}
