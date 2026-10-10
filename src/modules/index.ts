import type { ModuleManifest } from "@/core/registry/types";
import { health } from "./health";
import { salah } from "./salah";
import { tasks } from "./tasks";

export const modules: ModuleManifest[] = [tasks, salah, health];
