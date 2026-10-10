import type { ModuleManifest } from "@/core/registry/types";
import { NextPrayerCard } from "./components/NextPrayerCard";

export const salah: ModuleManifest = {
  id: "salah",
  sidebarWidgets: [{ id: "next-prayer", order: 10, Component: NextPrayerCard }],
};
