import { Controller, useFormContext, useWatch } from "react-hook-form";
import { TimePicker } from "@/shared/components/time-picker";
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
  const { control, formState } = useFormContext<OnboardingValues>();
  const workDays = useWatch({ control, name: "workDays" });
  const familyEnabled = useWatch({ control, name: "familyEnabled" });

  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col gap-1">
        <h2 className="font-heading text-2xl font-bold">Your week</h2>
        <p className="text-sm text-text-secondary">
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
            <Controller
              control={control}
              name={`workDays.${index}.start`}
              render={({ field }) => (
                <TimePicker
                  value={field.value}
                  onChange={field.onChange}
                  disabled={!day.enabled}
                  aria-label={`${weekdays[day.weekday]} start`}
                  className="w-40"
                />
              )}
            />
            <span className="text-caption">to</span>
            <Controller
              control={control}
              name={`workDays.${index}.end`}
              render={({ field }) => (
                <TimePicker
                  value={field.value}
                  onChange={field.onChange}
                  disabled={!day.enabled}
                  aria-label={`${weekdays[day.weekday]} end`}
                  className="w-40"
                />
              )}
            />
          </div>
        ))}
        {formState.errors.workDays && (
          <p role="alert" className="text-sm text-critical">
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
            <Controller
              control={control}
              name="familyStart"
              render={({ field }) => (
                <TimePicker
                  value={field.value}
                  onChange={field.onChange}
                  aria-label="Family time start"
                  className="w-40"
                />
              )}
            />
            <span className="text-caption">to</span>
            <Controller
              control={control}
              name="familyEnd"
              render={({ field }) => (
                <TimePicker
                  value={field.value}
                  onChange={field.onChange}
                  aria-label="Family time end"
                  className="w-40"
                />
              )}
            />
          </div>
        )}
        {formState.errors.familyEnd && (
          <p role="alert" className="text-sm text-critical">
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
