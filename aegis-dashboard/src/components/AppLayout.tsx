// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useState } from "react";
import { Link, Outlet, useNavigate, useRouterState } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { LayoutDashboard, FlaskConical, Scan, FileJson, ScrollText, Settings, LogOut, Menu } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { AegisBrandMark, AegisWordmark } from "@/components/AegisBrandLogo";
import { useAuthStore } from "@/stores/authStore";
import { I18nLanguageSelect } from "@/components/I18nLanguageSelect";
import { Sheet, SheetTrigger, SheetContent, SheetHeader, SheetTitle, SheetClose } from "@/components/ui/sheet";

const nav = [
  { to: "/", labelKey: "nav.dashboard", icon: LayoutDashboard },
  { to: "/playground", labelKey: "nav.playground", icon: FlaskConical },
  { to: "/recognizers", labelKey: "nav.recognizers", icon: Scan },
  { to: "/policies", labelKey: "nav.policies", icon: FileJson },
  { to: "/audit", labelKey: "nav.audit", icon: ScrollText },
  { to: "/settings", labelKey: "nav.settings", icon: Settings },
] as const;

function navItemClass(pathname: string, to: string) {
  return cn(
    "flex min-h-[2.75rem] items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors duration-200 ease-smooth",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-orange/45 focus-visible:ring-offset-2 focus-visible:ring-offset-background",
    pathname === to
      ? "bg-gradient-to-r from-brand-orange/15 to-brand-pink/10 text-brand-orange shadow-sm ring-1 ring-brand-orange/20"
      : "text-zokastech-gray hover:bg-muted hover:text-zokastech-dark"
  );
}

function SidebarBrand() {
  const { t } = useTranslation("common");
  return (
    <div className="border-b border-orange-100/80 bg-gradient-to-br from-white via-orange-50/30 to-blue-50/40 px-3 py-4">
      <div className="flex items-center gap-3 rounded-xl border border-orange-100/90 bg-white/95 p-3 shadow-brand">
        <div className="flex shrink-0 items-center justify-center rounded-xl border border-brand-orange/20 bg-gradient-to-br from-brand-orange/10 via-white to-brand-blue/10 p-1 shadow-sm">
          <AegisBrandMark size="md" />
        </div>
        <div className="min-w-0 flex-1">
          <AegisWordmark className="block text-[0.95rem] leading-tight tracking-[0.1em]" />
          <a
            href="https://zokastech.fr"
            target="_blank"
            rel="noreferrer"
            className="mt-0.5 inline-block text-[11px] font-semibold text-brand-pink transition-colors duration-200 hover:text-brand-orange focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-orange/40 focus-visible:ring-offset-2 rounded-sm"
          >
            zokastech.fr
          </a>
          <div className="mt-0.5 text-[10px] font-medium leading-tight text-zokastech-gray">{t("app.tagline")}</div>
        </div>
      </div>
    </div>
  );
}

function MainNavList({
  pathname,
  onItemActivate,
}: {
  pathname: string;
  onItemActivate?: () => void;
}) {
  const { t } = useTranslation("common");
  return (
    <ul className="flex flex-1 flex-col gap-0.5 p-2">
      {nav.map(({ to, labelKey, icon: Icon }) => (
        <li key={to}>
          <Link to={to} className={navItemClass(pathname, to)} onClick={onItemActivate}>
            <Icon className="h-4 w-4 shrink-0" aria-hidden />
            {t(labelKey)}
          </Link>
        </li>
      ))}
    </ul>
  );
}

