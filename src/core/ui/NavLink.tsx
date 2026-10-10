import { Link, type LinkProps } from "@tanstack/react-router";
import type { LucideIcon } from "lucide-react";

export const navLinkClass =
  "flex items-center gap-2.5 rounded-lg px-3 py-1.5 text-sm text-text-secondary hover:bg-muted hover:text-foreground data-[status=active]:bg-muted data-[status=active]:font-medium data-[status=active]:text-foreground";

export function NavLink({
  to,
  icon: Icon,
  label,
  count,
}: {
  to: LinkProps["to"];
  icon: LucideIcon;
  label: string;
  count?: number;
}) {
  return (
    <Link to={to} className={navLinkClass} activeOptions={{ exact: true }}>
      <Icon aria-hidden className="size-4" />
      <span className="flex-1">{label}</span>
      {count !== undefined && count > 0 && (
        <span className="font-mono text-xs text-caption">{count}</span>
      )}
    </Link>
  );
}
