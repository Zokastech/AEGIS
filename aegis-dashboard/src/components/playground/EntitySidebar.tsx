// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useMemo, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown, ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import type { PlaygroundEntity } from "./types";
import { highlightClassForEntityType } from "./entityPalette";

export type EntitySidebarProps = {
  entities: PlaygroundEntity[];
  /** IDs marked as false positives (feedback). */
  falsePositiveIds?: ReadonlySet<string>;
  /** Called when the user toggles false-positive status. */
  onFalsePositiveChange?: (entityId: string, isFalsePositive: boolean) => void;
  className?: string;
};

function groupByType(entities: PlaygroundEntity[]): Map<string, PlaygroundEntity[]> {
  const m = new Map<string, PlaygroundEntity[]>();
  for (const e of entities) {
    const list = m.get(e.entityType);
    if (list) list.push(e);
    else m.set(e.entityType, [e]);
  }
  for (const list of m.values()) {
    list.sort((a, b) => a.start - b.start);
  }
  return m;
}

/**
 * Side panel: entities grouped by type and score, with false-positive toggle for the feedback loop.
 */
export function EntitySidebar({
  entities,
  falsePositiveIds = new Set(),
  onFalsePositiveChange,
  className,
}: EntitySidebarProps) {
  const { t } = useTranslation("common");
  const grouped = useMemo(() => groupByType(entities), [entities]);
  const types = useMemo(() => [...grouped.keys()].sort((a, b) => a.localeCompare(b)), [grouped]);

  const [collapsed, setCollapsed] = useState<Set<string>>(() => new Set());

  const toggleType = useCallback((typeKey: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(typeKey)) next.delete(typeKey);
      else next.add(typeKey);
      return next;
    });
  }, []);

  if (entities.length === 0) {
    return (
      <div className={cn("rounded-lg border border-dashed border-border p-6 text-center text-sm text-muted-foreground", className)}>
        {t("playground.entitySidebar.none")}
      </div>
    );
  }

  return (
    <div className={cn("flex max-h-[min(70vh,560px)] flex-col gap-2 overflow-y-auto rounded-lg border border-border bg-card/40", className)}>
      <div className="sticky top-0 z-10 border-b border-border bg-card/90 px-3 py-2 backdrop-blur-sm">
        <h3 className="text-sm font-semibold text-foreground">{t("playground.entitySidebar.title")}</h3>
        <p className="text-xs text-muted-foreground">
          {t("playground.entitySidebar.occurrences", { count: entities.length, types: types.length })}
        </p>
      </div>
      <div className="space-y-1 p-2">
        {types.map((type) => {
          const list = grouped.get(type) ?? [];
          const isCollapsed = collapsed.has(type);
          const chipCls = highlightClassForEntityType(type);
          return (
            <div key={type} className="rounded-md border border-border/80 bg-background/50">
              <button
                type="button"
                onClick={() => toggleType(type)}
                className="flex w-full items-center gap-2 px-2 py-2 text-left text-sm hover:bg-muted/50"
              >
                {isCollapsed ? <ChevronRight className="h-4 w-4 shrink-0" /> : <ChevronDown className="h-4 w-4 shrink-0" />}
                <span className={cn("rounded border px-1.5 py-0.5 text-xs font-medium", chipCls)}>{type}</span>
                <Badge variant="secondary" className="ml-auto text-[10px]">
                  {list.length}
                </Badge>
              </button>
              {!isCollapsed ? (
                <ul className="space-y-2 border-t border-border/60 px-2 py-2">
                  {list.map((e) => {
                    const isFp = falsePositiveIds.has(e.id);
                    const preview =
                      e.text ?? t("playground.entitySidebar.offsetFallback", { start: e.start, end: e.end });
                    return (
                      <li
                        key={e.id}
                        className={cn(
                          "rounded-md border border-border/50 bg-muted/20 p-2 text-xs transition-opacity",
                          isFp && "opacity-50"
                        )}
                      >
                        <div className="flex items-start justify-between gap-2">
                          <div className="min-w-0 flex-1">
                            <div className="truncate font-mono text-[11px] text-foreground" title={preview}>
                              {preview}
                            </div>
                            <div className="mt-1 text-[10px] text-muted-foreground">
                              {t("playground.score")}{" "}
                              <span className="text-primary">{e.score.toFixed(3)}</span>
                              {e.recognizer != null ? (
                                <>
                                  {" "}
                                  · <span className="font-mono">{e.recognizer}</span>
                                </>
                              ) : null}
                            </div>
                          </div>
                          <div className="flex shrink-0 flex-col items-end gap-1">
                            <Label htmlFor={`fp-${e.id}`} className="text-[10px] text-muted-foreground">
                              {t("playground.entitySidebar.falsePositive")}
                            </Label>
                            <Switch
                              id={`fp-${e.id}`}
                              checked={isFp}
                              onCheckedChange={(v) => onFalsePositiveChange?.(e.id, v)}
                              disabled={onFalsePositiveChange == null}
                              aria-label={t("playground.entitySidebar.ariaFalsePositive", { type: e.entityType })}
                            />
                          </div>
                        </div>
                      </li>
                    );
                  })}
                </ul>
              ) : null}
            </div>
          );
        })}
      </div>
    </div>
  );
}
