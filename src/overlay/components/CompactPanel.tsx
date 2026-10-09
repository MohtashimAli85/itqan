import { useEffect, useRef } from "react";
import type { Task } from "@/shared/bindings/bindings";
import { openTasks, useSetTaskStatus, useTasks } from "@/shared/hooks/useTasks";
import { greeting, todayTasks } from "@/shared/lib/today";
import { QuickAdd } from "./QuickAdd";
import { TaskRow } from "./TaskRow";

type CompactPanelProps = {
  onClose: () => void;
  onResize: () => void;
};

export function CompactPanel({ onClose, onResize }: CompactPanelProps) {
  const panel = useRef<HTMLDivElement>(null);
  const { data: tasks = [] } = useTasks(openTasks);
  const setStatus = useSetTaskStatus();
  const { topThree, due } = todayTasks(tasks);

  useEffect(() => {
    const element = panel.current;
    if (!element) return;
    const observer = new ResizeObserver(onResize);
    observer.observe(element);
    return () => observer.disconnect();
  }, [onResize]);

  const complete = (task: Task) =>
    setStatus.mutate({ id: task.id, status: "done" });

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  return (
    <div
      ref={panel}
      data-hit-area
      role="dialog"
      aria-label="Itqan"
      className="orb-popover bg-card text-card-foreground ring-border absolute flex w-80 flex-col gap-3 rounded-2xl p-4 shadow-xl ring-1"
    >
      <header>
        <p className="font-heading text-lg font-semibold">{greeting()}</p>
        <p className="text-caption text-xs">
          {topThree.length + due.length === 0
            ? "Nothing due today. Add your top 3."
            : `${topThree.length + due.length} left for today`}
        </p>
      </header>

      <QuickAdd />

      {topThree.length > 0 && (
        <section aria-labelledby="panel-top-three">
          <h3
            id="panel-top-three"
            className="text-caption mb-1 px-2 text-xs font-medium"
          >
            Top 3
          </h3>
          <ul>
            {topThree.map((task) => (
              <TaskRow key={task.id} task={task} onComplete={complete} />
            ))}
          </ul>
        </section>
      )}

      {due.length > 0 && (
        <section aria-labelledby="panel-due">
          <h3
            id="panel-due"
            className="text-caption mb-1 px-2 text-xs font-medium"
          >
            Due today
          </h3>
          <ul className="max-h-48 overflow-y-auto">
            {due.map((task) => (
              <TaskRow key={task.id} task={task} onComplete={complete} />
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}
