import type { ModuleManifest } from "@/core/registry/types";
import { NextPrayerCard } from "./components/NextPrayerCard";

export const salah: ModuleManifest = {
  id: "salah",
  name: "Salah",
  description: "Prayer times, with quiet windows around each prayer.",
  sidebarWidgets: [{ id: "next-prayer", order: 10, Component: NextPrayerCard }],
};
