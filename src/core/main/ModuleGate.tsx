import { Navigate, type RouteComponent } from "@tanstack/react-router";
import { useModuleEnabled } from "@/core/registry/useModules";
import type { ModuleId } from "@/core/registry/types";

export function gate(id: ModuleId, Page: RouteComponent): RouteComponent {
  return function ModuleGate() {
    return useModuleEnabled(id) ? <Page /> : <Navigate to="/goals" replace />;
  };
}
