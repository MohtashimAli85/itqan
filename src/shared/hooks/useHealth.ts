import { useQuery, useQueryClient } from "@tanstack/react-query";
import { commands, events } from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";
import { useTauriEvent } from "./useTauriEvent";

const healthKey = ["health"] as const;

export function useHealthOverview() {
  const queryClient = useQueryClient();

  useTauriEvent(events.healthChanged, () => {
    void queryClient.invalidateQueries({ queryKey: healthKey });
  });

  return useQuery({
    queryKey: healthKey,
    queryFn: async () => unwrap(await commands.getHealthOverview()),
  });
}
