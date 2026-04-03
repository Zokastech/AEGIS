// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useEffect, useState, type ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import { Trans, useTranslation } from "react-i18next";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { policiesListQueryOptions, policyDpiaQueryOptions } from "@/queries/gateway-query-options";
import { useAuthCredential } from "@/hooks/useAuthCredential";
import { POLICY_YAML_DEMO_SNIPPET, showPolicyYamlDemo } from "@/lib/policyYamlDemo";

export function PoliciesPage() {
  const { t } = useTranslation("common");
  const credential = useAuthCredential();
  const policiesQuery = useQuery(policiesListQueryOptions(credential));
  const { data: polList, error: listErr, isLoading: listLoading } = policiesQuery;

  const names = polList?.policies ?? [];
  const [active, setActive] = useState("");

  useEffect(() => {
    if (names.length === 0) {
      setActive("");
      return;
    }
    if (!active || !names.includes(active)) setActive(names[0]);
  }, [names, active]);

  const dpiaQuery = useQuery(policyDpiaQueryOptions(credential, active));
  const { data: dpiaMd, error: dpiaErr, isFetching: dpiaLoading } = dpiaQuery;

  const body = dpiaLoading ? t("policies.dpiaLoading") : dpiaMd ?? "";
  const errMsg = dpiaErr instanceof Error ? dpiaErr.message : String(dpiaErr ?? "");

  let policySelectContent: ReactNode;
  if (listLoading) {
    policySelectContent = <p className="text-sm text-muted-foreground">{t("policies.loading")}</p>;
  } else if (names.length === 0) {
    policySelectContent = <p className="text-sm text-muted-foreground">{t("policies.empty")}</p>;
  } else {
    policySelectContent = (
      <Select value={active} onValueChange={setActive}>
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {names.map((n) => (
            <SelectItem key={n} value={n}>
              {n}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">{t("policies.title")}</h1>
        <p className="text-muted-foreground">
          <Trans i18nKey="policies.subtitle" components={{ mono: <span className="font-mono" /> }} />
        </p>
      </div>

      {listErr ? (
        <div className="rounded-lg border border-warm/40 bg-warm/10 px-4 py-3 text-sm" role="alert">
          {t("policies.listError")} {String(listErr)}
        </div>
      ) : null}

      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>{t("policies.cardPolicy")}</CardTitle>
            <CardDescription>{t("policies.cardPolicyDesc")}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>{t("policies.labelName")}</Label>
              {policySelectContent}
            </div>
            <Button
              variant="secondary"
              type="button"
              disabled={!body || Boolean(errMsg && !dpiaLoading)}
              onClick={() => navigator.clipboard.writeText(body)}
            >
              {t("policies.copyReport")}
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{t("policies.yamlTitle")}</CardTitle>
            <CardDescription>{t("policies.yamlDesc")}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <p className="text-sm text-muted-foreground">
              <Trans i18nKey="policies.yamlBody" components={{ mono: <span className="font-mono" /> }} />
            </p>
            {showPolicyYamlDemo() ? (
              <>
                <p className="text-xs font-medium text-amber-800 dark:text-amber-400/95" role="note">
                  {t("policies.yamlDemoBanner")}
                </p>
                <pre className="max-h-[min(40vh,320px)] overflow-auto rounded-md border border-border bg-muted/20 p-3 font-mono text-[11px] leading-relaxed whitespace-pre-wrap">
                  {POLICY_YAML_DEMO_SNIPPET.trimEnd()}
                </pre>
              </>
            ) : null}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>{t("policies.dpiaTitle")}</CardTitle>
          <CardDescription>{t("policies.dpiaDesc")}</CardDescription>
        </CardHeader>
        <CardContent>
          {errMsg && !dpiaLoading ? (
            <p className="mb-3 text-sm text-warm">{errMsg}</p>
          ) : null}
          <pre className="max-h-[min(70vh,560px)] overflow-auto rounded-md border border-border bg-muted/20 p-4 font-mono text-xs whitespace-pre-wrap">
            {active ? body : t("policies.selectPolicy")}
          </pre>
        </CardContent>
      </Card>
    </div>
  );
}
