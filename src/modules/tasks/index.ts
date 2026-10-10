import { lazyRouteComponent } from "@tanstack/react-router";
import type { ModuleManifest } from "@/core/registry/types";
import { AreasNav, TasksNav } from "./components/TasksNav";
import { TodayLeft } from "./panel/TodayLeft";
import { TodayTab } from "./panel/TodayTab";
import { TasksSync } from "./TasksSync";

export const tasks: ModuleManifest = {
  id: "tasks",
  name: "Tasks",
  description: "Tasks, Top 3, areas, reminders on tasks and quick add.",
  routes: [
    {
      path: "/",
      Component: lazyRouteComponent(() => import("./pages/Today"), "Today"),
    },
    {
      path: "/upcoming",
      Component: lazyRouteComponent(
        () => import("./pages/Upcoming"),
        "Upcoming",
      ),
    },
    {
      path: "/category/$categoryId",
      Component: lazyRouteComponent(
        () => import("./pages/CategoryView"),
        "CategoryView",
      ),
    },
  ],
  sidebarSections: [
    { id: "tasks", order: 10, Component: TasksNav },
    { id: "areas", order: 20, Component: AreasNav },
  ],
  panelHeaderLines: [{ id: "today-left", order: 10, Component: TodayLeft }],
  panelTabs: [{ id: "today", label: "Today", order: 10, Component: TodayTab }],
  listeners: [{ id: "tasks-sync", order: 10, Component: TasksSync }],
};
