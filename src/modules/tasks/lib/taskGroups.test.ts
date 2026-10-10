import { describe, expect, it } from "vitest";
import type { Task } from "@/shared/bindings/bindings";
import { dayjs } from "@/shared/lib/dayjs";
import {
  categoryDot,
  doneToday,
  todaySections,
  upcomingGroups,
} from "./taskGroups";

const now = dayjs("2026-10-10T10:00:00");

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
    goalId: null,
    skillId: null,
    createdAt: now.toISOString(),
    updatedAt: now.toISOString(),
    ...overrides,
  };
}

describe("todaySections", () => {
  it("splits top three, due today and anytime tasks", () => {
    const sections = todaySections(
      [
        task({ id: 1, isTopThree: true }),
        task({ id: 2, dueAt: now.hour(18).toISOString() }),
        task({ id: 3 }),
        task({ id: 4, dueAt: now.add(2, "day").toISOString() }),
        task({ id: 5, parentId: 1 }),
      ],
      now,
    );
    expect(sections.topThree.map((t) => t.id)).toEqual([1]);
    expect(sections.today.map((t) => t.id)).toEqual([2]);
    expect(sections.anytime.map((t) => t.id)).toEqual([3]);
  });
});

describe("upcomingGroups", () => {
  it("groups future tasks by day and puts far ones in later", () => {
    const groups = upcomingGroups(
      [
        task({ id: 1, dueAt: now.add(1, "day").toISOString() }),
        task({ id: 2, dueAt: now.add(3, "day").toISOString() }),
        task({ id: 3, dueAt: now.add(1, "day").hour(20).toISOString() }),
        task({ id: 4, dueAt: now.add(40, "day").toISOString() }),
        task({ id: 5, dueAt: now.toISOString() }),
      ],
      now,
    );
    expect(groups.map((g) => g.label)).toEqual([
      "Tomorrow",
      "Tuesday 13 Oct",
      "Later",
    ]);
    expect(groups[0]?.tasks.map((t) => t.id)).toEqual([1, 3]);
  });
});

describe("doneToday", () => {
  it("keeps only tasks completed today", () => {
    const done = doneToday(
      [
        task({ id: 1, status: "done", completedAt: now.toISOString() }),
        task({
          id: 2,
          status: "done",
          completedAt: now.subtract(1, "day").toISOString(),
        }),
      ],
      now,
    );
    expect(done.map((t) => t.id)).toEqual([1]);
  });
});

describe("categoryDot", () => {
  it("maps colour tokens and falls back", () => {
    expect(categoryDot("work")).toBe("bg-work");
    expect(categoryDot(undefined)).toBe("bg-caption");
  });
});
