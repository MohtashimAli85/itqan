import { useTasksSync } from "@/modules/tasks/hooks/useTasks";

export function TasksSync() {
  useTasksSync();
  return null;
}
