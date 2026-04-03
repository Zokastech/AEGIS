// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useMemo, useId } from "react";
import { useTranslation } from "react-i18next";
import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip as RechartsTooltip,
  XAxis,
  YAxis,
} from "recharts";
import { cn } from "@/lib/utils";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";

export type ConfidenceSliderProps = {
  /** Current threshold ∈ [min, max]. */
  value: number;
  onChange: (threshold: number) => void;
  /** Entities whose score is compared to the threshold. */
  entities: Array<{ score: number }>;
  min?: number;
  max?: number;
  step?: number;
  /** Number of samples along the threshold axis for the curve. */
  chartSamples?: number;
  className?: string;
};

type ChartRow = { t: number; detected: number; ignored: number };

function buildCurve(
  entities: Array<{ score: number }>,
  min: number,
  max: number,
  samples: number
): ChartRow[] {
  const n = Math.max(2, samples);
  const rows: ChartRow[] = [];
  for (let i = 0; i < n; i++) {
    const t = min + ((max - min) * i) / (n - 1);
    const detected = entities.filter((e) => e.score >= t).length;
    rows.push({ t, detected, ignored: entities.length - detected });
  }
  return rows;
}

function countsAt(entities: Array<{ score: number }>, threshold: number) {
  const detected = entities.filter((e) => e.score >= threshold).length;
  return { detected, ignored: entities.length - detected };
}

/**
 * Confidence threshold slider with live chart: entities kept vs filtered by threshold.
 */
export function ConfidenceSlider({
  value,
  onChange,
  entities,
  min = 0.5,
  max = 0.99,
  step = 0.01,
  chartSamples = 24,
  className,
}: ConfidenceSliderProps) {
  const { t } = useTranslation("common");
  const gid = useId().replace(/:/g, "");
  const data = useMemo(() => buildCurve(entities, min, max, chartSamples), [entities, min, max, chartSamples]);
  const { detected, ignored } = useMemo(() => countsAt(entities, value), [entities, value]);
  const sliderValue = [Math.min(max, Math.max(min, value))];

  return (
    <div className={cn("space-y-4 rounded-lg border border-border bg-muted/25 p-4", className)}>
      <div className="flex flex-wrap items-end justify-between gap-2">
        <div>
          <Label className="text-sm font-semibold text-foreground">{t("playground.confidence.label")}</Label>
          <p className="mt-1 text-sm font-normal leading-snug text-foreground/82">{t("playground.confidence.hint")}</p>
        </div>
        <div className="text-right text-sm">
          <div className="leading-snug">
            <span className="font-medium text-foreground/75">{t("playground.confidence.at")} </span>
            <span className="font-mono font-semibold text-primary">{value.toFixed(2)}</span>
            <span className="font-medium text-foreground/75"> : </span>
            <span className="font-semibold text-success">{detected}</span>
            <span className="font-medium text-foreground/75"> {t("playground.confidence.detected")} · </span>
            <span className="font-semibold text-rose-700">{ignored}</span>
            <span className="font-medium text-foreground/75"> {t("playground.confidence.ignored")}</span>
          </div>
        </div>
      </div>
      <Slider
        value={sliderValue}
        onValueChange={(v) => onChange(v[0] ?? min)}
        min={min}
        max={max}
        step={step}
        className="py-1"
      />
      <div className="h-36 w-full">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data} margin={{ top: 4, right: 8, left: -18, bottom: 0 }}>
            <defs>
              <linearGradient id={`aegis-detected-${gid}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="hsl(142 76% 45%)" stopOpacity={0.35} />
                <stop offset="100%" stopColor="hsl(142 76% 45%)" stopOpacity={0} />
              </linearGradient>
              <linearGradient id={`aegis-ignored-${gid}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="hsl(350 80% 60%)" stopOpacity={0.3} />
                <stop offset="100%" stopColor="hsl(350 80% 60%)" stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="hsl(217 33% 22%)" />
            <XAxis
              dataKey="t"
              type="number"
              domain={[min, max]}
              tickFormatter={(x: number) => x.toFixed(2)}
              stroke="hsl(215 20% 55%)"
              fontSize={10}
              tickLine={false}
            />
            <YAxis allowDecimals={false} stroke="hsl(215 20% 55%)" fontSize={10} tickLine={false} width={28} />
            <RechartsTooltip
              contentStyle={{
                background: "hsl(222 40% 10%)",
                border: "1px solid hsl(217 33% 22%)",
                borderRadius: 8,
                fontSize: 11,
              }}
              labelFormatter={(x) => t("playground.confidence.thresholdAt", { value: Number(x).toFixed(2) })}
              formatter={(val: number, name: string) => [
                val,
                name === "detected" ? t("playground.confidence.chartDetected") : t("playground.confidence.chartIgnored"),
              ]}
            />
            <Area
              type="monotone"
              dataKey="detected"
              stroke="hsl(142 76% 45%)"
              fill={`url(#aegis-detected-${gid})`}
              strokeWidth={1.5}
            />
            <Area
              type="monotone"
              dataKey="ignored"
              stroke="hsl(350 80% 60%)"
              fill={`url(#aegis-ignored-${gid})`}
              strokeWidth={1.5}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
