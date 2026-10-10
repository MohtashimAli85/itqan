import { z } from "zod";
import type { Task, TaskInput, TaskKind } from "@/shared/bindings/bindings";
import { dayjs } from "@/shared/lib/dayjs";

export const kinds: { id: TaskKind; label: string }[] = [
  { id: "output", label: "Output (ship something)" },
  { id: "deepWork", label: "Deep work" },
  { id: "learning", label: "Learning" },
  { id: "habit", label: "Habit" },
];

export const detailSchema = z.object({
  title: z.string().trim().min(1, "A task needs a title").max(500),
  notes: z.string().max(5000),
  categoryId: z.string(),
  kind: z.enum(["output", "deepWork", "learning", "habit"]),
  priority: z.string(),
  dueDate: z.string(),
  dueTime: z.string(),
});

export type DetailValues = z.infer<typeof detailSchema>;

export function toValues(task: Task): DetailValues {
  const due = task.dueAt ? dayjs(task.dueAt) : null;
  return {
    title: task.title,
    notes: task.notes ?? "",
    categoryId: task.categoryId === null ? "none" : String(task.categoryId),
    kind: task.kind,
    priority: String(task.priority),
    dueDate: due ? due.format("YYYY-MM-DD") : "",
    dueTime: due ? due.format("HH:mm") : "",
  };
}

export function toInput(values: DetailValues, task: Task): TaskInput {
  const dueAt = values.dueDate
    ? dayjs(`${values.dueDate}T${values.dueTime || "09:00"}`).toISOString()
    : null;
  return {
    title: values.title,
    notes: values.notes.trim() || null,
    categoryId: values.categoryId === "none" ? null : Number(values.categoryId),
    kind: values.kind,
    priority: Number(values.priority),
    dueAt,
    parentId: task.parentId,
    goalId: task.goalId,
    skillId: task.skillId,
  };
}

export const repeats = [
  { id: "none", label: "Once", rule: null },
  { id: "daily", label: "Every day", rule: "FREQ=DAILY" },
  {
    id: "weekdays",
    label: "Weekdays",
    rule: "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR",
  },
  { id: "weekly", label: "Every week", rule: "FREQ=WEEKLY" },
] as const;
