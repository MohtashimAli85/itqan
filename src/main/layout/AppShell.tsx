import { Outlet, useNavigate } from "@tanstack/react-router";
import { events } from "@/shared/bindings/bindings";
import { useGoalsSync } from "@/shared/hooks/useGoals";
import { useRewardSync } from "@/shared/hooks/useProgress";
import { useTasksSync } from "@/shared/hooks/useTasks";
import { useTauriEvent } from "@/shared/hooks/useTauriEvent";
import { Sidebar } from "./Sidebar";

export function AppShell() {
  const navigate = useNavigate();
  useTasksSync();
  useGoalsSync();
  useRewardSync();

  useTauriEvent(events.navigate, ({ to }) => {
    void navigate({ to });
  });

  return (
    <div className="flex h-screen overflow-hidden bg-background text-foreground">
      <Sidebar />
      <Outlet />
    </div>
  );
}
