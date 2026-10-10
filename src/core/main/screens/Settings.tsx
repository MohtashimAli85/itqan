import { contributions } from "@/core/registry/contributions";
import type { SettingsSection } from "@/core/registry/types";
import { AiSettingsForm } from "../settings/AiSettingsForm";
import { HealthSettings } from "../settings/HealthSettings";
import { NudgeSettings } from "../settings/NudgeSettings";
import { OrbSettings } from "../settings/OrbSettings";
import { Section } from "../settings/Section";
import { SystemSettings } from "../settings/SystemSettings";

function AiSection() {
  return (
    <Section
      id="ai"
      title="AI provider"
      description="Bring your own key, or run a model on this computer."
    >
      <AiSettingsForm />
    </Section>
  );
}

const coreSections: SettingsSection[] = [
  { id: "orb", order: 10, Component: OrbSettings },
  { id: "nudges", order: 20, Component: NudgeSettings },
  { id: "health", order: 30, Component: HealthSettings },
  { id: "ai", order: 40, Component: AiSection },
  { id: "system", order: 50, Component: SystemSettings },
];

const sections = contributions(
  (manifest) => manifest.settingsSections,
  coreSections,
);

export function Settings() {
  return (
    <main className="flex min-w-0 flex-1 flex-col gap-6 overflow-y-auto p-8">
      <h1 className="font-heading text-3xl font-bold">Settings</h1>
      <div className="flex max-w-2xl flex-col gap-6">
        {sections.map(({ id, Component }) => (
          <Component key={id} />
        ))}
      </div>
    </main>
  );
}
