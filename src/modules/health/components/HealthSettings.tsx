import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { commands } from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Label } from "@/shared/components/ui/label";
import { Switch } from "@/shared/components/ui/switch";
import { useHealthOverview } from "../hooks/useHealth";
import { unwrap } from "@/shared/lib/result";
import { Section } from "@/core/ui/Section";

export function HealthSettings() {
  const { data: health } = useHealthOverview();
  const [target, setTarget] = useState<number | null>(null);
  const toggle = useMutation({
    mutationFn: async (enabled: boolean) =>
      unwrap(await commands.setHealthEnabled(enabled)),
  });
  const saveTarget = useMutation({
    mutationFn: async (value: number) =>
      unwrap(await commands.setWaterTarget(value)),
    onSuccess: () => setTarget(null),
  });
  if (!health) return null;

  return (
    <Section
      id="health"
      title="Health"
      description="Stretch, eye rest and water nudges, tuned to your age. Habits only, never medical advice."
    >
      <div className="flex items-center justify-between">
        <Label htmlFor="settings-health">Health nudges</Label>
        <Switch
          id="settings-health"
          checked={health.enabled}
          onCheckedChange={(enabled) => toggle.mutate(enabled)}
        />
      </div>
      <div className="flex items-end gap-2">
        <div className="flex flex-col gap-2">
          <Label htmlFor="water-target">Glasses of water a day</Label>
          <Input
            id="water-target"
            type="number"
            min={1}
            max={20}
            className="w-28"
            value={target ?? health.waterTarget}
            onChange={(event) => setTarget(Number(event.target.value))}
          />
        </div>
        <Button
          variant="secondary"
          disabled={target === null || saveTarget.isPending}
          onClick={() => target !== null && saveTarget.mutate(target)}
        >
          Save
        </Button>
      </div>
      <p className="text-sm text-caption">
        Manage medicines from the Health tab in the panel.
      </p>
    </Section>
  );
}
