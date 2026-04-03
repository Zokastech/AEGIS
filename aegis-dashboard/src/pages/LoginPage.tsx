// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { Trans, useTranslation } from "react-i18next";
import { AegisBrandMark, AegisWordmark } from "@/components/AegisBrandLogo";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useAuthStore } from "@/stores/authStore";
import { I18nLanguageSelect } from "@/components/I18nLanguageSelect";

const mono = <span className="font-mono" />;

const devLoginBypassEnabled =
  import.meta.env.DEV && import.meta.env.VITE_ENABLE_DEV_LOGIN_BYPASS === "true";

export function LoginPage() {
  const { t } = useTranslation("common");
  const [displayName, setDisplayName] = useState("");
  const [credential, setCredential] = useState("");
  const [devNoAuth, setDevNoAuth] = useState(false);
  const [err, setErr] = useState("");
  const setAuth = useAuthStore((s) => s.setAuth);
  const setDevBypass = useAuthStore((s) => s.setDevBypass);
  const navigate = useNavigate();

  function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setErr("");
    if (devNoAuth) {
      if (!devLoginBypassEnabled) {
        setErr(t("login.errorDevBypassDisabled"));
        return;
      }
      setDevBypass(displayName.trim() || "dev");
      navigate({ to: "/" });
      return;
    }
    const c = credential.trim();
    if (!c) {
      setErr(t("login.errorCredential"));
      return;
    }
    setAuth(c, displayName.trim() || "—");
    navigate({ to: "/" });
  }

  return (
    <div className="relative flex min-h-screen flex-col items-center justify-center bg-mesh-zokastech p-5 md:p-10">
      <div className="absolute right-5 top-5 md:right-10 md:top-8">
        <I18nLanguageSelect compact />
      </div>
      <div className="mb-10 flex flex-col items-center text-center motion-safe:animate-fade-in">
        <div className="motion-safe:animate-shield-float mb-4">
          <AegisBrandMark size="xl" glow aria-hidden />
        </div>
        <h1 className="m-0 font-display">
          <AegisWordmark className="text-3xl md:text-4xl" />
        </h1>
        <p className="mt-2 max-w-md text-sm font-medium text-zokastech-gray">{t("login.subtitle")}</p>
        <p className="mt-1 text-xs font-semibold text-brand-orange/90">{t("app.tagline")}</p>
      </div>
      <Card className="w-full max-w-md border border-orange-100/90 bg-white/95 shadow-brand-lg backdrop-blur-sm motion-safe:animate-fade-in">
        <CardHeader>
          <CardTitle>{t("login.title")}</CardTitle>
          <CardDescription>
            <Trans i18nKey="login.description" components={{ mono }} />
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={onSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="label">{t("login.labelOptional")}</Label>
              <Input
                id="label"
                type="text"
                autoComplete="nickname"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                placeholder={t("login.labelPlaceholder")}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="credential">{t("login.credential")}</Label>
              <Input
                id="credential"
                type="password"
                autoComplete="current-password"
                value={credential}
                onChange={(e) => setCredential(e.target.value)}
                disabled={devNoAuth}
                placeholder={devNoAuth ? t("login.credentialDisabled") : t("login.credentialPlaceholder")}
              />
            </div>
            {devLoginBypassEnabled ? (
              <div className="flex items-start gap-2">
                <input
                  id="dev"
                  type="checkbox"
                  className="mt-1 h-4 w-4 rounded border-[#e2e8f0] text-brand-orange focus:ring-brand-orange/30"
                  checked={devNoAuth}
                  onChange={(e) => setDevNoAuth(e.target.checked)}
                />
                <Label htmlFor="dev" className="cursor-pointer text-sm font-normal leading-snug">
                  <Trans i18nKey="login.devAuth" components={{ mono }} />
                </Label>
              </div>
            ) : null}
            {err ? (
              <p className="rounded-md border border-brand-pink/40 bg-red-50 px-3 py-2 text-sm text-red-800">{err}</p>
            ) : null}
            <Button type="submit" className="w-full">
              {t("login.submit")}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
