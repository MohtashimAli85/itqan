import { cn } from "cn";
import { DatePicker } from "./date-picker";
import { TimePicker } from "./time-picker";

type DateTimePickerProps = {
  value?: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  className?: string;
  "aria-label"?: string;
};

export function DateTimePicker({
  value,
  onChange,
  disabled = false,
  className,
  "aria-label": ariaLabel = "Date and time",
}: DateTimePickerProps) {
  const [date = "", time = ""] = (value ?? "").split("T");

  return (
    <div className={cn("flex gap-2", className)}>
      <DatePicker
        value={date}
        onChange={(next) => onChange(`${next}T${time || "00:00"}`)}
        disabled={disabled}
        aria-label={`${ariaLabel} date`}
        className="flex-1"
      />
      <TimePicker
        value={time}
        onChange={(next) => onChange(`${date}T${next}`)}
        disabled={disabled || !date}
        aria-label={`${ariaLabel} time`}
        className="shrink-0"
      />
    </div>
  );
}
