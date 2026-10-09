import { commands, type PrayerSettings } from "@/shared/bindings/bindings";
import { cityById } from "@/shared/lib/cities";
import { unwrap } from "@/shared/lib/result";
import {
  toPrayerSettings,
  toProfile,
  toWorkDays,
  type OnboardingValues,
} from "./schema";

export async function saveOnboarding(
  values: OnboardingValues,
  prayer: PrayerSettings,
) {
  unwrap(await commands.saveProfile(toProfile(values)));
  unwrap(await commands.setWorkHours(toWorkDays(values)));
  const city = cityById(values.cityId);
  if (city) unwrap(await commands.setTimezone(city.timezone));
  const settings = toPrayerSettings(values, prayer);
  if (settings) unwrap(await commands.setPrayerSettings(settings));
  unwrap(await commands.setFollowMode(values.followMode));
  unwrap(await commands.completeOnboarding());
}
