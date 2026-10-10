import type { ComponentType } from "react";

export type ModuleId =
  "tasks" | "salah" | "health" | "progress" | "focus" | "learning" | "memory";

export type Contribution = {
  id: string;
  order: number;
  Component: ComponentType;
};

export type PanelTab = Contribution & { label: string };

export type ModuleManifest = {
  id: ModuleId;
  panelTabs?: PanelTab[];
  settingsSections?: Contribution[];
  sidebarWidgets?: Contribution[];
};
