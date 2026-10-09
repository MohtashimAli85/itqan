import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  commands,
  events,
  type RewardEarned,
} from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";

const progressKey = ["progress"] as const;

export function useRewardSync(onReward?: (reward: RewardEarned) => void) {
  const queryClient = useQueryClient();
  useEffect(() => {
    const unlisten = events.rewardEarned.listen((event) => {
      void queryClient.invalidateQueries({ queryKey: progressKey });
      onReward?.(event.payload);
    });
    return () => void unlisten.then((stop) => stop());
  }, [queryClient, onReward]);
}

export function useProgress() {
  return useQuery({
    queryKey: progressKey,
    queryFn: async () => unwrap(await commands.getProgress()),
  });
}
