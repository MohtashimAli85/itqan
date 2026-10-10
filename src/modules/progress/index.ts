import { lazyRouteComponent } from "@tanstack/react-router";
import type { ModuleManifest } from "@/core/registry/types";
import { ProgressNavItem } from "./components/ProgressNavItem";
import { ProgressRow } from "./panel/ProgressRow";
import { RewardCelebrations, RewardSync } from "./RewardSync";
import { RewardSettings } from "./settings/RewardSettings";

export const progress: ModuleManifest = {
  id: "progress",
  routes: [
    {
      path: "/progress",
      Component: lazyRouteComponent(
        () => import("./pages/Progress"),
        "Progress",
      ),
    },
  ],
  navItems: [{ id: "progress", order: 20, Component: ProgressNavItem }],
  panelRows: [{ id: "progress", order: 10, Component: ProgressRow }],
  settingsSections: [{ id: "rewards", order: 45, Component: RewardSettings }],
  listeners: [
    { id: "reward-sync", order: 20, Component: RewardSync, surface: "main" },
    {
      id: "reward-celebrations",
      order: 20,
      Component: RewardCelebrations,
      surface: "overlay",
    },
  ],
};
