import type { QueryClient } from "@tanstack/react-query";
import {
  createRootRouteWithContext,
  createRoute,
  createRouter,
  Outlet,
  redirect,
  type RouteComponent,
} from "@tanstack/react-router";
import { AppShell } from "./layout/AppShell";
import { Onboarding } from "./onboarding/Onboarding";
import { ensureOnboarded } from "./onboardingStatus";
import { modules } from "@/modules";
import { gate } from "./ModuleGate";
import { Goals } from "./screens/Goals";
import { Settings } from "./screens/Settings";

type RouterContext = { queryClient: QueryClient };

const rootRoute = createRootRouteWithContext<RouterContext>()({
  component: Outlet,
});

const appRoute = createRoute({
  getParentRoute: () => rootRoute,
  id: "app",
  beforeLoad: async ({ context }) => {
    if (!(await ensureOnboarded(context.queryClient))) {
      throw redirect({ to: "/onboarding" });
    }
  },
  component: AppShell,
});

const page = <Path extends string>(path: Path, component: RouteComponent) =>
  createRoute({ getParentRoute: () => appRoute, path, component });

const routeTree = rootRoute.addChildren([
  appRoute.addChildren([
    ...modules.flatMap((manifest) =>
      (manifest.routes ?? []).map((route) =>
        page(route.path, gate(manifest.id, route.Component)),
      ),
    ),
    page("/goals", Goals),
    page("/settings", Settings),
  ]),
  createRoute({
    getParentRoute: () => rootRoute,
    path: "/onboarding",
    component: Onboarding,
  }),
]);

export function createAppRouter(queryClient: QueryClient) {
  return createRouter({ routeTree, context: { queryClient } });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
