// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useLayoutEffect, useRef, useState, useCallback, useId, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import type { AnonymizationLink } from "./types";
import { highlightClassForEntityType } from "./entityPalette";

export type AnonymizationPreviewProps = {
  originalText: string;
  anonymizedText: string;
  /** Pairs of [start,end) ranges in each string; same `id` links matching segments. */
  links: AnonymizationLink[];
  className?: string;
  /** Minimum height of both panels. */
  minHeight?: number;
};

type LineSeg = { x1: number; y1: number; x2: number; y2: number; id: string };

function renderSegmented(
  text: string,
  links: AnonymizationLink[],
  side: "orig" | "anon",
  keyPrefix: string
): ReactNode[] {
  const ranges = links
    .map((l) => {
      const start = side === "orig" ? l.originalStart : l.anonymizedStart;
      const end = side === "orig" ? l.originalEnd : l.anonymizedEnd;
      return { id: l.id, entityType: l.entityType ?? "DEFAULT", start, end };
    })
    .filter((r) => r.start >= 0 && r.end <= text.length && r.start < r.end)
    .sort((a, b) => a.start - b.start);

  const nodes: ReactNode[] = [];
  let c = 0;
  ranges.forEach((r, i) => {
    if (r.start > c) nodes.push(<span key={`${keyPrefix}-t-${i}`}>{text.slice(c, r.start)}</span>);
    const cls = highlightClassForEntityType(r.entityType);
    nodes.push(
      <span
        key={`${keyPrefix}-e-${r.id}`}
        data-aegis-side={side}
        data-aegis-entity={r.id}
        className={cn("rounded-sm font-mono text-inherit leading-[inherit] [font-variant-ligatures:none]", cls)}
      >
        {text.slice(r.start, r.end)}
      </span>
    );
    c = r.end;
  });
  if (c < text.length) nodes.push(<span key={`${keyPrefix}-end`}>{text.slice(c)}</span>);
  return nodes.length > 0 ? nodes : [text];
}

/**
 * Split view: source / anonymized text with SVG connectors between linked segments.
 */
export function AnonymizationPreview({
  originalText,
  anonymizedText,
  links,
  className,
  minHeight = 140,
}: AnonymizationPreviewProps) {
  const { t } = useTranslation("common");
  const gradId = useId().replace(/:/g, "");
  const wrapRef = useRef<HTMLDivElement>(null);
  const [lines, setLines] = useState<LineSeg[]>([]);

  const recompute = useCallback(() => {
    const root = wrapRef.current;
    if (root == null || links.length === 0) {
      setLines([]);
      return;
    }
    const rootRect = root.getBoundingClientRect();
    const next: LineSeg[] = [];
    for (const link of links) {
      const esc =
        typeof CSS !== "undefined" && typeof CSS.escape === "function"
          ? CSS.escape(link.id)
          : link.id.replace(/"/g, '\\"');
      const left = root.querySelector(`[data-aegis-side="orig"][data-aegis-entity="${esc}"]`);
      const right = root.querySelector(`[data-aegis-side="anon"][data-aegis-entity="${esc}"]`);
      if (!(left instanceof HTMLElement) || !(right instanceof HTMLElement)) continue;
      const a = left.getBoundingClientRect();
      const b = right.getBoundingClientRect();
      next.push({
        id: link.id,
        x1: a.right - rootRect.left,
        y1: a.top + a.height / 2 - rootRect.top,
        x2: b.left - rootRect.left,
        y2: b.top + b.height / 2 - rootRect.top,
      });
    }
    setLines(next);
  }, [links]);

  useLayoutEffect(() => {
    recompute();
    const ro = new ResizeObserver(() => recompute());
    const el = wrapRef.current;
    if (el) ro.observe(el);
    window.addEventListener("scroll", recompute, true);
    return () => {
      ro.disconnect();
      window.removeEventListener("scroll", recompute, true);
    };
  }, [recompute, originalText, anonymizedText, links]);

  const leftNodes = renderSegmented(originalText, links, "orig", "o");
  const rightNodes = renderSegmented(anonymizedText, links, "anon", "a");

  return (
    <div ref={wrapRef} className={cn("relative rounded-lg border border-border bg-card/30", className)}>
      <div className="grid gap-0 md:grid-cols-2">
        <div className="border-b border-border p-3 md:border-b-0 md:border-r">
          <div className="mb-2 text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
            {t("playground.anonPreview.original")}
          </div>
          <div
            className="overflow-auto rounded-md border border-border/60 bg-muted/10 p-3 font-mono text-sm leading-normal [font-variant-ligatures:none]"
            style={{ minHeight }}
          >
            {leftNodes}
          </div>
        </div>
        <div className="p-3">
          <div className="mb-2 text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
            {t("playground.anonPreview.anonymized")}
          </div>
          <div
            className="overflow-auto rounded-md border border-border/60 bg-muted/10 p-3 font-mono text-sm leading-normal [font-variant-ligatures:none]"
            style={{ minHeight }}
          >
            {rightNodes}
          </div>
        </div>
      </div>
      {lines.length > 0 ? (
        <svg
          className="pointer-events-none absolute inset-0 z-10 h-full w-full overflow-visible"
          aria-hidden
        >
          <defs>
            <linearGradient id={gradId} x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" stopColor="hsl(199 89% 48% / 0.5)" />
              <stop offset="100%" stopColor="hsl(199 89% 48% / 0.15)" />
            </linearGradient>
          </defs>
          {lines.map((ln) => {
            const mx = (ln.x1 + ln.x2) / 2;
            const d = `M ${ln.x1} ${ln.y1} C ${mx} ${ln.y1}, ${mx} ${ln.y2}, ${ln.x2} ${ln.y2}`;
            return (
              <path
                key={ln.id}
                d={d}
                fill="none"
                stroke={`url(#${gradId})`}
                strokeWidth={1.25}
                vectorEffect="non-scaling-stroke"
              />
            );
          })}
        </svg>
      ) : null}
    </div>
  );
}
