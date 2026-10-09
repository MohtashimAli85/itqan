import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { commands } from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { Label } from "@/shared/components/ui/label";
import { Slider } from "@/shared/components/ui/slider";
import { dayjs } from "@/shared/lib/dayjs";
import { unwrap } from "@/shared/lib/result";
import { Section } from "./Section";

const key = ["coach-settings"];

export function NudgeSettings() {
  const queryClient = useQueryClient();
  const { data: coach } = useQuery({
    queryKey: key,
    queryFn: async () => unwrap(await commands.getCoachSettings()),
  });
  const refresh = () => queryClient.invalidateQueries({ queryKey: key });
  const budget = useMutation({
    mutationFn: async (perHour: number) =>
      unwrap(await commands.setNudgeBudget(perHour)),
    onSuccess: refresh,
  });
  const pause = useMutation({
    mutationFn: async (minutes: number | null) =>
      unwrap(await commands.setNudgesPaused(minutes)),
    onSuccess: refresh,
  });
  if (!coach) return null;

  return (
    <Section
      id="nudges"
      title="Nudges"
      description="I stay quiet around salah, during focus, rest and family time. Critical reminders always come through."
    >
      <div className="flex flex-col gap-3">
        <div className="flex justify-between text-sm">
          <Label id="budget-label">Most nudges per hour</Label>
          <span className="text-caption font-mono">{coach.budgetPerHour}</span>
        </div>
        <Slider
          aria-labelledby="budget-label"
          min={0}
          max={12}
          step={1}
          value={[coach.budgetPerHour]}
          onValueCommit={(value) => budget.mutate(value[0] ?? 3)}
        />
      </div>
      <div className="flex items-center gap-3">
        {coach.pausedUntil ? (
          <>
            <p className="text-sm">
              Paused until{" "}
              <span className="font-mono">
                {dayjs(coach.pausedUntil).format("H:mm")}
              </span>
            </p>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => pause.mutate(null)}
            >
              Resume
            </Button>
          </>
        ) : (
          <Button
            size="sm"
            variant="secondary"
            onClick={() => pause.mutate(60)}
          >
            Pause nudges for an hour
          </Button>
        )}
      </div>
    </Section>
  );
}
