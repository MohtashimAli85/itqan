import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { commands, events } from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";

export const goalsKey = ["goals"] as const;

export function useGoalsSync() {
  const queryClient = useQueryClient();
  useEffect(() => {
    const unlisten = events.goalsChanged.listen(() => {
      void queryClient.invalidateQueries({ queryKey: goalsKey });
    });
    return () => void unlisten.then((stop) => stop());
  }, [queryClient]);
}

export function useGoals() {
  return useQuery({
    queryKey: [...goalsKey, "list"],
    queryFn: async () => unwrap(await commands.listGoals()),
  });
}

export function useMilestones(goalId: number) {
  return useQuery({
    queryKey: [...goalsKey, "milestones", goalId],
    queryFn: async () => unwrap(await commands.listMilestones(goalId)),
  });
}

export function useSkills() {
  return useQuery({
    queryKey: [...goalsKey, "skills"],
    queryFn: async () => unwrap(await commands.listSkills()),
  });
}
