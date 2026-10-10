import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { commands, events } from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";
import { useTauriEvent } from "./useTauriEvent";

const modeKey = ["mode"] as const;

export function useModeStatus() {
  const queryClient = useQueryClient();

  useTauriEvent(events.modeChanged, (status) => {
    queryClient.setQueryData(modeKey, status);
  });

  return useQuery({
    queryKey: modeKey,
    queryFn: async () => unwrap(await commands.getModeStatus()),
    staleTime: Infinity,
  });
}

export function useStartFocus() {
  return useMutation({
    mutationFn: async (minutes: number) =>
      unwrap(await commands.startFocus(minutes, null)),
  });
}

export function useStopFocus() {
  return useMutation({
    mutationFn: async () => unwrap(await commands.stopFocus()),
  });
}
