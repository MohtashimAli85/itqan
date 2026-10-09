import { describe, expect, it } from "vitest";
import type { Task } from "@/shared/bindings/bindings";
import { dayjs } from "./dayjs";
import { greeting, todayTasks } from "./today";

const now = dayjs("2026-10-10T16:30:00");

function task(overrides: Partial<Task>): Task {
  return {
    id: 1,
    title: "Task",
    notes: null,
    categoryId: null,
    kind: "output",
    priority: 0,
    dueAt: null,
    isTopThree: false,
    status: "open",
    completedAt: null,
    parentId: null,
    createdAt: now.toISOString(),
    updatedAt: now.toISOString(),
    ...overrides,
  };
}

describe("todayTasks", () => {
  it("splits top three from tasks due by the end of today", () => {
    const tasks = [
      task({ id: 1, isTopThree: true }),
      task({ id: 2, dueAt: now.hour(19).toISOString() }),
      task({ id: 3, dueAt: now.subtract(2, "day").toISOString() }),
      task({ id: 4, dueAt: now.add(1, "day").toISOString() }),
      task({ id: 5 }),
      task({ id: 6, status: "done", isTopThree: true }),
      task({ id: 7, parentId: 1, dueAt: now.toISOString() }),
    ];

    const result = todayTasks(tasks, now);

    expect(result.topThree.map((t) => t.id)).toEqual([1]);
    expect(result.due.map((t) => t.id)).toEqual([2, 3]);
  });
});

describe("greeting", () => {
  it("follows the time of day", () => {
    expect(greeting(now.hour(8))).toBe("Good morning");
    expect(greeting(now.hour(13))).toBe("Good afternoon");
    expect(greeting(now.hour(21))).toBe("Good evening");
  });
});
