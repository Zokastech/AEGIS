// AEGIS — zokastech.fr — Apache 2.0 / MIT

import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  LineChart,
  Line,
  PieChart,
  Pie,
  Cell,
  Legend,
} from "recharts";
import type { LucideIcon } from "lucide-react";
import { Activity, ShieldCheck, Timer, Radio, AlertTriangle, Layers } from "lucide-react";
import { Trans, useTranslation } from "react-i18next";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useGatewayMetrics, fpRatePercent, meanAnalyzeLatencyMs } from "@/hooks/useGatewayMetrics";
import { isApiBaseConfigured } from "@/lib/api";
import { alertsFromMetrics, entitiesToChartData, topEntitiesForDonut } from "@/lib/prometheus";
import { grafanaGatewayDashboardHref, prometheusTargetsHref, prometheusUiHref } from "@/lib/observabilityUi";
import { cn } from "@/lib/utils";

/** Palette graphiques Zokastech (orange / rose / bleu marque). */
const ZK = {
  primary: "#ff8a00",
  tech: "#4361ee",
  warm: "#e52e71",
  success: "#2f855a",
  muted: "#64748b",
  grid: "rgba(67, 97, 238, 0.12)",
  donut: ["#ff8a00", "#e52e71", "#4361ee", "#2f855a", "#94a3b8"],
};

/** Recharts tooltips readable on light background (contrast, light shadow). */
const chartTooltipStyle: React.CSSProperties = {
  backgroundColor: "hsl(0 0% 100%)",
  border: "1px solid hsl(214 32% 91%)",
  borderRadius: 8,
  color: "hsl(220 16% 12%)",
  fontSize: 12,
  boxShadow: "0 4px 14px rgba(26, 29, 36, 0.08)",
};

function KpiCard({
  icon: Icon,
  labelId,
  description,
  titleChildren,
  footer,
  accentClass,
  titleClassName = "text-3xl",
  className,
}: {
  icon: LucideIcon;
  labelId: string;
  description: string;
  titleChildren: React.ReactNode;
  footer?: React.ReactNode;
  accentClass?: string;
  titleClassName?: string;
  className?: string;
}) {
  return (
    <Card className={cn("border-border/80 bg-card/90 shadow-sm transition-shadow duration-200 hover:shadow-md", className)}>
      <CardHeader className="space-y-3 pb-2">
        <div className="flex items-start justify-between gap-2">
          <CardDescription className="text-xs font-medium uppercase tracking-wide text-muted-foreground" id={labelId}>
            {description}
          </CardDescription>
          <span
            className={cn(
              "flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-border/80 bg-muted/40 text-muted-foreground",
              accentClass
            )}
            aria-hidden
          >
            <Icon className="h-4 w-4" />
          </span>
        </div>
        <CardTitle
          className={cn("font-display font-bold tabular-nums leading-none tracking-tight", titleClassName)}
          aria-labelledby={labelId}
        >
          {titleChildren}
        </CardTitle>
        {footer ? <div className="pt-0.5 text-xs leading-snug text-muted-foreground">{footer}</div> : null}
      </CardHeader>
    </Card>
  );
}

function localeForI18n(lng: string) {
  const b = lng.split("-")[0]?.toLowerCase() ?? "en";
  if (b === "fr") return "fr-FR";
  if (b === "de") return "de-DE";
  if (b === "es") return "es-ES";
  if (b === "it") return "it-IT";
  return "en-US";
}

