import { useQuery } from "@tanstack/react-query";
import { commands } from "@/shared/bindings/bindings";

export function useAppInfo() {
  return useQuery({
    queryKey: ["app-info"],
    queryFn: async () => {
      const result = await commands.getAppInfo();
      if (result.status === "error") {
        throw new Error(result.error.message);
      }
      return result.data;
    },
  });
}
