// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Trans, useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { auditExportQueryOptions } from "@/queries/gateway-query-options";
import { useAuthCredential } from "@/hooks/useAuthCredential";

export function AuditPage() {
  const { t } = useTranslation("common");
  const credential = useAuthCredential();
  const { data, error, isLoading, isFetching, refetch } = useQuery(auditExportQueryOptions(credential));

  const rows = data ?? [];

  const [user, setUser] = useState("");
  const [action, setAction] = useState("");
  const [result, setResult] = useState<"all" | "ok" | "fail">("all");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [page, setPage] = useState(1);
  const pageSize = 10;

  const filtered = useMemo(() => {
    return rows.filter((r) => {
      if (user && !r.user.toLowerCase().includes(user.toLowerCase())) return false;
      if (action && !r.action.toLowerCase().includes(action.toLowerCase()) && !r.path.toLowerCase().includes(action.toLowerCase()))
        return false;
      if (result === "ok" && !r.success) return false;
      if (result === "fail" && r.success) return false;
      if (from && r.at && r.at < new Date(from).toISOString()) return false;
      if (to && r.at && r.at > new Date(to).toISOString()) return false;
      return true;
    });
  }, [rows, user, action, result, from, to]);

  const pageRows = filtered.slice((page - 1) * pageSize, page * pageSize);
  const totalPages = Math.max(1, Math.ceil(filtered.length / pageSize));

  function exportCsv() {
    const header = "id,at,user,action,path,success,status_code\n";
    const body = filtered
      .map((r) => `${r.id},${r.at},${r.user},${r.action},${r.path},${r.success},${r.statusCode ?? ""}`)
      .join("\n");
    const blob = new Blob([header + body], { type: "text/csv" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "aegis-audit.csv";
    a.click();
  }

  function exportJson() {
    const blob = new Blob([JSON.stringify(filtered, null, 2)], { type: "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "aegis-audit.json";
    a.click();
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">{t("audit.title")}</h1>
          <p className="text-muted-foreground">
            <Trans i18nKey="audit.subtitle" components={{ mono: <span className="font-mono" /> }} />
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" size="sm" onClick={() => refetch()} disabled={isFetching}>
            {t("audit.refresh")}
          </Button>
          <Button variant="secondary" size="sm" onClick={exportCsv} disabled={!filtered.length}>
            {t("audit.exportCsv")}
          </Button>
          <Button variant="secondary" size="sm" onClick={exportJson} disabled={!filtered.length}>
            {t("audit.exportJson")}
          </Button>
        </div>
      </div>

      {error ? (
        <div className="rounded-lg border border-warm/40 bg-warm/10 px-4 py-3 text-sm" role="alert">
          {String(error)}
        </div>
      ) : null}

      <Card>
        <CardHeader>
          <CardTitle>{t("audit.filtersTitle")}</CardTitle>
          <CardDescription>{t("audit.filtersDesc")}</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <div className="space-y-2">
            <Label>{t("audit.from")}</Label>
            <Input type="datetime-local" value={from} onChange={(e) => setFrom(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label>{t("audit.to")}</Label>
            <Input type="datetime-local" value={to} onChange={(e) => setTo(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label>{t("audit.user")}</Label>
            <Input value={user} onChange={(e) => setUser(e.target.value)} placeholder={t("audit.userPlaceholder")} />
          </div>
          <div className="space-y-2">
            <Label>{t("audit.action")}</Label>
            <Input value={action} onChange={(e) => setAction(e.target.value)} placeholder={t("audit.actionPlaceholder")} />
          </div>
          <div className="space-y-2">
            <Label>{t("audit.result")}</Label>
            <select
              className="flex h-9 w-full rounded-md border border-border bg-muted/30 px-3 text-sm"
              value={result}
              onChange={(e) => setResult(e.target.value as typeof result)}
            >
              <option value="all">{t("audit.resultAll")}</option>
              <option value="ok">{t("audit.resultOk")}</option>
              <option value="fail">{t("audit.resultFail")}</option>
            </select>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className="pt-6">
          {isLoading ? (
            <p className="text-sm text-muted-foreground">{t("audit.loading")}</p>
          ) : rows.length === 0 && !error ? (
            <p className="text-sm text-muted-foreground">{t("audit.empty")}</p>
          ) : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("audit.colTime")}</TableHead>
                    <TableHead>{t("audit.colUser")}</TableHead>
                    <TableHead>{t("audit.colAction")}</TableHead>
                    <TableHead>{t("audit.colPath")}</TableHead>
                    <TableHead>{t("audit.colResult")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {pageRows.map((r) => (
                    <TableRow key={r.id}>
                      <TableCell className="whitespace-nowrap text-xs text-muted-foreground">{r.at}</TableCell>
                      <TableCell className="font-mono text-sm">{r.user}</TableCell>
                      <TableCell className="text-sm">{r.action}</TableCell>
                      <TableCell className="font-mono text-xs text-muted-foreground">{r.path}</TableCell>
                      <TableCell>
                        <Badge variant={r.success ? "default" : "destructive"}>
                          {r.success ? t("audit.badgeOk") : t("audit.badgeError")}
                        </Badge>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
              <div className="mt-4 flex items-center justify-between text-sm text-muted-foreground">
                <span>
                  {t("audit.pageInfo", { page, totalPages, count: filtered.length })}
                </span>
                <div className="flex gap-2">
                  <Button variant="outline" size="sm" disabled={page <= 1} onClick={() => setPage((p) => p - 1)}>
                    {t("audit.prev")}
                  </Button>
                  <Button variant="outline" size="sm" disabled={page >= totalPages} onClick={() => setPage((p) => p + 1)}>
                    {t("audit.next")}
                  </Button>
                </div>
              </div>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
