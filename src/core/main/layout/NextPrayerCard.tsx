import { useModeStatus } from "@/shared/hooks/useMode";
import { dayjs } from "@/shared/lib/dayjs";
import { prayerName } from "@/shared/lib/prayer";

export function NextPrayerCard() {
  const { data: status } = useModeStatus();
  const next = status?.nextPrayer;
  if (!next) return null;
  const at = dayjs(next.at);

  return (
    <div className="flex flex-col gap-0.5 rounded-xl bg-background p-3 ring-1 ring-border">
      <span className="text-xs text-caption">Next prayer</span>
      <span className="font-heading text-base font-semibold">
        {prayerName(next.prayer)}{" "}
        <span className="font-mono text-sm font-normal">
          {at.format("H:mm")}
        </span>
      </span>
      <span className="text-xs text-caption">{at.fromNow()}</span>
    </div>
  );
}
