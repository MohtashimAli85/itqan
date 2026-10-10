import type { ModuleManifest } from "@/core/registry/types";
import { AreasNav, TasksNav } from "./components/TasksNav";
import { CategoryView } from "./pages/CategoryView";
import { Today } from "./pages/Today";
import { Upcoming } from "./pages/Upcoming";
import { TodayLeft } from "./panel/TodayLeft";
import { TodayTab } from "./panel/TodayTab";
import { TasksSync } from "./TasksSync";

export const tasks: ModuleManifest = {
  id: "tasks",
  routes: [
    { path: "/", Component: Today },
    { path: "/upcoming", Component: Upcoming },
    { path: "/category/$categoryId", Component: CategoryView },
  ],
  sidebarSections: [
    { id: "tasks", order: 10, Component: TasksNav },
    { id: "areas", order: 20, Component: AreasNav },
  ],
  panelHeaderLines: [{ id: "today-left", order: 10, Component: TodayLeft }],
  panelTabs: [{ id: "today", label: "Today", order: 10, Component: TodayTab }],
  listeners: [{ id: "tasks-sync", order: 10, Component: TasksSync }],
};
