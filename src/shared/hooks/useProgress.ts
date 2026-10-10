import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  commands,
  events,
  type RewardEarned,
} from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";
import { useTauriEvent } from "./useTauriEvent";

const progressKey = ["progress"] as const;

export function useRewardSync(onReward?: (reward: RewardEarned) => void) {
  const queryClient = useQueryClient();
  useTauriEvent(events.rewardEarned, (reward) => {
    void queryClient.invalidateQueries({ queryKey: progressKey });
    onReward?.(reward);
  });
}

export function useProgress() {
  return useQuery({
    queryKey: progressKey,
    queryFn: async () => unwrap(await commands.getProgress()),
  });
}
