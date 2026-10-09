import { Link } from "@tanstack/react-router";
import { AiSettingsForm } from "../settings/AiSettingsForm";

export function Settings() {
  return (
    <main className="mx-auto flex max-w-2xl flex-col gap-8 p-10">
      <header className="flex items-baseline justify-between">
        <h1 className="font-heading text-3xl font-bold">Settings</h1>
        <Link to="/" className="text-text-secondary text-sm underline">
          Back
        </Link>
      </header>
      <section aria-labelledby="ai" className="flex flex-col gap-4">
        <h2 id="ai" className="font-heading text-xl font-semibold">
          AI provider
        </h2>
        <AiSettingsForm />
      </section>
    </main>
  );
}
