// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { Trans, useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import {
  TextHighlighter,
  EntitySidebar,
  AnonymizationPreview,
  PipelineLevelSelector,
  ConfidenceSlider,
  AnonymizationOperatorSelect,
  PlaygroundConfigPreview,
} from "@/components/playground";
import { usePlaygroundPage } from "@/hooks/usePlaygroundPage";

export function PlaygroundPage() {
  const { t } = useTranslation("common");
  const {
    policiesQuery,
    text,
    lang,
    pipeline,
    threshold,
    policy,
    anonOperator,
    entities,
    anonymizedBlock,
    anonLinks,
    falsePositiveIds,
    analyzeError,
    anonymError,
    gatewayUsesMockEngine,
    analyzeMut,
    anonymMut,
    setLang,
    setPipeline,
    setThreshold,
    setPolicy,
    setAnonOperator,
    onTextChange,
    onFalsePositiveChange,
    scoreSamples,
    policyNames,
    configJson,
    anonymizeConfigJson,
    insertSampleText,
  } = usePlaygroundPage();

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">{t("playground.title")}</h1>
        <p className="text-muted-foreground">
          <Trans i18nKey="playground.subtitle" components={{ mono: <span className="font-mono" /> }} />
        </p>
      </div>

      <div className="grid gap-4 lg:grid-cols-3">
        <Card className="lg:col-span-1">
          <CardHeader>
            <CardTitle>{t("playground.paramsTitle")}</CardTitle>
            <CardDescription className="text-sm font-medium leading-relaxed text-foreground/80">
              {t("playground.paramsDesc")}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-5">
            <div className="space-y-2">
              <Label className="text-foreground font-semibold">{t("playground.language")}</Label>
              <Select value={lang} onValueChange={setLang}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="fr">{t("language.fr")}</SelectItem>
                  <SelectItem value="en">{t("language.en")}</SelectItem>
                  <SelectItem value="de">{t("language.de")}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <PipelineLevelSelector value={pipeline} onChange={setPipeline} />
            <ConfidenceSlider value={threshold} onChange={setThreshold} entities={scoreSamples} min={0.35} />
            <AnonymizationOperatorSelect value={anonOperator} onChange={setAnonOperator} />
            <div className="space-y-2">
              <Label className="text-foreground font-semibold">{t("playground.policyOptional")}</Label>
              <Select value={policy || "_"} onValueChange={(v) => setPolicy(v === "_" ? "" : v)}>
                <SelectTrigger>
                  <SelectValue placeholder={t("playground.policyNone")} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="_">{t("playground.policyNone")}</SelectItem>
                  {policyNames.map((n) => (
                    <SelectItem key={n} value={n}>
                      {n}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {policiesQuery.error ? (
                <p className="text-xs text-warm">
                  {t("playground.policiesUnavailable", { message: String(policiesQuery.error) })}
                </p>
              ) : null}
            </div>
            {gatewayUsesMockEngine ? (
              <div
                role="status"
                className="rounded-md border border-amber-500/45 bg-amber-500/10 px-3 py-2 text-xs leading-relaxed text-amber-950 dark:text-amber-50"
              >
                <Trans i18nKey="playground.mockEngineWarning" components={{ mono: <span className="font-mono text-[11px]" /> }} />
              </div>
            ) : null}
            {analyzeError ? <p className="text-xs text-red-400">{analyzeError}</p> : null}
            {anonymError ? <p className="text-xs text-red-400">{anonymError}</p> : null}
            <PlaygroundConfigPreview analysisConfigJson={configJson} anonymizeConfigJson={anonymizeConfigJson} />
            <div className="flex flex-wrap gap-2">
              <Button onClick={() => analyzeMut.mutate()} disabled={analyzeMut.isPending}>
                {t("playground.analyze")}
              </Button>
              <Button variant="secondary" onClick={() => anonymMut.mutate()} disabled={anonymMut.isPending}>
                {t("playground.anonymize")}
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>{t("playground.textTitle")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex flex-wrap items-center gap-2">
              <Button type="button" variant="outline" size="sm" onClick={insertSampleText}>
                {t("playground.insertSample")}
              </Button>
              <span className="text-xs text-muted-foreground">{t("playground.insertSampleHint")}</span>
            </div>
            <Textarea
              value={text}
              onChange={(e) => onTextChange(e.target.value)}
              placeholder={t("playground.textPlaceholder")}
              className="min-h-[140px] font-mono text-sm"
            />
            <div>
              <Label className="mb-2 block text-muted-foreground">{t("playground.highlightLabel")}</Label>
              <div className="min-h-[100px] rounded-md border border-border bg-muted/20 p-3">
                {analyzeMut.isPending ? (
                  <span className="font-mono text-sm text-muted-foreground">{t("playground.analyzing")}</span>
                ) : (
                  <TextHighlighter text={text} entities={entities} />
                )}
              </div>
            </div>
            {anonLinks != null && anonymizedBlock ? (
              <div>
                <Label className="mb-2 block text-muted-foreground">{t("playground.compareLabel")}</Label>
                <AnonymizationPreview originalText={text} anonymizedText={anonymizedBlock} links={anonLinks} minHeight={100} />
              </div>
            ) : (
              <div>
                <Label className="mb-2 block text-muted-foreground">{t("playground.anonTextLabel")}</Label>
                <pre className="max-h-48 overflow-auto rounded-md border border-border bg-muted/20 p-3 font-mono text-sm whitespace-pre-wrap">
                  {anonymizedBlock || "—"}
                </pre>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        {entities.length > 0 ? (
          <EntitySidebar
            entities={entities}
            falsePositiveIds={falsePositiveIds}
            onFalsePositiveChange={onFalsePositiveChange}
            className="lg:max-h-none"
          />
        ) : null}
        {entities.length > 0 ? (
          <Card>
            <CardHeader>
              <CardTitle>{t("playground.decisionTraceTitle")}</CardTitle>
              <CardDescription>{t("playground.decisionTraceDesc")}</CardDescription>
            </CardHeader>
            <CardContent className="max-h-[min(70vh,560px)] space-y-4 overflow-y-auto">
              {entities.map((e) => {
                const d = e.decisionTrace;
                return (
                  <div key={e.id} className="rounded-lg border border-border p-4">
                    <div className="mb-2 flex flex-wrap items-center gap-2">
                      <Badge>{e.entityType}</Badge>
                      <span className="text-xs text-muted-foreground">
                        [{e.start}:{e.end}] {t("playground.score")} {e.score.toFixed(2)}
                      </span>
                      {d?.pipelineLevel != null ? <Badge variant="outline">L{d.pipelineLevel}</Badge> : null}
                    </div>
                    {d != null && d.steps.length > 0 ? (
                      <ul className="text-sm text-muted-foreground">
                        {d.steps.map((s) => (
                          <li key={`${e.id}-${s.name}`}>
                            {s.name} → {s.passed ? t("playground.decisionTraceStepOk") : t("playground.decisionTraceStepReject")}
                            {s.detail != null && s.detail !== "" ? (
                              <span className="block pl-2 text-xs opacity-80">{s.detail}</span>
                            ) : null}
                          </li>
                        ))}
                      </ul>
                    ) : (
                      <p className="text-sm text-muted-foreground">{t("playground.decisionTraceEmpty")}</p>
                    )}
                  </div>
                );
              })}
            </CardContent>
          </Card>
        ) : null}
      </div>
    </div>
  );
}
