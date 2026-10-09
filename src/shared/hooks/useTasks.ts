import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  commands,
  events,
  type TaskFilter,
  type TaskStatus,
} from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";

const tasksKey = ["tasks"] as const;

export const openTasks: TaskFilter = {
  status: "open",
  categoryId: null,
  dueBefore: null,
};

export function useTasksSync() {
  const queryClient = useQueryClient();
  useEffect(() => {
    const unlisten = events.tasksChanged.listen(() => {
      void queryClient.invalidateQueries({ queryKey: tasksKey });
    });
    return () => void unlisten.then((stop) => stop());
  }, [queryClient]);
}

export function useTasks(filter: TaskFilter) {
  return useQuery({
    queryKey: [...tasksKey, filter],
    queryFn: async () => unwrap(await commands.listTasks(filter)),
  });
}

export function useSetTaskStatus() {
  return useMutation({
    mutationFn: async ({ id, status }: { id: number; status: TaskStatus }) =>
      unwrap(await commands.setTaskStatus(id, status)),
  });
}

export function useQuickAdd() {
  return useMutation({
    mutationFn: async (text: string) =>
      unwrap(await commands.quickAddTask(text)),
  });
}
