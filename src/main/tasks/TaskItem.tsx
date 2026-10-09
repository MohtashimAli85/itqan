import { Star } from "lucide-react";
import type { Category, Task } from "@/shared/bindings/bindings";
import { Checkbox } from "@/shared/components/ui/checkbox";
import { useSetTaskStatus, useSetTopThree } from "@/shared/hooks/useTasks";
import { dayjs } from "@/shared/lib/dayjs";
import { categoryDot } from "@/shared/lib/taskGroups";
import { useMainUi } from "../uiStore";

type TaskItemProps = {
  task: Task;
  category: Category | undefined;
};

export function TaskItem({ task, category }: TaskItemProps) {
  const selected = useMainUi((state) => state.selectedTaskId === task.id);
  const selectTask = useMainUi((state) => state.selectTask);
  const setStatus = useSetTaskStatus();
  const setTopThree = useSetTopThree();
  const done = task.status === "done";
  const due = task.dueAt ? dayjs(task.dueAt) : null;
  const overdue = !done && due !== null && due.isBefore(dayjs());

  return (
    <li
      className={
        selected
          ? "group bg-muted flex items-center gap-3 rounded-lg px-3 py-2"
          : "group hover:bg-muted flex items-center gap-3 rounded-lg px-3 py-2"
      }
    >
      <Checkbox
        aria-label={done ? `Reopen ${task.title}` : `Complete ${task.title}`}
        checked={done}
        onCheckedChange={(checked) =>
          setStatus.mutate({ id: task.id, status: checked ? "done" : "open" })
        }
      />
      <button
        type="button"
        onClick={() => selectTask(task.id)}
        className="flex min-w-0 flex-1 items-center gap-2 text-left"
      >
        <span
          aria-hidden
          className={`size-2 shrink-0 rounded-full ${categoryDot(category?.colour)}`}
        />
        <span
          className={
            done
              ? "text-caption truncate text-sm line-through"
              : "text-foreground truncate text-sm"
          }
        >
          {task.title}
        </span>
      </button>
      {due && (
        <span
          className={
            overdue
              ? "text-critical font-mono text-xs"
              : "text-caption font-mono text-xs"
          }
        >
          {due.isSame(dayjs(), "day")
            ? due.format("H:mm")
            : due.format("D MMM")}
        </span>
      )}
      {!done && (
        <button
          type="button"
          aria-label={task.isTopThree ? "Remove from top 3" : "Add to top 3"}
          aria-pressed={task.isTopThree}
          onClick={() =>
            setTopThree.mutate({ id: task.id, on: !task.isTopThree })
          }
          className={
            task.isTopThree
              ? "text-amber rounded-md p-1"
              : "text-caption rounded-md p-1 opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
          }
        >
          <Star
            className="size-4"
            fill={task.isTopThree ? "currentColor" : "none"}
          />
        </button>
      )}
    </li>
  );
}
