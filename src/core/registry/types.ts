import type { ComponentType } from "react";
import type { RouteComponent } from "@tanstack/react-router";

export type ModuleId =
  "tasks" | "salah" | "health" | "progress" | "focus" | "learning" | "memory";

export type Contribution = {
  id: string;
  order: number;
  Component: ComponentType;
};

export type PanelTab = Contribution & { label: string };

export type Surface = "main" | "overlay";

export type Listener = Contribution & { surface?: Surface };

export type ModuleRoute = {
  path: string;
  Component: RouteComponent;
};

export type ModuleManifest = {
  id: ModuleId;
  routes?: ModuleRoute[];
  sidebarSections?: Contribution[];
  navItems?: Contribution[];
  sidebarWidgets?: Contribution[];
  panelHeaderLines?: Contribution[];
  panelRows?: Contribution[];
  panelTabs?: PanelTab[];
  settingsSections?: Contribution[];
  listeners?: Listener[];
};
