// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { createRootRoute, createRoute, createRouter, Outlet, redirect } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";
import { AppLayout } from "@/components/AppLayout";
import { LoginPage } from "@/pages/LoginPage";
import { DashboardPage } from "@/pages/DashboardPage";
import { PlaygroundPage } from "@/pages/PlaygroundPage";
import { RecognizersPage } from "@/pages/RecognizersPage";
import { PoliciesPage } from "@/pages/PoliciesPage";
import { AuditPage } from "@/pages/AuditPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { ensureAuthHydrated, isSessionValid } from "@/lib/authSession";

const rootRoute = createRootRoute({
  component: () => (
    <>
      <Outlet />
      {import.meta.env.DEV ? <TanStackRouterDevtools position="bottom-right" /> : null}
    </>
  ),
});

const loginRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/login",
  beforeLoad: async () => {
    await ensureAuthHydrated();
    if (isSessionValid()) throw redirect({ to: "/" });
  },
  component: LoginPage,
});

const appLayoutRoute = createRoute({
  getParentRoute: () => rootRoute,
  id: "appShell",
  beforeLoad: async () => {
    await ensureAuthHydrated();
    if (!isSessionValid()) throw redirect({ to: "/login" });
  },
  component: AppLayout,
});

const dashboardRoute = createRoute({
  getParentRoute: () => appLayoutRoute,
  path: "/",
  component: DashboardPage,
});

const playgroundRoute = createRoute({
  getParentRoute: () => appLayoutRoute,
  path: "/playground",
  component: PlaygroundPage,
});

const recognizersRoute = createRoute({
  getParentRoute: () => appLayoutRoute,
  path: "/recognizers",
  component: RecognizersPage,
});

const policiesRoute = createRoute({
  getParentRoute: () => appLayoutRoute,
  path: "/policies",
  component: PoliciesPage,
});

const auditRoute = createRoute({
  getParentRoute: () => appLayoutRoute,
  path: "/audit",
  component: AuditPage,
});

const settingsRoute = createRoute({
  getParentRoute: () => appLayoutRoute,
  path: "/settings",
  component: SettingsPage,
});

const routeTree = rootRoute.addChildren([
  loginRoute,
  appLayoutRoute.addChildren([dashboardRoute, playgroundRoute, recognizersRoute, policiesRoute, auditRoute, settingsRoute]),
]);

export const router = createRouter({
  routeTree,
  defaultPreload: "intent",
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
