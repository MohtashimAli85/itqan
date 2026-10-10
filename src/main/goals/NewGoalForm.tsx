import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { z } from "zod";
import { commands } from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Label } from "@/shared/components/ui/label";
import { unwrap } from "@/shared/lib/result";

const schema = z.object({
  title: z.string().trim().min(1, "Give the goal a name").max(200),
  targetDate: z.string(),
});

type Values = z.infer<typeof schema>;

export function NewGoalForm() {
  const form = useForm<Values>({
    resolver: zodResolver(schema),
    defaultValues: { title: "", targetDate: "" },
  });
  const create = useMutation({
    mutationFn: async (values: Values) =>
      unwrap(
        await commands.createGoal({
          title: values.title,
          motivator: null,
          targetDate: values.targetDate || null,
        }),
      ),
    onSuccess: () => form.reset(),
  });

  return (
    <form
      onSubmit={form.handleSubmit((values) => create.mutate(values))}
      className="flex flex-wrap items-end gap-3"
    >
      <div className="flex min-w-64 flex-1 flex-col gap-2">
        <Label htmlFor="goal-title">New goal</Label>
        <Input
          id="goal-title"
          placeholder="Ship Itqan v1, learn Rust, run 5k"
          {...form.register("title")}
        />
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor="goal-target">Target date</Label>
        <Input id="goal-target" type="date" {...form.register("targetDate")} />
      </div>
      <Button type="submit" disabled={create.isPending}>
        Add goal
      </Button>
      {(form.formState.errors.title ?? create.error) && (
        <p role="alert" className="w-full text-sm text-critical">
          {form.formState.errors.title?.message ?? create.error?.message}
        </p>
      )}
    </form>
  );
}
