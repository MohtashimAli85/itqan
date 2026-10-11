import { useMemo } from "react";
import { cn } from "cn";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";

const pad = (value: number) => String(value).padStart(2, "0");

type TimePickerProps = {
  value?: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  id?: string;
  className?: string;
  "aria-label"?: string;
};

export function TimePicker({
  value,
  onChange,
  disabled = false,
  id,
  className,
  "aria-label": ariaLabel = "Time",
}: TimePickerProps) {
  const [hour = "", minute = ""] = (value ?? "").split(":");
  const hours = useMemo(
    () => Array.from({ length: 24 }, (_, index) => pad(index)),
    [],
  );
  const minutes = useMemo(() => {
    const base = Array.from({ length: 12 }, (_, index) => pad(index * 5));
    if (minute !== "" && !base.includes(minute)) base.push(minute);
    return base.sort((a, b) => Number(a) - Number(b));
  }, [minute]);

  return (
    <div className={cn("flex gap-2", className)}>
      <Select
        value={hour}
        onValueChange={(next) => onChange(`${next}:${minute || "00"}`)}
        disabled={disabled}
      >
        <SelectTrigger
          id={id}
          aria-label={`${ariaLabel} hour`}
          className="flex-1"
        >
          <SelectValue placeholder="HH" />
        </SelectTrigger>
        <SelectContent>
          {hours.map((item) => (
            <SelectItem key={item} value={item}>
              {item}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Select
        value={minute}
        onValueChange={(next) => onChange(`${hour || "00"}:${next}`)}
        disabled={disabled}
      >
        <SelectTrigger aria-label={`${ariaLabel} minute`} className="flex-1">
          <SelectValue placeholder="MM" />
        </SelectTrigger>
        <SelectContent>
          {minutes.map((item) => (
            <SelectItem key={item} value={item}>
              {item}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
