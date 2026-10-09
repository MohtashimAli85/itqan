import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { commands, events } from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";

const overlayKey = ["overlay"] as const;

export function useOverlaySnapshot() {
  const queryClient = useQueryClient();

  useEffect(() => {
    const unlisten = events.overlayChanged.listen((event) => {
      queryClient.setQueryData(overlayKey, event.payload);
    });
    return () => void unlisten.then((stop) => stop());
  }, [queryClient]);

  return useQuery({
    queryKey: overlayKey,
    queryFn: async () => unwrap(await commands.getOverlayState()),
    staleTime: Infinity,
  });
}
