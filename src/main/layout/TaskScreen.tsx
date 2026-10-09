import type { ReactNode } from "react";
import { TaskDetail } from "../tasks/TaskDetail";

type TaskScreenProps = {
  title: string;
  subtitle?: string;
  children: ReactNode;
};

export function TaskScreen({ title, subtitle, children }: TaskScreenProps) {
  return (
    <div className="flex min-w-0 flex-1">
      <main className="flex min-w-0 flex-1 flex-col gap-6 overflow-y-auto p-8">
        <header>
          <h1 className="font-heading text-3xl font-bold">{title}</h1>
          {subtitle && <p className="text-caption text-sm">{subtitle}</p>}
        </header>
        {children}
      </main>
      <TaskDetail />
    </div>
  );
}
