import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  commands,
  events,
  type TaskFilter,
  type TaskInput,
  type TaskStatus,
} from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";
import { useTauriEvent } from "./useTauriEvent";

const tasksKey = ["tasks"] as const;

export const openTasks: TaskFilter = {
  status: "open",
  categoryId: null,
  dueBefore: null,
};

export function useTasksSync() {
  const queryClient = useQueryClient();
  useTauriEvent(events.tasksChanged, () => {
    void queryClient.invalidateQueries({ queryKey: tasksKey });
  });
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

export function useCategories() {
  return useQuery({
    queryKey: ["categories"],
    queryFn: async () => unwrap(await commands.listCategories()),
    staleTime: Infinity,
  });
}

export const allTasks: TaskFilter = {
  status: null,
  categoryId: null,
  dueBefore: null,
};

export function useSetTopThree() {
  return useMutation({
    mutationFn: async ({ id, on }: { id: number; on: boolean }) =>
      unwrap(await commands.setTaskTopThree(id, on)),
  });
}

export function useUpdateTask() {
  return useMutation({
    mutationFn: async ({ id, input }: { id: number; input: TaskInput }) =>
      unwrap(await commands.updateTask(id, input)),
  });
}

export function useCreateTask() {
  return useMutation({
    mutationFn: async (input: TaskInput) =>
      unwrap(await commands.createTask(input)),
  });
}

export function useDeleteTask() {
  return useMutation({
    mutationFn: async (id: number) => unwrap(await commands.deleteTask(id)),
  });
}
