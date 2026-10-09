import { useEffect, useRef, useState, type FormEvent } from "react";
import { Plus } from "lucide-react";
import { Input } from "@/shared/components/ui/input";
import { useQuickAdd } from "@/shared/hooks/useTasks";

export function QuickAdd() {
  const [text, setText] = useState("");
  const quickAdd = useQuickAdd();
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => {
    input.current?.focus();
    const refocus = () => input.current?.focus();
    window.addEventListener("focus", refocus);
    return () => window.removeEventListener("focus", refocus);
  }, []);

  function submit(event: FormEvent) {
    event.preventDefault();
    const value = text.trim();
    if (!value) return;
    quickAdd.mutate(value, { onSuccess: () => setText("") });
  }

  const notice = quickAdd.error?.message ?? quickAdd.data?.notice ?? null;

  return (
    <form onSubmit={submit} className="flex flex-col gap-1.5">
      <div className="relative">
        <Plus
          aria-hidden
          className="text-caption pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2"
        />
        <Input
          ref={input}
          aria-label="Add a task"
          placeholder="Add a task, like buy dahi at 7pm"
          value={text}
          onChange={(event) => setText(event.target.value)}
          className="pl-8"
        />
      </div>
      {notice && (
        <p role="status" className="text-caption px-1 text-xs">
          {notice}
        </p>
      )}
    </form>
  );
}