export function DashboardPage() {
  const { t, i18n } = useTranslation("common");
  const loc = localeForI18n(i18n.language);
  const { data, isError, error, isFetching, trend } = useGatewayMetrics(15_000);

  const types = data ? entitiesToChartData(data.entitiesByType) : [];
  const donut = data && data.entitiesByType.size > 0 ? topEntitiesForDonut(data.entitiesByType, ZK.donut, 5) : [];
  const alerts = data ? alertsFromMetrics(data) : [];
  const latMs = data ? meanAnalyzeLatencyMs(data) : null;
  const fpPct = data ? fpRatePercent(data) : null;

  const activity = trend.length > 0 ? trend : [];

  const promUi = prometheusUiHref();
  const promTargets = prometheusTargetsHref();
  const grafanaDash = grafanaGatewayDashboardHref();
  const showObservabilityLinks = Boolean(promUi || grafanaDash);

  return (
    <div className="space-y-8">
      <header className="flex flex-col gap-3 border-b border-border/60 pb-6 sm:flex-row sm:items-end sm:justify-between">
        <div className="space-y-2">
          <p className="text-[11px] font-semibold uppercase tracking-[0.12em] text-muted-foreground">
            {t("dashboard.overviewEyebrow")}
          </p>
          <h1 className="font-display text-3xl font-bold tracking-tight text-foreground md:text-4xl">{t("dashboard.title")}</h1>
          <p className="max-w-2xl text-sm leading-relaxed text-muted-foreground">
            {t("dashboard.subtitlePrefix")}{" "}
            <span className="font-mono text-sm text-tech">{t("dashboard.subtitleMetricsPrometheus")}</span>{" "}
            {t("dashboard.subtitleMid")}{" "}
            <span className="font-semibold text-brand-pink">Zokastech</span>
            <span className="text-brand-orange"> · </span>
            <span className="bg-gradient-to-r from-brand-orange to-brand-blue bg-clip-text font-semibold text-transparent">
              AEGIS
            </span>
          </p>
          {showObservabilityLinks ? (
            <p className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
              <span className="font-medium text-foreground/80">{t("dashboard.observabilityLinksIntro")}</span>
              {promUi ? (
                <a
                  href={promUi}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="font-mono text-tech underline-offset-2 hover:underline"
                >
                  {t("dashboard.linkPrometheusUi")}
                </a>
              ) : null}
              {promTargets ? (
                <a
                  href={promTargets}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="font-mono text-tech/90 underline-offset-2 hover:underline"
                >
                  {t("dashboard.linkPrometheusTargets")}
                </a>
              ) : null}
              {grafanaDash ? (
                <a
                  href={grafanaDash}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="font-mono text-tech underline-offset-2 hover:underline"
                >
                  {t("dashboard.linkGrafanaGateway")}
                </a>
              ) : null}
            </p>
          ) : null}
        </div>
        {isFetching && !isError ? (
          <output className="text-xs font-medium text-muted-foreground" aria-live="polite">
            {t("dashboard.refreshing")}
          </output>
        ) : null}
      </header>

      {import.meta.env.PROD && !isApiBaseConfigured() ? (
        <div
          className="rounded-xl border border-amber-500/40 bg-amber-500/10 px-4 py-3 text-sm leading-relaxed text-foreground shadow-sm"
          role="status"
        >
          {t("dashboard.metricsProdBaseMissing")}
        </div>
      ) : null}

      {isError ? (
        <div
          className="rounded-xl border border-destructive/30 bg-destructive/5 px-4 py-4 text-sm text-foreground shadow-sm"
          role="alert"
        >
          <strong className="font-semibold text-destructive">{t("dashboard.metricsUnavailable")}</strong>{" "}
          {error instanceof Error ? error.message : String(error)} — {t("dashboard.metricsCheckEnv")}{" "}
          <code className="rounded-md bg-muted px-1.5 py-0.5 font-mono text-xs">VITE_AEGIS_API_URL</code> /{" "}
          <code className="rounded-md bg-muted px-1.5 py-0.5 font-mono text-xs">VITE_API_BASE</code> {t("dashboard.metricsAndGateway")}{" "}
          <code className="rounded-md bg-muted px-1.5 py-0.5 font-mono text-xs">GET /metrics</code>.
        </div>
      ) : null}

      <section aria-labelledby="dash-kpi-primary-heading" className="space-y-4">
        <h2 id="dash-kpi-primary-heading" className="sr-only">
          {t("dashboard.sectionPrimaryKpis")}
        </h2>
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <KpiCard
            icon={Activity}
            labelId="kpi-analyze-label"
            description={t("dashboard.kpiHttpAnalyze")}
            accentClass="text-primary border-primary/20 bg-primary/5"
            titleChildren={
              data ? (
                <span className="text-primary">
                  {Math.round(data.httpAnalyzeCount + data.httpAnalyzeBatchCount).toLocaleString(loc)}
                </span>
              ) : (
                "—"
              )
            }
            footer={
              data
                ? t("dashboard.kpiHttpAnalyzeHint", {
                    simple: Math.round(data.httpAnalyzeCount).toLocaleString(loc),
                    batch: Math.round(data.httpAnalyzeBatchCount).toLocaleString(loc),
                  })
                : undefined
            }
          />
          <KpiCard
            icon={ShieldCheck}
            labelId="kpi-anon-label"
            description={t("dashboard.kpiAnonymize")}
            accentClass="text-tech border-tech/20 bg-tech/5"
            titleChildren={
              data ? <span className="text-tech">{Math.round(data.httpAnonymizeCount).toLocaleString(loc)}</span> : "—"
            }
            footer={
              data
                ? t("dashboard.kpiAnonymizeHint", {
                    count: Math.round(data.anonymizeOpsTotal).toLocaleString(loc),
                  })
                : undefined
            }
          />
          <KpiCard
            icon={Timer}
            labelId="kpi-latency-label"
            description={t("dashboard.kpiLatency")}
            titleChildren={
              latMs !== null ? (
                <>
                  <span className="text-primary">{latMs.toFixed(0)}</span>
                  <span className="text-lg font-normal text-muted-foreground"> ms</span>
                </>
              ) : (
                "—"
              )
            }
          />
          <KpiCard
            icon={Radio}
            labelId="kpi-conn-label"
            description={t("dashboard.kpiConnections")}
            titleChildren={data ? Math.round(data.activeConnections).toLocaleString(loc) : "—"}
          />
        </div>
      </section>

      <section aria-labelledby="dash-kpi-secondary-heading" className="space-y-4">
        <h2 id="dash-kpi-secondary-heading" className="sr-only">
          {t("dashboard.sectionSecondaryKpis")}
        </h2>
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <KpiCard
            icon={AlertTriangle}
            labelId="kpi-fp-label"
            description={t("dashboard.kpiFalsePositiveRate")}
            titleClassName="text-2xl text-warm"
            accentClass="text-warm border-warm/25 bg-warm/5"
            titleChildren={fpPct !== null ? `${fpPct.toFixed(2)} %` : "—"}
          />
          <KpiCard
            icon={Layers}
            labelId="kpi-entities-label"
            description={t("dashboard.kpiEntitiesTotal")}
            titleClassName="text-2xl text-success"
            accentClass="text-success border-success/25 bg-success-muted"
            className="sm:col-span-1 lg:col-span-2"
            titleChildren={
              data
                ? [...data.entitiesByType.values()]
                    .reduce((s, v) => s + v, 0)
                    .toLocaleString(loc, { maximumFractionDigits: 0 })
                : "—"
            }
          />
        </div>
      </section>

      <div className="grid gap-6 lg:grid-cols-2">
        <Card className="border-border/80 bg-card/90 shadow-sm">
          <CardHeader>
            <CardTitle className="font-display text-lg">{t("dashboard.chartEntitiesByType")}</CardTitle>
            <CardDescription>
              <Trans i18nKey="dashboard.chartEntitiesByTypeDesc" components={{ mono: <span className="font-mono" /> }} />
            </CardDescription>
          </CardHeader>
          <CardContent className="h-72">
            {types.length === 0 ? (
              <p className="flex h-full items-center justify-center text-sm text-muted-foreground">
                {t("dashboard.chartEmpty")}
              </p>
            ) : (
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={types}>
                  <CartesianGrid strokeDasharray="3 3" stroke={ZK.grid} />
                  <XAxis dataKey="name" tick={{ fill: ZK.muted, fontSize: 11 }} />
                  <YAxis tick={{ fill: ZK.muted, fontSize: 11 }} />
                  <Tooltip contentStyle={chartTooltipStyle} />
                  <Bar dataKey="count" fill={ZK.primary} radius={[4, 4, 0, 0]} name={t("dashboard.chartBarSeries")} />
                </BarChart>
              </ResponsiveContainer>
            )}
          </CardContent>
        </Card>

        <Card className="border-border/80 bg-card/90 shadow-sm">
          <CardHeader>
            <CardTitle className="font-display text-lg">{t("dashboard.chartDeltaTitle")}</CardTitle>
            <CardDescription>{t("dashboard.chartDeltaDesc")}</CardDescription>
          </CardHeader>
          <CardContent className="h-72">
            {activity.length === 0 ? (
              <p className="flex h-full items-center justify-center text-sm text-muted-foreground">
                {t("dashboard.chartDeltaEmpty")}
              </p>
            ) : (
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={activity}>
                  <CartesianGrid strokeDasharray="3 3" stroke={ZK.grid} />
                  <XAxis dataKey="label" tick={{ fill: ZK.muted, fontSize: 9 }} interval="preserveStartEnd" />
                  <YAxis tick={{ fill: ZK.muted, fontSize: 11 }} allowDecimals={false} />
                  <Tooltip contentStyle={chartTooltipStyle} />
                  <Legend wrapperStyle={{ paddingTop: 8 }} className="text-xs text-muted-foreground" />
                  <Line
                    type="monotone"
                    dataKey="scans"
                    name={t("dashboard.chartDeltaScans")}
                    stroke={ZK.primary}
                    dot={false}
                    strokeWidth={2}
                  />
                  <Line
                    type="monotone"
                    dataKey="anonymized"
                    name={t("dashboard.chartDeltaAnon")}
                    stroke={ZK.tech}
                    dot={false}
                    strokeWidth={2}
                  />
                </LineChart>
              </ResponsiveContainer>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card className="border-border/80 bg-card/90 shadow-sm">
          <CardHeader>
            <CardTitle className="font-display text-lg">{t("dashboard.donutTitle")}</CardTitle>
            <CardDescription>{t("dashboard.donutDesc")}</CardDescription>
          </CardHeader>
          <CardContent className="h-64">
            {donut.length === 0 ? (
              <p className="flex h-full items-center justify-center text-sm text-muted-foreground">
                {t("dashboard.donutEmpty")}
              </p>
            ) : (
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie data={donut} dataKey="value" nameKey="name" cx="50%" cy="50%" innerRadius={50} outerRadius={80} paddingAngle={2}>
                    {donut.map((e, i) => (
                      <Cell key={i} fill={e.fill} />
                    ))}
                  </Pie>
                  <Tooltip contentStyle={chartTooltipStyle} />
                  <Legend wrapperStyle={{ paddingTop: 8 }} className="text-xs text-muted-foreground" />
                </PieChart>
              </ResponsiveContainer>
            )}
          </CardContent>
        </Card>

        <Card className="border-border/80 bg-card/90 shadow-sm">
          <CardHeader>
            <CardTitle className="font-display text-lg">{t("dashboard.alertsTitle")}</CardTitle>
            <CardDescription>{t("dashboard.alertsDesc")}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {alerts.length === 0 ? (
              <p className="text-sm text-muted-foreground">{t("dashboard.alertsNone")}</p>
            ) : (
              alerts.map((a) => (
                <div
                  key={a.id}
                  className="flex items-start justify-between gap-3 rounded-lg border border-border/80 bg-muted/30 p-4 transition-colors hover:bg-muted/40"
                >
                  <div>
                    <p className="text-sm font-medium">{t(a.titleKey, a.titleParams)}</p>
                    <p className="text-xs text-muted-foreground">
                      {new Date(a.at).toLocaleTimeString(loc, { hour: "2-digit", minute: "2-digit" })}
                    </p>
                  </div>
                  <Badge
                    variant={
                      a.severity === "high" ? "destructive" : a.severity === "medium" ? "default" : "secondary"
                    }
                    className={a.severity === "medium" ? "border-warm/50 bg-warm/15 text-warm" : undefined}
                  >
                    {t(`dashboard.severity.${a.severity}`)}
                  </Badge>
                </div>
              ))
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
