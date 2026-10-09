import { Controller, useFormContext, useWatch } from "react-hook-form";
import { Input } from "@/shared/components/ui/input";
import { Label } from "@/shared/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/shared/components/ui/radio-group";
import { Slider } from "@/shared/components/ui/slider";
import { motivatorOptions, type OnboardingValues } from "../schema";

const coachStyles = [
  { id: "mentor", label: "Mentor", hint: "Supportive and patient" },
  { id: "manager", label: "Manager", hint: "Direct, keeps you on deadlines" },
  { id: "trainer", label: "Trainer", hint: "No excuses" },
] as const;

export function MotivationStep() {
  const { control, register, formState } = useFormContext<OnboardingValues>();
  const motivators = useWatch({ control, name: "motivators" });
  const total = Object.values(motivators).reduce(
    (sum, value) => sum + value,
    0,
  );

  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col gap-1">
        <h2 className="font-heading text-2xl font-bold">What drives you?</h2>
        <p className="text-text-secondary text-sm">
          This decides what I reward most and how I word nudges. You can change
          it any time.
        </p>
      </div>
      <ul className="grid grid-cols-2 gap-x-6 gap-y-4">
        {motivatorOptions.map((option) => {
          const value = motivators[option.id];
          const share = total > 0 ? Math.round((value / total) * 100) : 0;
          return (
            <li key={option.id} className="flex flex-col gap-2">
              <div className="flex justify-between text-sm">
                <Label id={`motivator-${option.id}`}>{option.label}</Label>
                <span className="text-caption font-mono">{share}%</span>
              </div>
              <Controller
                control={control}
                name={`motivators.${option.id}`}
                render={({ field }) => (
                  <Slider
                    aria-labelledby={`motivator-${option.id}`}
                    min={0}
                    max={10}
                    step={1}
                    value={[field.value]}
                    onValueChange={(next) => field.onChange(next[0] ?? 0)}
                  />
                )}
              />
            </li>
          );
        })}
      </ul>
      {formState.errors.motivators && (
        <p role="alert" className="text-critical text-sm">
          {formState.errors.motivators.message}
        </p>
      )}

      <fieldset className="flex flex-col gap-2">
        <legend className="mb-2 text-sm font-medium">Coach style</legend>
        <Controller
          control={control}
          name="coachStyle"
          render={({ field }) => (
            <RadioGroup
              value={field.value}
              onValueChange={field.onChange}
              className="grid grid-cols-3 gap-2"
            >
              {coachStyles.map((style) => (
                <Label
                  key={style.id}
                  htmlFor={`coach-${style.id}`}
                  className="has-data-[state=checked]:border-ink flex cursor-pointer flex-col items-start gap-1 rounded-lg border p-3"
                >
                  <span className="flex items-center gap-2">
                    <RadioGroupItem id={`coach-${style.id}`} value={style.id} />
                    {style.label}
                  </span>
                  <span className="text-caption text-xs font-normal">
                    {style.hint}
                  </span>
                </Label>
              ))}
            </RadioGroup>
          )}
        />
      </fieldset>

      <div className="grid grid-cols-2 gap-4">
        <div className="flex flex-col gap-2">
          <Label htmlFor="free-hours">Free hours a week for growth</Label>
          <Input
            id="free-hours"
            type="number"
            min={0}
            max={168}
            {...register("freeHours", { valueAsNumber: true })}
          />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="age">Age (optional, for health tips)</Label>
          <Input
            id="age"
            type="number"
            min={13}
            max={110}
            {...register("age", {
              setValueAs: (value: string) =>
                value === "" ? null : Number(value),
            })}
          />
        </div>
      </div>
    </div>
  );
}
