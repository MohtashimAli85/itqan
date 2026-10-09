import { useQuery } from "@tanstack/react-query";
import { commands } from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";

export const aiKey = ["ai-status"] as const;

export function useAiStatus() {
  return useQuery({
    queryKey: aiKey,
    queryFn: async () => unwrap(await commands.getAiStatus()),
  });
}
