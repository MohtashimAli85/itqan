import { describe, expect, it } from "vitest";
import type { Task } from "@/shared/bindings/bindings";
import { dayjs } from "@/shared/lib/dayjs";
import { detailSchema, toInput, toValues } from "./detailForm";

const task: Task = {
  id: 7,
  title: "Review MR",
  notes: null,
  categoryId: 1,
  kind: "deepWork",
  priority: 2,
  dueAt: dayjs("2026-10-10T15:30:00").toISOString(),
  isTopThree: false,
  status: "open",
  completedAt: null,
  parentId: null,
  goalId: 3,
  skillId: null,
  createdAt: "2026-10-10T09:00:00Z",
  updatedAt: "2026-10-10T09:00:00Z",
};

describe("task detail form", () => {
  it("round trips a task through the form values", () => {
    const values = toValues(task);
    expect(values.dueDate).toBe("2026-10-10");
    expect(values.dueTime).toBe("15:30");
    expect(detailSchema.safeParse(values).success).toBe(true);

    const input = toInput({ ...values, notes: "  " }, task);
    expect(input.dueAt).toBe(task.dueAt);
    expect(input.notes).toBeNull();
    expect(input.categoryId).toBe(1);
    expect(input.goalId).toBe(3);
  });

  it("clears the due date and category", () => {
    const input = toInput(
      { ...toValues(task), dueDate: "", categoryId: "none" },
      task,
    );
    expect(input.dueAt).toBeNull();
    expect(input.categoryId).toBeNull();
  });
});
