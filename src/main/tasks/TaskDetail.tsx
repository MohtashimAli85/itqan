import { Controller, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { Star, Trash2, X } from "lucide-react";
import type { Task } from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Label } from "@/shared/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/shared/components/ui/select";
import {
  allTasks,
  useCategories,
  useDeleteTask,
  useSetTopThree,
  useTasks,
  useUpdateTask,
} from "@/shared/hooks/useTasks";
import { useMainUi } from "../uiStore";
import {
  detailSchema,
  kinds,
  toInput,
  toValues,
  type DetailValues,
} from "./detailForm";
import { RemindersEditor } from "./RemindersEditor";
import { Subtasks } from "./Subtasks";

const priorities = ["None", "Low", "Medium", "High"];

export function TaskDetail() {
  const selectedId = useMainUi((state) => state.selectedTaskId);
  const { data: tasks = [] } = useTasks(allTasks);
  const task = tasks.find((candidate) => candidate.id === selectedId);
  if (!task) return null;
  return (
    <DetailPanel
      key={`${task.id}-${task.updatedAt}`}
      task={task}
      subtasks={tasks.filter((candidate) => candidate.parentId === task.id)}
    />
  );
}

function DetailPanel({ task, subtasks }: { task: Task; subtasks: Task[] }) {
  const selectTask = useMainUi((state) => state.selectTask);
  const { data: categories = [] } = useCategories();
  const update = useUpdateTask();
  const remove = useDeleteTask();
  const setTopThree = useSetTopThree();
  const form = useForm<DetailValues>({
    resolver: zodResolver(detailSchema),
    defaultValues: toValues(task),
  });
  const save = form.handleSubmit((values) =>
    update.mutate({ id: task.id, input: toInput(values, task) }),
  );

  return (
    <aside
      aria-label="Task details"
      className="flex w-96 shrink-0 flex-col gap-5 overflow-y-auto border-l border-border bg-card p-5"
    >
      <div className="flex items-center justify-between">
        <Button
          variant={task.isTopThree ? "secondary" : "ghost"}
          size="sm"
          onClick={() =>
            setTopThree.mutate({ id: task.id, on: !task.isTopThree })
          }
        >
          <Star
            data-icon="inline-start"
            fill={task.isTopThree ? "currentColor" : "none"}
          />
          {task.isTopThree ? "In top 3" : "Add to top 3"}
        </Button>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Close details"
          onClick={() => selectTask(null)}
        >
          <X />
        </Button>
      </div>

      <form onSubmit={save} className="flex flex-col gap-4">
        <div className="flex flex-col gap-2">
          <Label htmlFor="detail-title">Title</Label>
          <Input id="detail-title" {...form.register("title")} />
          {form.formState.errors.title && (
            <p role="alert" className="text-xs text-critical">
              {form.formState.errors.title.message}
            </p>
          )}
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="detail-notes">Notes</Label>
          <textarea
            id="detail-notes"
            rows={4}
            className="rounded-lg border border-input bg-transparent px-3 py-2 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            {...form.register("notes")}
          />
        </div>
        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-2">
            <Label htmlFor="detail-category">Category</Label>
            <Controller
              control={form.control}
              name="categoryId"
              render={({ field }) => (
                <Select value={field.value} onValueChange={field.onChange}>
                  <SelectTrigger id="detail-category" className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">None</SelectItem>
                    {categories.map((category) => (
                      <SelectItem key={category.id} value={String(category.id)}>
                        {category.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="detail-priority">Priority</Label>
            <Controller
              control={form.control}
              name="priority"
              render={({ field }) => (
                <Select value={field.value} onValueChange={field.onChange}>
                  <SelectTrigger id="detail-priority" className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {priorities.map((label, index) => (
                      <SelectItem key={label} value={String(index)}>
                        {label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}
            />
          </div>
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="detail-kind">Kind</Label>
          <Controller
            control={form.control}
            name="kind"
            render={({ field }) => (
              <Select value={field.value} onValueChange={field.onChange}>
                <SelectTrigger id="detail-kind" className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {kinds.map((kind) => (
                    <SelectItem key={kind.id} value={kind.id}>
                      {kind.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          />
        </div>
        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-2">
            <Label htmlFor="detail-date">Due date</Label>
            <Input id="detail-date" type="date" {...form.register("dueDate")} />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="detail-time">Time</Label>
            <Input id="detail-time" type="time" {...form.register("dueTime")} />
          </div>
        </div>
        <Button
          type="submit"
          disabled={update.isPending || !form.formState.isDirty}
        >
          Save changes
        </Button>
      </form>

      {task.parentId === null && <Subtasks parent={task} subtasks={subtasks} />}
      <RemindersEditor taskId={task.id} />

      <Button
        variant="destructive"
        className="mt-auto"
        onClick={() =>
          remove.mutate(task.id, { onSuccess: () => selectTask(null) })
        }
      >
        <Trash2 data-icon="inline-start" />
        Delete task
      </Button>
    </aside>
  );
}
