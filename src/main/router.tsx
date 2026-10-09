import type { QueryClient } from "@tanstack/react-query";
import {
  createRootRouteWithContext,
  createRoute,
  createRouter,
  Outlet,
  redirect,
} from "@tanstack/react-router";
import { AppShell } from "./layout/AppShell";
import { Onboarding } from "./onboarding/Onboarding";
import { ensureOnboarded } from "./onboardingStatus";
import { CategoryView } from "./screens/CategoryView";
import { Goals } from "./screens/Goals";
import { Progress } from "./screens/Progress";
import { Settings } from "./screens/Settings";
import { Today } from "./screens/Today";
import { Upcoming } from "./screens/Upcoming";

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

const page = <Path extends string>(
  path: Path,
  component: () => React.ReactNode,
) => createRoute({ getParentRoute: () => appRoute, path, component });

const routeTree = rootRoute.addChildren([
  appRoute.addChildren([
    page("/", Today),
    page("/upcoming", Upcoming),
    page("/category/$categoryId", CategoryView),
    page("/goals", Goals),
    page("/progress", Progress),
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
