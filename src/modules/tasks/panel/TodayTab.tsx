import type { Task } from "@/shared/bindings/bindings";
import {
  useSetTaskStatus,
  useTodayTasks,
} from "@/modules/tasks/hooks/useTasks";
import { FocusRow } from "@/core/ui/FocusRow";
import { QuickAdd } from "./QuickAdd";
import { TaskRow } from "./TaskRow";

export function TodayTab() {
  const setStatus = useSetTaskStatus();
  const { topThree, due } = useTodayTasks();

  const complete = (task: Task) =>
    setStatus.mutate({ id: task.id, status: "done" });

  return (
    <div className="flex flex-col gap-3">
      <QuickAdd />

      <FocusRow />

      {topThree.length > 0 && (
        <section aria-labelledby="panel-top-three">
          <h3
            id="panel-top-three"
            className="mb-1 px-2 text-xs font-medium text-caption"
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
            className="mb-1 px-2 text-xs font-medium text-caption"
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
