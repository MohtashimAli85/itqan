import { Settings, Target } from "lucide-react";
import { useContributions } from "@/core/registry/contributions";
import type { Contribution } from "@/core/registry/types";
import { NavLink } from "@/core/ui/NavLink";

const GoalsLink = () => <NavLink to="/goals" icon={Target} label="Goals" />;
const SettingsLink = () => (
  <NavLink to="/settings" icon={Settings} label="Settings" />
);

const coreItems: Contribution[] = [
  { id: "goals", order: 10, Component: GoalsLink },
  { id: "settings", order: 90, Component: SettingsLink },
];

function CoreNav() {
  const items = useContributions((manifest) => manifest.navItems, coreItems);
  return (
    <div className="flex flex-col gap-0.5">
      {items.map(({ id, Component }) => (
        <Component key={id} />
      ))}
    </div>
  );
}

const coreSections: Contribution[] = [
  { id: "core", order: 50, Component: CoreNav },
];

export function Sidebar() {
  const sections = useContributions(
    (manifest) => manifest.sidebarSections,
    coreSections,
  );
  const widgets = useContributions((manifest) => manifest.sidebarWidgets, []);

  return (
    <nav
      aria-label="Main"
      className="flex w-56 shrink-0 flex-col gap-6 border-r border-sidebar-border bg-sidebar p-4"
    >
      <p className="px-3 font-heading text-lg font-bold">Itqan</p>
      {sections.map(({ id, Component }) => (
        <Component key={id} />
      ))}
      <div className="mt-auto flex flex-col gap-3">
        {widgets.map(({ id, Component }) => (
          <Component key={id} />
        ))}
      </div>
    </nav>
  );
}
