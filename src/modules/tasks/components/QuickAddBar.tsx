import { useState, type FormEvent } from "react";
import { Plus } from "lucide-react";
import { Input } from "@/shared/components/ui/input";
import { useQuickAdd } from "@/modules/tasks/hooks/useTasks";

export function QuickAddBar() {
  const [text, setText] = useState("");
  const quickAdd = useQuickAdd();

  function submit(event: FormEvent) {
    event.preventDefault();
    const value = text.trim();
    if (value) quickAdd.mutate(value, { onSuccess: () => setText("") });
  }

  return (
    <form onSubmit={submit} className="flex flex-col gap-1">
      <div className="relative">
        <Plus
          aria-hidden
          className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-caption"
        />
        <Input
          aria-label="Add a task"
          placeholder="Add a task: review MR at 3pm #work !"
          value={text}
          onChange={(event) => setText(event.target.value)}
          className="h-10 pl-9"
        />
      </div>
      {quickAdd.data?.notice && (
        <p role="status" className="px-1 text-xs text-caption">
          {quickAdd.data.notice}
        </p>
      )}
    </form>
  );
}
