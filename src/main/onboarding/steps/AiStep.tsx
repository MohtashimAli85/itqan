import { AiSettingsForm } from "../../settings/AiSettingsForm";

export function AiStep() {
  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col gap-1">
        <h2 className="font-heading text-2xl font-bold">
          Bring your own AI (optional)
        </h2>
        <p className="text-sm text-text-secondary">
          AI helps me plan goals and answer questions. Use a provider key or a
          model running on this computer. Without it, I still work with simple
          rules. You can set this up later.
        </p>
      </div>
      <AiSettingsForm />
    </div>
  );
}
