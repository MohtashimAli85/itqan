import type { Task } from "@/shared/bindings/bindings";
import { Checkbox } from "@/shared/components/ui/checkbox";
import { dayjs } from "@/shared/lib/dayjs";

type TaskRowProps = {
  task: Task;
  onComplete: (task: Task) => void;
};

export function TaskRow({ task, onComplete }: TaskRowProps) {
  const checkboxId = `task-${task.id}`;
  const due = task.dueAt ? dayjs(task.dueAt) : null;
  const overdue = due?.isBefore(dayjs()) ?? false;

  return (
    <li className="hover:bg-muted flex items-center gap-2.5 rounded-lg px-2 py-1.5">
      <Checkbox id={checkboxId} onCheckedChange={() => onComplete(task)} />
      <label
        htmlFor={checkboxId}
        className="text-foreground min-w-0 flex-1 truncate text-sm"
      >
        {task.title}
      </label>
      {due && (
        <span
          className={
            overdue
              ? "text-critical font-mono text-xs"
              : "text-caption font-mono text-xs"
          }
        >
          {due.format("H:mm")}
        </span>
      )}
    </li>
  );
}
