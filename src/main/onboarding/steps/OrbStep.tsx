import { Controller, useFormContext } from "react-hook-form";
import { Label } from "@/shared/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/shared/components/ui/radio-group";
import type { OnboardingValues } from "../schema";

const placements = [
  {
    id: "follow",
    label: "Follow my cursor",
    hint: "Stays beside you like a buddy",
  },
  {
    id: "corner",
    label: "Sit in a corner",
    hint: "Bottom right, out of the way",
  },
  {
    id: "hidden",
    label: "Stay hidden",
    hint: "Reminders arrive as notifications",
  },
] as const;

export function OrbStep() {
  const { control } = useFormContext<OnboardingValues>();
  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col gap-1">
        <h2 className="font-heading text-2xl font-bold">Where should I sit?</h2>
        <p className="text-text-secondary text-sm">
          You can switch any time from the menu bar.
        </p>
      </div>
      <Controller
        control={control}
        name="followMode"
        render={({ field }) => (
          <RadioGroup
            value={field.value}
            onValueChange={field.onChange}
            className="flex flex-col gap-2"
          >
            {placements.map((placement) => (
              <Label
                key={placement.id}
                htmlFor={`orb-${placement.id}`}
                className="has-data-[state=checked]:border-ink flex cursor-pointer items-center gap-3 rounded-lg border p-4"
              >
                <RadioGroupItem
                  id={`orb-${placement.id}`}
                  value={placement.id}
                />
                <span className="flex flex-col gap-0.5">
                  <span>{placement.label}</span>
                  <span className="text-caption text-xs font-normal">
                    {placement.hint}
                  </span>
                </span>
              </Label>
            ))}
          </RadioGroup>
        )}
      />
    </div>
  );
}
