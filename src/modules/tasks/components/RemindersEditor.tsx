import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { BellRing, Trash2 } from "lucide-react";
import { commands } from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Label } from "@/shared/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/shared/components/ui/select";
import { Switch } from "@/shared/components/ui/switch";
import { dayjs } from "@/shared/lib/dayjs";
import { unwrap } from "@/shared/lib/result";
import { repeats } from "./detailForm";

export function RemindersEditor({ taskId }: { taskId: number }) {
  const queryClient = useQueryClient();
  const key = ["reminders", taskId];
  const { data: reminders = [] } = useQuery({
    queryKey: key,
    queryFn: async () => unwrap(await commands.listTaskReminders(taskId)),
  });
  const [at, setAt] = useState(() =>
    dayjs().add(1, "hour").format("YYYY-MM-DDTHH:mm"),
  );
  const [repeat, setRepeat] = useState<(typeof repeats)[number]["id"]>("none");
  const [critical, setCritical] = useState(false);
  const refresh = () => queryClient.invalidateQueries({ queryKey: key });
  const add = useMutation({
    mutationFn: async () =>
      unwrap(
        await commands.createReminder({
          taskId,
          title: null,
          at: dayjs(at).toISOString(),
          rrule: repeats.find((option) => option.id === repeat)?.rule ?? null,
          critical,
        }),
      ),
    onSuccess: refresh,
  });
  const remove = useMutation({
    mutationFn: async (id: number) => unwrap(await commands.deleteReminder(id)),
    onSuccess: refresh,
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    add.mutate();
  }

  return (
    <section aria-labelledby="reminders" className="flex flex-col gap-2">
      <h3 id="reminders" className="text-xs font-medium text-caption">
        Reminders
      </h3>
      <ul className="flex flex-col gap-1">
        {reminders.map((reminder) => (
          <li key={reminder.id} className="flex items-center gap-2 text-sm">
            <BellRing
              aria-hidden
              className={
                reminder.critical
                  ? "size-3.5 text-critical"
                  : "size-3.5 text-caption"
              }
            />
            <span className="flex-1">
              {reminder.nextAt
                ? dayjs(reminder.nextAt).format("ddd D MMM, H:mm")
                : "Done"}
              {reminder.rrule && (
                <span className="text-caption"> · repeats</span>
              )}
            </span>
            <Button
              variant="ghost"
              size="icon-xs"
              aria-label="Delete reminder"
              onClick={() => remove.mutate(reminder.id)}
            >
              <Trash2 />
            </Button>
          </li>
        ))}
      </ul>
      <form onSubmit={submit} className="flex flex-col gap-2">
        <div className="flex gap-2">
          <Input
            type="datetime-local"
            aria-label="Remind at"
            value={at}
            onChange={(event) => setAt(event.target.value)}
          />
          <Select
            value={repeat}
            onValueChange={(value) => setRepeat(value as typeof repeat)}
          >
            <SelectTrigger aria-label="Repeat" className="w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {repeats.map((option) => (
                <SelectItem key={option.id} value={option.id}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Switch
              id="critical"
              checked={critical}
              onCheckedChange={setCritical}
            />
            <Label htmlFor="critical">Critical (breaks through focus)</Label>
          </div>
          <Button
            type="submit"
            size="sm"
            variant="secondary"
            disabled={add.isPending}
          >
            Add reminder
          </Button>
        </div>
      </form>
    </section>
  );
}
