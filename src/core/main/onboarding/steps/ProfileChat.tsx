import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Sparkles, X } from "lucide-react";
import {
  commands,
  type DraftBelief,
  type DraftSubject,
} from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { unwrap } from "@/shared/lib/result";

export const questions = [
  "What are you trying to get better at this year?",
  "What does a great day look like for you?",
  "What usually pulls you off track?",
  "When do you have the most energy?",
  "How should I talk to you when you're slacking?",
];

const labels: Record<DraftSubject["type"], string> = {
  motivator: "What drives you",
  coachStyle: "Coach style",
  note: "Noted",
};

type ProfileChatProps = {
  onApply: (drafts: DraftBelief[]) => void;
  onSkip: () => void;
};

export function ProfileChat({ onApply, onSkip }: ProfileChatProps) {
  const [answers, setAnswers] = useState(() => questions.map(() => ""));
  const [index, setIndex] = useState(0);
  const [drafts, setDrafts] = useState<DraftBelief[] | null>(null);
  const draft = useMutation({
    mutationFn: async () =>
      unwrap(
        await commands.draftBeliefs(
          questions.map((question, at) => ({
            question,
            answer: answers[at] ?? "",
          })),
        ),
      ),
    onSuccess: setDrafts,
  });

  if (drafts) {
    return (
      <div className="flex flex-col gap-4">
        <p className="text-sm text-text-secondary">
          Here's what I took from that. Change or remove anything that's off.
        </p>
        {drafts.length === 0 && (
          <p className="text-sm text-caption">
            I couldn't pick anything out. You can set it yourself instead.
          </p>
        )}
        <ul className="flex flex-col gap-2">
          {drafts.map((item, at) => (
            <li key={at} className="flex items-center gap-2">
              <span className="w-28 shrink-0 text-xs text-caption">
                {labels[item.subject.type]}
              </span>
              {item.subject.type === "note" ? (
                <Input
                  aria-label={`Belief ${at + 1}`}
                  value={item.statement}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") event.preventDefault();
                  }}
                  onChange={(event) =>
                    setDrafts(
                      drafts.map((other, position) =>
                        position === at
                          ? { ...other, statement: event.target.value }
                          : other,
                      ),
                    )
                  }
                />
              ) : (
                <span className="flex-1 text-sm">{item.statement}</span>
              )}
              <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                aria-label={`Remove belief ${at + 1}`}
                onClick={() =>
                  setDrafts(drafts.filter((_, position) => position !== at))
                }
              >
                <X />
              </Button>
            </li>
          ))}
        </ul>
        <div className="flex gap-2">
          <Button type="button" onClick={() => onApply(drafts)}>
            Use these
          </Button>
          <Button type="button" variant="ghost" onClick={onSkip}>
            Set it myself
          </Button>
        </div>
      </div>
    );
  }

  const question = questions[index] ?? "";
  const lastQuestion = index === questions.length - 1;

  return (
    <div className="flex flex-col gap-4">
      <p className="font-mono text-xs text-caption">
        Question {index + 1} of {questions.length}
      </p>
      <label htmlFor="profile-answer" className="font-heading text-lg">
        {question}
      </label>
      <textarea
        id="profile-answer"
        rows={3}
        value={answers[index] ?? ""}
        onChange={(event) =>
          setAnswers(
            answers.map((answer, at) =>
              at === index ? event.target.value : answer,
            ),
          )
        }
        className="rounded-lg border border-input bg-transparent px-3 py-2 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
      />
      {draft.error && (
        <p role="alert" className="text-sm text-critical">
          {draft.error.message}
        </p>
      )}
      <div className="flex gap-2">
        <Button
          type="button"
          variant="ghost"
          disabled={index === 0}
          onClick={() => setIndex(index - 1)}
        >
          Previous
        </Button>
        {lastQuestion ? (
          <Button
            type="button"
            disabled={draft.isPending}
            onClick={() => draft.mutate()}
          >
            <Sparkles data-icon="inline-start" />
            {draft.isPending ? "Thinking" : "Build my profile"}
          </Button>
        ) : (
          <Button type="button" onClick={() => setIndex(index + 1)}>
            Next
          </Button>
        )}
        <Button type="button" variant="ghost" onClick={onSkip}>
          Set it myself
        </Button>
      </div>
    </div>
  );
}
