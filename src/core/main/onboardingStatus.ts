import type { QueryClient } from "@tanstack/react-query";
import { commands } from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";

export const onboardingKey = ["onboarded"] as const;

export function ensureOnboarded(queryClient: QueryClient) {
  return queryClient.ensureQueryData({
    queryKey: onboardingKey,
    queryFn: async () => unwrap(await commands.isOnboarded()),
  });
}
