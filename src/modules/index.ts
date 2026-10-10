import type { ModuleManifest } from "@/core/registry/types";
import { health } from "./health";
import { salah } from "./salah";

export const modules: ModuleManifest[] = [salah, health];
