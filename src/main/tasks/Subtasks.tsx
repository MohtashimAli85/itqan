import { useState, type FormEvent } from "react";
import type { Task } from "@/shared/bindings/bindings";
import { Checkbox } from "@/shared/components/ui/checkbox";
import { Input } from "@/shared/components/ui/input";
import { useCreateTask, useSetTaskStatus } from "@/shared/hooks/useTasks";

type SubtasksProps = {
  parent: Task;
  subtasks: Task[];
};

export function Subtasks({ parent, subtasks }: SubtasksProps) {
  const [title, setTitle] = useState("");
  const create = useCreateTask();
  const setStatus = useSetTaskStatus();

  function submit(event: FormEvent) {
    event.preventDefault();
    if (!title.trim()) return;
    create.mutate(
      {
        title,
        notes: null,
        categoryId: parent.categoryId,
        kind: parent.kind,
        priority: 0,
        dueAt: null,
        parentId: parent.id,
        goalId: parent.goalId,
        skillId: parent.skillId,
      },
      { onSuccess: () => setTitle("") },
    );
  }

  return (
    <section aria-labelledby="subtasks" className="flex flex-col gap-2">
      <h3 id="subtasks" className="text-caption text-xs font-medium">
        Subtasks
      </h3>
      <ul className="flex flex-col gap-1">
        {subtasks.map((subtask) => (
          <li key={subtask.id} className="flex items-center gap-2">
            <Checkbox
              id={`subtask-${subtask.id}`}
              checked={subtask.status === "done"}
              onCheckedChange={(checked) =>
                setStatus.mutate({
                  id: subtask.id,
                  status: checked ? "done" : "open",
                })
              }
            />
            <label
              htmlFor={`subtask-${subtask.id}`}
              className={
                subtask.status === "done"
                  ? "text-caption text-sm line-through"
                  : "text-sm"
              }
            >
              {subtask.title}
            </label>
          </li>
        ))}
      </ul>
      <form onSubmit={submit}>
        <Input
          aria-label="Add a subtask"
          placeholder="Add a subtask"
          value={title}
          onChange={(event) => setTitle(event.target.value)}
        />
      </form>
    </section>
  );
}
