import type { ModuleManifest } from "@/core/registry/types";
import { HealthSettings } from "./components/HealthSettings";
import { HealthTab } from "./components/HealthTab";

export const health: ModuleManifest = {
  id: "health",
  panelTabs: [
    { id: "health", label: "Health", order: 20, Component: HealthTab },
  ],
  settingsSections: [{ id: "health", order: 30, Component: HealthSettings }],
};
