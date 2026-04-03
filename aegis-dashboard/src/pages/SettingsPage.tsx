// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useState } from "react";
import { Trans, useTranslation } from "react-i18next";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Switch } from "@/components/ui/switch";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Slider } from "@/components/ui/slider";

export function SettingsPage() {
  const { t } = useTranslation("common");
  const [pipeline, setPipeline] = useState([2]);
  const [timeoutMs, setTimeoutMs] = useState("20000");
  const [nerModel, setNerModel] = useState("ner_onnx_v1");
  const [webhook, setWebhook] = useState("");
  const [siemCef, setSiemCef] = useState(true);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">{t("settings.title")}</h1>
        <p className="text-muted-foreground">{t("settings.subtitle")}</p>
      </div>

      <Tabs defaultValue="engine">
        <TabsList className="flex flex-wrap">
          <TabsTrigger value="engine">{t("settings.tabEngine")}</TabsTrigger>
          <TabsTrigger value="keys">{t("settings.tabKeys")}</TabsTrigger>
          <TabsTrigger value="rbac">{t("settings.tabRbac")}</TabsTrigger>
          <TabsTrigger value="integrations">{t("settings.tabIntegrations")}</TabsTrigger>
        </TabsList>

        <TabsContent value="engine">
          <Card>
            <CardHeader>
              <CardTitle>{t("settings.engineTitle")}</CardTitle>
              <CardDescription>{t("settings.engineDesc")}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6 max-w-xl">
              <div className="space-y-2">
                <Label>
                  {t("settings.pipelineMax")} {pipeline[0]}
                </Label>
                <Slider value={pipeline} onValueChange={setPipeline} min={1} max={3} step={1} />
              </div>
              <div className="space-y-2">
                <Label>{t("settings.timeoutMs")}</Label>
                <Input value={timeoutMs} onChange={(e) => setTimeoutMs(e.target.value)} />
              </div>
              <div className="space-y-2">
                <Label>{t("settings.nerModel")}</Label>
                <Input value={nerModel} onChange={(e) => setNerModel(e.target.value)} />
              </div>
              <Button type="button" variant="secondary" disabled title={t("settings.saveDisabledTitle")}>
                {t("settings.saveDisabled")}
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="keys">
          <Card>
            <CardHeader>
              <CardTitle>{t("settings.keysTitle")}</CardTitle>
              <CardDescription>{t("settings.keysDesc")}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4 max-w-xl">
              <div className="space-y-2">
                <Label>{t("settings.newKeyRole")}</Label>
                <Input placeholder={t("settings.newKeyPlaceholder")} />
              </div>
              <Button type="button" variant="secondary" disabled title={t("settings.generateKeyTitle")}>
                {t("settings.generateKey")}
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="rbac">
          <Card>
            <CardHeader>
              <CardTitle>{t("settings.rbacTitle")}</CardTitle>
              <CardDescription>{t("settings.rbacDesc")}</CardDescription>
            </CardHeader>
            <CardContent>
              <p className="mb-4 text-sm text-muted-foreground">
                <Trans i18nKey="settings.rbacBody" components={{ mono: <span className="font-mono" /> }} />
              </p>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("settings.colSubject")}</TableHead>
                    <TableHead>{t("settings.colRole")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell colSpan={2} className="text-muted-foreground">
                      {t("settings.rbacEmpty")}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="integrations">
          <Card>
            <CardHeader>
              <CardTitle>{t("settings.integrationsTitle")}</CardTitle>
              <CardDescription>{t("settings.integrationsDesc")}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4 max-w-xl">
              <div className="space-y-2">
                <Label>{t("settings.webhookUrl")}</Label>
                <Input value={webhook} onChange={(e) => setWebhook(e.target.value)} placeholder={t("settings.webhookPlaceholder")} />
              </div>
              <div className="flex items-center gap-2">
                <Switch checked={siemCef} onCheckedChange={setSiemCef} id="cef" />
                <Label htmlFor="cef">{t("settings.cefLabel")}</Label>
              </div>
              <div className="space-y-2">
                <Label>{t("settings.testPayload")}</Label>
                <Textarea className="font-mono text-xs" readOnly value={JSON.stringify({ event: "aegis.test", severity: "info" }, null, 2)} />
              </div>
              <Button type="button" variant="outline">
                {t("settings.sendTest")}
              </Button>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
