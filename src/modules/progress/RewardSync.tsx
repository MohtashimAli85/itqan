import { useRewardSync } from "./hooks/useProgress";
import { celebrateReward } from "./rewardEffects";

export function RewardSync() {
  useRewardSync();
  return null;
}

export function RewardCelebrations() {
  useRewardSync(celebrateReward);
  return null;
}
