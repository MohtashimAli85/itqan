import { TrendingUp } from "lucide-react";
import { NavLink } from "@/core/ui/NavLink";

export function ProgressNavItem() {
  return <NavLink to="/progress" icon={TrendingUp} label="Progress" />;
}
