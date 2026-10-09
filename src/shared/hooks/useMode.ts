import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { commands, events } from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";

const modeKey = ["mode"] as const;

export function useModeStatus() {
  const queryClient = useQueryClient();

  useEffect(() => {
    const unlisten = events.modeChanged.listen((event) => {
      queryClient.setQueryData(modeKey, event.payload);
    });
    return () => void unlisten.then((stop) => stop());
  }, [queryClient]);

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
