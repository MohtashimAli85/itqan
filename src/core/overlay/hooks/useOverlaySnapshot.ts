import { useQuery, useQueryClient } from "@tanstack/react-query";
import { commands, events } from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";
import { useTauriEvent } from "@/shared/hooks/useTauriEvent";

const overlayKey = ["overlay"] as const;

export function useOverlaySnapshot() {
  const queryClient = useQueryClient();

  useTauriEvent(events.overlayChanged, (snapshot) => {
    queryClient.setQueryData(overlayKey, snapshot);
  });

  return useQuery({
    queryKey: overlayKey,
    queryFn: async () => unwrap(await commands.getOverlayState()),
    staleTime: Infinity,
  });
}
