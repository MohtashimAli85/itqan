import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { commands, events } from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";

const healthKey = ["health"] as const;

export function useHealthOverview() {
  const queryClient = useQueryClient();

  useEffect(() => {
    const unlisten = events.healthChanged.listen(() => {
      void queryClient.invalidateQueries({ queryKey: healthKey });
    });
    return () => void unlisten.then((stop) => stop());
  }, [queryClient]);

  return useQuery({
    queryKey: healthKey,
    queryFn: async () => unwrap(await commands.getHealthOverview()),
  });
}
