import { useState, type FormEvent } from "react";
import { useMutation } from "@tanstack/react-query";
import { GlassWater, Trash2 } from "lucide-react";
import { commands } from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Label } from "@/shared/components/ui/label";
import { Switch } from "@/shared/components/ui/switch";
import { useHealthOverview } from "@/shared/hooks/useHealth";
import { dayjs } from "@/shared/lib/dayjs";
import { unwrap } from "@/shared/lib/result";

function MedicineForm() {
  const [name, setName] = useState("");
  const [time, setTime] = useState("09:00");
  const add = useMutation({
    mutationFn: async () => unwrap(await commands.addMedicine(name, [time])),
    onSuccess: () => setName(""),
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    if (name.trim()) add.mutate();
  }

  return (
    <form onSubmit={submit} className="flex flex-col gap-1.5">
      <div className="flex gap-1.5">
        <Input
          aria-label="Medicine name"
          placeholder="Add a medicine"
          value={name}
          onChange={(event) => setName(event.target.value)}
        />
        <Input
          aria-label="Daily time"
          type="time"
          className="w-28"
          value={time}
          onChange={(event) => setTime(event.target.value)}
        />
        <Button
          type="submit"
          size="sm"
          variant="secondary"
          disabled={add.isPending}
        >
          Add
        </Button>
      </div>
      {add.error && (
        <p role="alert" className="text-xs text-critical">
          {add.error.message}
        </p>
      )}
    </form>
  );
}

export function HealthTab() {
  const { data: health } = useHealthOverview();
  const toggle = useMutation({
    mutationFn: async (enabled: boolean) =>
      unwrap(await commands.setHealthEnabled(enabled)),
  });
  const drink = useMutation({
    mutationFn: async () => unwrap(await commands.logHabit("water")),
  });
  const remove = useMutation({
    mutationFn: async (id: number) => unwrap(await commands.deleteMedicine(id)),
  });

  if (!health) return null;

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <Label htmlFor="health-enabled">Health nudges</Label>
        <Switch
          id="health-enabled"
          checked={health.enabled}
          onCheckedChange={(enabled) => toggle.mutate(enabled)}
        />
      </div>

      <div className="flex items-center justify-between rounded-lg bg-muted px-3 py-2">
        <p className="text-sm">
          <span className="font-mono">
            {health.waterToday} of {health.waterTarget}
          </span>{" "}
          glasses of water
        </p>
        <Button
          size="sm"
          variant="secondary"
          disabled={drink.isPending}
          onClick={() => drink.mutate()}
        >
          <GlassWater data-icon="inline-start" />
          +1
        </Button>
      </div>

      <p className="px-1 text-xs text-caption">
        Today: {health.stretchesToday} stretch
        {health.stretchesToday === 1 ? "" : "es"}, {health.eyeRestsToday} eye
        rest{health.eyeRestsToday === 1 ? "" : "s"}
      </p>

      <section aria-labelledby="medicines" className="flex flex-col gap-2">
        <h3 id="medicines" className="px-1 text-xs font-medium text-caption">
          Medicine
        </h3>
        <ul className="flex flex-col gap-1">
          {health.medicines.map((medicine) => (
            <li
              key={medicine.habit.id}
              className="flex items-center gap-2 rounded-lg px-2 py-1 hover:bg-muted"
            >
              <span className="flex-1 truncate text-sm">
                {medicine.habit.name}
              </span>
              <span className="font-mono text-xs text-caption">
                {medicine.reminders
                  .map((reminder) => dayjs(reminder.anchorAt).format("H:mm"))
                  .join(", ")}
              </span>
              <Button
                variant="ghost"
                size="icon-xs"
                aria-label={`Remove ${medicine.habit.name}`}
                onClick={() => remove.mutate(medicine.habit.id)}
              >
                <Trash2 />
              </Button>
            </li>
          ))}
        </ul>
        <MedicineForm />
      </section>
    </div>
  );
}
