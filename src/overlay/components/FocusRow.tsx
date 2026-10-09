import { Play, Square } from "lucide-react";
import { Button } from "@/shared/components/ui/button";
import {
  useModeStatus,
  useStartFocus,
  useStopFocus,
} from "@/shared/hooks/useMode";
import { dayjs } from "@/shared/lib/dayjs";
import { prayerName } from "@/shared/lib/prayer";

const FOCUS_MINUTES = 25;

export function FocusRow() {
  const { data: status } = useModeStatus();
  const start = useStartFocus();
  const stop = useStopFocus();
  const focus = status?.focus ?? null;
  const prayer = status?.activePrayer ?? null;

  if (!focus) {
    return (
      <Button
        variant="secondary"
        size="sm"
        className="justify-start"
        disabled={start.isPending}
        onClick={() => start.mutate(FOCUS_MINUTES)}
      >
        <Play data-icon="inline-start" />
        Start a {FOCUS_MINUTES} minute focus
      </Button>
    );
  }

  return (
    <div className="bg-muted flex items-center justify-between rounded-lg px-3 py-2">
      <p className="text-sm">
        {focus.pausedForPrayer && prayer ? (
          <>Paused for {prayerName(prayer.prayer)}</>
        ) : (
          <>
            Focus until{" "}
            <span className="font-mono">
              {dayjs(focus.endsAt).format("H:mm")}
            </span>
          </>
        )}
      </p>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="Stop focus"
        disabled={stop.isPending}
        onClick={() => stop.mutate()}
      >
        <Square />
      </Button>
    </div>
  );
}
