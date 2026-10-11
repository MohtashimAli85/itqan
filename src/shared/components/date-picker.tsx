import { useState } from "react";
import { CalendarIcon } from "lucide-react";
import { cn } from "cn";
import { dayjs } from "@/shared/lib/dayjs";
import { Button } from "./ui/button";
import { Calendar } from "./ui/calendar";
import { Popover, PopoverContent, PopoverTrigger } from "./ui/popover";

type DatePickerProps = {
  value?: string;
  onChange: (value: string) => void;
  placeholder?: string;
  optional?: boolean;
  disabled?: boolean;
  min?: string;
  max?: string;
  id?: string;
  className?: string;
  "aria-label"?: string;
};

export function DatePicker({
  value,
  onChange,
  placeholder = "Pick a date",
  optional = false,
  disabled = false,
  min,
  max,
  id,
  className,
  "aria-label": ariaLabel,
}: DatePickerProps) {
  const [open, setOpen] = useState(false);
  const selected = value ? dayjs(value) : null;

  function select(next: string) {
    onChange(next);
    setOpen(false);
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          type="button"
          id={id}
          variant="outline"
          disabled={disabled}
          aria-label={ariaLabel}
          className={cn(
            "w-full justify-start font-normal",
            !selected && "text-muted-foreground",
            className,
          )}
        >
          <CalendarIcon data-icon="inline-start" />
          {selected ? selected.format("MMM D, YYYY") : placeholder}
        </Button>
      </PopoverTrigger>
      <PopoverContent align="start" className="w-auto p-3">
        <Calendar
          value={value}
          onChange={select}
          min={min}
          max={max}
          aria-label={ariaLabel ? `${ariaLabel} calendar` : "Calendar"}
        />
        {optional && (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="w-full"
            onClick={() => select("")}
          >
            Clear
          </Button>
        )}
      </PopoverContent>
    </Popover>
  );
}
