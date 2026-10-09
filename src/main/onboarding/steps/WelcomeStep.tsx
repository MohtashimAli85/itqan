import { useFormContext } from "react-hook-form";
import { Input } from "@/shared/components/ui/input";
import { Label } from "@/shared/components/ui/label";
import type { OnboardingValues } from "../schema";

export function WelcomeStep() {
  const { register } = useFormContext<OnboardingValues>();
  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col gap-2">
        <h1 className="font-heading text-3xl font-bold">Assalamu alaikum</h1>
        <p className="text-text-secondary">
          I'm Itqan. I sit on your screen, keep your tasks and reminders, and
          nudge you toward the skills and projects you care about. Everything
          stays on this computer.
        </p>
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor="name">What should I call you?</Label>
        <Input id="name" autoComplete="given-name" {...register("name")} />
      </div>
    </div>
  );
}
