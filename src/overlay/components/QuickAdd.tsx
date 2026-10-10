import { useEffect, useRef, useState, type FormEvent } from "react";
import { Plus } from "lucide-react";
import { Input } from "@/shared/components/ui/input";
import { useMutation } from "@tanstack/react-query";
import { commands } from "@/shared/bindings/bindings";
import { useQuickAdd } from "@/shared/hooks/useTasks";
import { unwrap } from "@/shared/lib/result";

export function QuickAdd() {
  const [text, setText] = useState("");
  const quickAdd = useQuickAdd();
  const ask = useMutation({
    mutationFn: async (question: string) =>
      unwrap(await commands.askItqan(question)),
  });
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
    if (value.endsWith("?")) {
      ask.mutate(value, { onSuccess: () => setText("") });
      return;
    }
    quickAdd.mutate(value, { onSuccess: () => setText("") });
  }

  const notice = ask.isPending
    ? "Thinking"
    : (ask.error?.message ??
      ask.data ??
      quickAdd.error?.message ??
      quickAdd.data?.notice ??
      null);

  return (
    <form onSubmit={submit} className="flex flex-col gap-1.5">
      <div className="relative">
        <Plus
          aria-hidden
          className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-caption"
        />
        <Input
          ref={input}
          aria-label="Add a task or ask a question"
          placeholder="Add a task or ask anything"
          value={text}
          onChange={(event) => setText(event.target.value)}
          className="pl-8"
        />
      </div>
      {notice && (
        <p role="status" className="px-1 text-xs text-caption">
          {notice}
        </p>
      )}
    </form>
  );
}
