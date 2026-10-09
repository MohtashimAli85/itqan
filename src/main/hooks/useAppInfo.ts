import { useQuery } from "@tanstack/react-query";
import { commands } from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";

export function useAppInfo() {
  return useQuery({
    queryKey: ["app-info"],
    queryFn: async () => unwrap(await commands.getAppInfo()),
  });
}
