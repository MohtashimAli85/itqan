import type { ComponentType } from "react";

export type PanelTab = {
  id: string;
  label: string;
  order: number;
  Component: ComponentType;
};

export type SettingsSection = {
  id: string;
  order: number;
  Component: ComponentType;
};

export type SidebarWidget = {
  id: string;
  order: number;
  Component: ComponentType;
};

export type ModuleManifest = {
  id: string;
  panelTabs?: PanelTab[];
  settingsSections?: SettingsSection[];
  sidebarWidgets?: SidebarWidget[];
};
