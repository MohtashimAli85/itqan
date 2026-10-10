import { useQuery } from "@tanstack/react-query";
import { commands } from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";

async function load() {
  const [profile, workDays, prayer, overlay] = await Promise.all([
    commands.getProfile(),
    commands.getWorkHours(),
    commands.getPrayerSettings(),
    commands.getOverlayState(),
  ]);
  return {
    profile: unwrap(profile),
    workDays: unwrap(workDays),
    prayer: unwrap(prayer),
    followMode: unwrap(overlay).followMode,
  };
}

export function useOnboardingData() {
  return useQuery({ queryKey: ["onboarding-data"], queryFn: load });
}
