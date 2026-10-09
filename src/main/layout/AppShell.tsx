import { useEffect } from "react";
import { Outlet, useNavigate } from "@tanstack/react-router";
import { events } from "@/shared/bindings/bindings";
import { useGoalsSync } from "@/shared/hooks/useGoals";
import { useRewardSync } from "@/shared/hooks/useProgress";
import { useTasksSync } from "@/shared/hooks/useTasks";
import { Sidebar } from "./Sidebar";

export function AppShell() {
  const navigate = useNavigate();
  useTasksSync();
  useGoalsSync();
  useRewardSync();

  useEffect(() => {
    const unlisten = events.navigate.listen((event) => {
      void navigate({ to: event.payload.to });
    });
    return () => void unlisten.then((stop) => stop());
  }, [navigate]);

  return (
    <div className="bg-background text-foreground flex h-screen overflow-hidden">
      <Sidebar />
      <Outlet />
    </div>
  );
}
