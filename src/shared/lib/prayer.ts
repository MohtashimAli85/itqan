import type { Prayer } from "@/shared/bindings/bindings";

const names: Record<Prayer, string> = {
  fajr: "Fajr",
  dhuhr: "Dhuhr",
  jumuah: "Jumu'ah",
  asr: "Asr",
  maghrib: "Maghrib",
  isha: "Isha",
};

export function prayerName(prayer: Prayer): string {
  return names[prayer];
}
