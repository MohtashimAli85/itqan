import type { QueryClient } from "@tanstack/react-query";
import {
  createRootRouteWithContext,
  createRoute,
  createRouter,
  Outlet,
  redirect,
} from "@tanstack/react-router";
import { Onboarding } from "./onboarding/Onboarding";
import { ensureOnboarded } from "./onboardingStatus";
import { Goals } from "./screens/Goals";
import { Preview } from "./screens/Preview";

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
  component: Outlet,
});

const homeRoute = createRoute({
  getParentRoute: () => appRoute,
  path: "/",
  component: Preview,
});

const goalsRoute = createRoute({
  getParentRoute: () => appRoute,
  path: "/goals",
  component: Goals,
});

const onboardingRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/onboarding",
  component: Onboarding,
});

const routeTree = rootRoute.addChildren([
  appRoute.addChildren([homeRoute, goalsRoute]),
  onboardingRoute,
]);

export function createAppRouter(queryClient: QueryClient) {
  return createRouter({ routeTree, context: { queryClient } });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
