import { useState, type FormEvent } from "react";
import { useMutation } from "@tanstack/react-query";
import { commands } from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { useSkills } from "@/shared/hooks/useGoals";
import { unwrap } from "@/shared/lib/result";

export function SkillsPanel() {
  const { data: skills = [] } = useSkills();
  const [name, setName] = useState("");
  const create = useMutation({
    mutationFn: async (value: string) =>
      unwrap(await commands.createSkill(value)),
    onSuccess: () => setName(""),
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    if (name.trim()) create.mutate(name.trim());
  }

  return (
    <section aria-labelledby="skills" className="flex flex-col gap-4">
      <h2 id="skills" className="font-heading text-xl font-semibold">
        Skills
      </h2>
      <ul className="flex flex-col gap-3">
        {skills.map((skill) => (
          <li key={skill.id} className="flex flex-col gap-1.5">
            <div className="flex justify-between text-sm">
              <span>{skill.name}</span>
              <span className="font-mono text-xs text-caption">
                Level {skill.level.level}
              </span>
            </div>
            <div
              role="progressbar"
              aria-label={`${skill.name} progress to the next level`}
              aria-valuemin={0}
              aria-valuemax={skill.level.xpForNext}
              aria-valuenow={skill.level.xpIntoLevel}
              className="h-1.5 overflow-hidden rounded-full bg-muted"
            >
              <div
                className="h-full origin-left rounded-full bg-brand"
                style={{
                  transform: `scaleX(${skill.level.xpIntoLevel / skill.level.xpForNext})`,
                }}
              />
            </div>
          </li>
        ))}
      </ul>
      <form onSubmit={submit} className="flex gap-2">
        <Input
          aria-label="New skill"
          placeholder="Add a skill, like Rust"
          value={name}
          onChange={(event) => setName(event.target.value)}
        />
        <Button type="submit" variant="secondary" disabled={create.isPending}>
          Add
        </Button>
      </form>
      {create.error && (
        <p role="alert" className="text-sm text-critical">
          {create.error.message}
        </p>
      )}
    </section>
  );
}
