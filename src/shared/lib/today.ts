import type { Task } from "@/shared/bindings/bindings";
import { dayjs, type Dayjs } from "./dayjs";

export type TodayTasks = { topThree: Task[]; due: Task[] };

export function todayTasks(tasks: Task[], now: Dayjs = dayjs()): TodayTasks {
  const endOfDay = now.endOf("day");
  const open = tasks.filter(
    (task) => task.status === "open" && task.parentId === null,
  );
  return {
    topThree: open.filter((task) => task.isTopThree),
    due: open.filter(
      (task) =>
        !task.isTopThree &&
        task.dueAt !== null &&
        dayjs(task.dueAt).isSameOrBefore(endOfDay),
    ),
  };
}

export function greeting(now: Dayjs = dayjs()): string {
  const hour = now.hour();
  if (hour < 12) return "Good morning";
  if (hour < 17) return "Good afternoon";
  return "Good evening";
}
