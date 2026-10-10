import type { ModuleManifest } from "@/core/registry/types";
import { health } from "./health";
import { progress } from "./progress";
import { salah } from "./salah";
import { tasks } from "./tasks";

export const modules: ModuleManifest[] = [tasks, salah, health, progress];
