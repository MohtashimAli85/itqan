import { AiSettingsForm } from "../settings/AiSettingsForm";
import { HealthSettings } from "../settings/HealthSettings";
import { NudgeSettings } from "../settings/NudgeSettings";
import { OrbSettings } from "../settings/OrbSettings";
import { Section } from "../settings/Section";
import { SystemSettings } from "../settings/SystemSettings";

export function Settings() {
  return (
    <main className="flex min-w-0 flex-1 flex-col gap-6 overflow-y-auto p-8">
      <h1 className="font-heading text-3xl font-bold">Settings</h1>
      <div className="flex max-w-2xl flex-col gap-6">
        <OrbSettings />
        <NudgeSettings />
        <HealthSettings />
        <Section
          id="ai"
          title="AI provider"
          description="Bring your own key, or run a model on this computer."
        >
          <AiSettingsForm />
        </Section>
        <SystemSettings />
      </div>
    </main>
  );
}
