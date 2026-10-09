import { Controller, useFormContext, useWatch } from "react-hook-form";
import { Input } from "@/shared/components/ui/input";
import { Label } from "@/shared/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/shared/components/ui/radio-group";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/shared/components/ui/select";
import { Switch } from "@/shared/components/ui/switch";
import { cities } from "@/shared/lib/cities";
import type { OnboardingValues } from "../schema";

const weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

export function ScheduleStep() {
  const { control, register, formState } = useFormContext<OnboardingValues>();
  const workDays = useWatch({ control, name: "workDays" });
  const familyEnabled = useWatch({ control, name: "familyEnabled" });

  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col gap-1">
        <h2 className="font-heading text-2xl font-bold">Your week</h2>
        <p className="text-text-secondary text-sm">
          I stay quiet outside these hours, around salah and during family time.
        </p>
      </div>

      <fieldset className="flex flex-col gap-2">
        <legend className="mb-2 text-sm font-medium">Work hours</legend>
        {workDays.map((day, index) => (
          <div key={day.weekday} className="flex items-center gap-3">
            <Controller
              control={control}
              name={`workDays.${index}.enabled`}
              render={({ field }) => (
                <Switch
                  id={`day-${day.weekday}`}
                  checked={field.value}
                  onCheckedChange={field.onChange}
                />
              )}
            />
            <Label htmlFor={`day-${day.weekday}`} className="w-10">
              {weekdays[day.weekday]}
            </Label>
            <Input
              type="time"
              aria-label={`${weekdays[day.weekday]} start`}
              disabled={!day.enabled}
              className="w-32"
              {...register(`workDays.${index}.start`)}
            />
            <span className="text-caption">to</span>
            <Input
              type="time"
              aria-label={`${weekdays[day.weekday]} end`}
              disabled={!day.enabled}
              className="w-32"
              {...register(`workDays.${index}.end`)}
            />
          </div>
        ))}
        {formState.errors.workDays && (
          <p role="alert" className="text-critical text-sm">
            {formState.errors.workDays.message}
          </p>
        )}
      </fieldset>

      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-3">
          <Controller
            control={control}
            name="familyEnabled"
            render={({ field }) => (
              <Switch
                id="family"
                checked={field.value}
                onCheckedChange={field.onChange}
              />
            )}
          />
          <Label htmlFor="family">Protect family time every day</Label>
        </div>
        {familyEnabled && (
          <div className="flex items-center gap-3 pl-12">
            <Input
              type="time"
              aria-label="Family time start"
              className="w-32"
              {...register("familyStart")}
            />
            <span className="text-caption">to</span>
            <Input
              type="time"
              aria-label="Family time end"
              className="w-32"
              {...register("familyEnd")}
            />
          </div>
        )}
        {formState.errors.familyEnd && (
          <p role="alert" className="text-critical text-sm">
            {formState.errors.familyEnd.message}
          </p>
        )}
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div className="flex flex-col gap-2">
          <Label htmlFor="city">City for prayer times</Label>
          <Controller
            control={control}
            name="cityId"
            render={({ field }) => (
              <Select value={field.value} onValueChange={field.onChange}>
                <SelectTrigger id="city" className="w-full">
                  <SelectValue placeholder="Skip prayer times" />
                </SelectTrigger>
                <SelectContent>
                  {cities.map((city) => (
                    <SelectItem key={city.id} value={city.id}>
                      {city.name}, {city.country}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          />
        </div>
        <fieldset className="flex flex-col gap-2">
          <legend className="mb-2 text-sm font-medium">Asr</legend>
          <Controller
            control={control}
            name="madhab"
            render={({ field }) => (
              <RadioGroup
                value={field.value}
                onValueChange={field.onChange}
                className="flex gap-4"
              >
                <Label htmlFor="madhab-hanafi" className="gap-2">
                  <RadioGroupItem id="madhab-hanafi" value="hanafi" />
                  Hanafi
                </Label>
                <Label htmlFor="madhab-shafi" className="gap-2">
                  <RadioGroupItem id="madhab-shafi" value="shafi" />
                  Shafi'i and others
                </Label>
              </RadioGroup>
            )}
          />
        </fieldset>
      </div>
    </div>
  );
}