export function AppLayout() {
  const { t } = useTranslation("common");
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const logout = useAuthStore((s) => s.logout);
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <div
      className="flex min-h-screen bg-app-shell"
      style={{ "--header-height": "3.5rem" } as React.CSSProperties}
    >
      <a
        href="#main-content"
        className="fixed left-4 top-4 z-[100] -translate-y-[200%] rounded-md bg-zokastech-dark px-3 py-2 text-sm font-semibold text-white shadow-md transition-transform focus:translate-y-0 focus:outline-none focus-visible:ring-2 focus-visible:ring-brand-orange focus-visible:ring-offset-2"
      >
        {t("nav.skipToContent")}
      </a>

      <aside className="hidden w-56 shrink-0 flex-col border-r border-border/90 bg-card/90 shadow-sm backdrop-blur-md md:flex">
        <SidebarBrand />
        <nav className="flex min-h-0 flex-1 flex-col" aria-label={t("nav.ariaMain")}>
          <MainNavList pathname={pathname} />
        </nav>
        <div className="border-t border-border p-2 space-y-2">
          <div className="px-1">
            <I18nLanguageSelect />
          </div>
          <Button
            variant="ghost"
            className="w-full justify-start gap-2 text-zokastech-gray hover:text-brand-orange"
            onClick={() => {
              queryClient.clear();
              logout();
              navigate({ to: "/login" });
            }}
          >
            <LogOut className="h-4 w-4" aria-hidden />
            {t("auth.logout")}
          </Button>
        </div>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="sticky top-0 z-40 flex h-[var(--header-height,3.5rem)] items-center justify-between gap-3 border-b border-border/90 bg-card/95 px-3 shadow-sm backdrop-blur-md md:hidden">
          <div className="flex min-w-0 flex-1 items-center gap-2">
            <Sheet open={mobileOpen} onOpenChange={setMobileOpen}>
              <SheetTrigger asChild>
                <Button
                  type="button"
                  variant="outline"
                  size="icon"
                  className="shrink-0 border-border"
                  aria-controls="mobile-navigation"
                >
                  <Menu className="h-5 w-5" aria-hidden />
                  <span className="sr-only">{t("nav.openMenu")}</span>
                </Button>
              </SheetTrigger>
              <SheetContent
                side="left"
                className="flex w-[min(100%,18rem)] flex-col p-0"
                id="mobile-navigation"
                aria-describedby={undefined}
              >
                <SheetHeader className="flex flex-row items-center justify-between gap-2 border-b border-orange-100/80 bg-gradient-to-br from-white via-orange-50/20 to-blue-50/30 px-4 py-3">
                  <SheetTitle className="font-display text-sm tracking-wide">{t("nav.menuTitle")}</SheetTitle>
                  <SheetClose asChild>
                    <Button type="button" variant="ghost" size="sm" className="h-8 shrink-0 px-2 text-xs" aria-label={t("nav.closeMenu")}>
                      {t("nav.closeMenu")}
                    </Button>
                  </SheetClose>
                </SheetHeader>
                <nav className="flex flex-1 flex-col overflow-y-auto" aria-label={t("nav.ariaMain")}>
                  <MainNavList pathname={pathname} onItemActivate={() => setMobileOpen(false)} />
                </nav>
                <div className="border-t border-border p-2 space-y-2">
                  <I18nLanguageSelect />
                  <Button
                    variant="ghost"
                    className="w-full justify-start gap-2 text-zokastech-gray"
                    onClick={() => {
                      setMobileOpen(false);
                      queryClient.clear();
                      logout();
                      navigate({ to: "/login" });
                    }}
                  >
                    <LogOut className="h-4 w-4" aria-hidden />
                    {t("auth.logout")}
                  </Button>
                </div>
              </SheetContent>
            </Sheet>
            <div className="flex min-w-0 items-center gap-2">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-brand-orange/20 bg-gradient-to-br from-brand-orange/10 to-brand-blue/10 p-0.5">
                <AegisBrandMark size="sm" className="h-7 w-7" />
              </div>
              <div className="min-w-0">
                <AegisWordmark className="block truncate text-sm tracking-[0.1em]" />
                <span className="truncate text-[10px] text-muted-foreground">{t("app.brandSuffix")}</span>
              </div>
            </div>
          </div>
          <I18nLanguageSelect compact className="shrink-0" />
        </header>

        <main
          id="main-content"
          tabIndex={-1}
          className="flex-1 overflow-auto outline-none focus:outline-none"
        >
          <div className="mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 md:px-8 md:py-10">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
}
