// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useMemo, useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Trans, useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Textarea } from "@/components/ui/textarea";
import { analyzeText } from "@/lib/api";
import { recognizersListQueryOptions } from "@/queries/gateway-query-options";
import { useAuthCredential } from "@/hooks/useAuthCredential";

const DEFAULT_THRESHOLD = 0.82;

export function RecognizersPage() {
  const { t } = useTranslation("common");
  const credential = useAuthCredential();
  const { data, error, isLoading } = useQuery(recognizersListQueryOptions(credential));

  const base = data?.recognizers ?? [];
  const [overrides, setOverrides] = useState<Record<string, { enabled?: boolean; threshold?: number }>>({});

  const rows = useMemo(
    () =>
      base.map((r) => ({
        ...r,
        enabled: overrides[r.name]?.enabled ?? r.enabled,
        threshold: overrides[r.name]?.threshold ?? DEFAULT_THRESHOLD,
      })),
    [base, overrides]
  );

  const [previewName, setPreviewName] = useState("");
  const [sample, setSample] = useState("");
  const [previewOut, setPreviewOut] = useState("");

  const previewMut = useMutation({
    mutationFn: async () => {
      const cfg = JSON.stringify({
        language: "fr",
        pipeline_level: 2,
        score_threshold: 0.5,
      });
      return analyzeText(sample, { analysisConfigJson: cfg }, credential);
    },
    onSuccess: (data) => {
      const o = data as { result?: { entities?: { entity_type?: string; score?: number; recognizer?: string }[] } };
      const ent = o?.result?.entities ?? [];
      const filtered = previewName.trim() ? ent.filter((e) => (e.recognizer ?? e.entity_type ?? "").includes(previewName)) : ent;
      setPreviewOut(JSON.stringify({ entities: filtered.length ? filtered : ent }, null, 2));
    },
    onError: (e: Error) =>
      setPreviewOut(t("recognizers.previewError", { message: e.message || String(e) })),
  });

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">{t("recognizers.title")}</h1>
        <p className="text-muted-foreground">
          <Trans i18nKey="recognizers.subtitle" components={{ mono: <span className="font-mono" /> }} />
        </p>
      </div>

      {error ? (
        <div className="rounded-lg border border-warm/40 bg-warm/10 px-4 py-3 text-sm" role="alert">
          {String(error)}
        </div>
      ) : null}

      <Card>
        <CardHeader>
          <CardTitle>{t("recognizers.catalogTitle")}</CardTitle>
          <CardDescription>{t("recognizers.catalogDesc")}</CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <p className="text-sm text-muted-foreground">{t("recognizers.loading")}</p>
          ) : rows.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t("recognizers.empty")}</p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t("recognizers.colName")}</TableHead>
                  <TableHead>{t("recognizers.colType")}</TableHead>
                  <TableHead>{t("recognizers.colActive")}</TableHead>
                  <TableHead className="w-[200px]">{t("recognizers.colThreshold")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map((r) => (
                  <TableRow key={r.name}>
                    <TableCell className="font-mono">{r.name}</TableCell>
                    <TableCell>{r.kind}</TableCell>
                    <TableCell>
                      <Switch
                        checked={r.enabled}
                        onCheckedChange={(v) =>
                          setOverrides((prev) => ({ ...prev, [r.name]: { ...prev[r.name], enabled: v } }))
                        }
                      />
                    </TableCell>
                    <TableCell>
                      <Slider
                        value={[r.threshold]}
                        min={0.5}
                        max={0.99}
                        step={0.01}
                        onValueChange={(v) =>
                          setOverrides((prev) => ({ ...prev, [r.name]: { ...prev[r.name], threshold: v[0] } }))
                        }
                      />
                      <span className="text-xs text-muted-foreground">{r.threshold.toFixed(2)}</span>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t("recognizers.previewTitle")}</CardTitle>
          <CardDescription>
            <Trans i18nKey="recognizers.previewDesc" components={{ mono: <span className="font-mono" /> }} />
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-2">
            <Label>{t("recognizers.filterLabel")}</Label>
            <Input
              value={previewName}
              onChange={(e) => setPreviewName(e.target.value)}
              placeholder={t("recognizers.filterPlaceholder")}
            />
          </div>
          <Textarea
            value={sample}
            onChange={(e) => setSample(e.target.value)}
            placeholder={t("recognizers.samplePlaceholder")}
            className="min-h-[100px] font-mono text-sm"
          />
          <Button type="button" variant="secondary" onClick={() => previewMut.mutate()} disabled={previewMut.isPending || !sample.trim()}>
            {previewMut.isPending ? t("recognizers.analyzing") : t("recognizers.runAnalyze")}
          </Button>
          <pre className="max-h-64 overflow-auto rounded-md border border-border bg-muted/20 p-2 text-xs">{previewOut || "—"}</pre>
        </CardContent>
      </Card>
    </div>
  );
}
