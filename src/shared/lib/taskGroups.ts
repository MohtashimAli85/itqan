import type { Task } from "@/shared/bindings/bindings";
import { dayjs, type Dayjs } from "./dayjs";

export type TodaySections = {
  topThree: Task[];
  today: Task[];
  anytime: Task[];
};
export type DayGroup = { key: string; label: string; tasks: Task[] };

function openRoots(tasks: Task[]) {
  return tasks.filter(
    (task) => task.status === "open" && task.parentId === null,
  );
}

export function todaySections(
  tasks: Task[],
  now: Dayjs = dayjs(),
): TodaySections {
  const endOfDay = now.endOf("day");
  const open = openRoots(tasks);
  const dueToday = (task: Task) =>
    task.dueAt !== null && dayjs(task.dueAt).isSameOrBefore(endOfDay);
  return {
    topThree: open.filter((task) => task.isTopThree),
    today: open.filter((task) => !task.isTopThree && dueToday(task)),
    anytime: open.filter((task) => !task.isTopThree && task.dueAt === null),
  };
}

export function upcomingGroups(
  tasks: Task[],
  now: Dayjs = dayjs(),
  days = 14,
): DayGroup[] {
  const start = now.add(1, "day").startOf("day");
  const horizon = start.add(days, "day");
  const groups = new Map<string, DayGroup>();
  const later: Task[] = [];
  for (const task of openRoots(tasks)) {
    if (task.dueAt === null) continue;
    const due = dayjs(task.dueAt);
    if (due.isBefore(start)) continue;
    if (!due.isBefore(horizon)) {
      later.push(task);
      continue;
    }
    const key = due.format("YYYY-MM-DD");
    const label = due.isSame(start, "day")
      ? "Tomorrow"
      : due.format("dddd D MMM");
    const group = groups.get(key) ?? { key, label, tasks: [] };
    group.tasks.push(task);
    groups.set(key, group);
  }
  const sorted = [...groups.values()].sort((a, b) =>
    a.key.localeCompare(b.key),
  );
  return later.length > 0
    ? [...sorted, { key: "later", label: "Later", tasks: later }]
    : sorted;
}

export function doneToday(tasks: Task[], now: Dayjs = dayjs()): Task[] {
  const start = now.startOf("day");
  return tasks.filter(
    (task) =>
      task.status === "done" &&
      task.completedAt !== null &&
      !dayjs(task.completedAt).isBefore(start),
  );
}

const colourClasses: Record<string, string> = {
  work: "bg-work",
  personal: "bg-personal",
  health: "bg-health",
  amber: "bg-amber",
  brand: "bg-brand",
  critical: "bg-critical",
};

export function categoryDot(colour: string | undefined): string {
  return (colour && colourClasses[colour]) ?? "bg-caption";
}
