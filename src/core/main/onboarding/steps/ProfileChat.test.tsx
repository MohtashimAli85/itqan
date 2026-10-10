import { fireEvent, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { DraftBelief } from "@/shared/bindings/bindings";
import { renderWithProviders } from "@/shared/test/render";
import { ProfileChat, questions } from "./ProfileChat";

const drafted: DraftBelief[] = [
  {
    statement: "Getting good at Rust matters",
    kind: "motivator",
    subject: { type: "motivator", motivator: "learning" },
    strength: "high",
  },
  {
    statement: "I scroll at night",
    kind: "pattern",
    subject: { type: "note" },
    strength: "medium",
  },
];

const draftBeliefs = vi.fn<
  (answers: { question: string; answer: string }[]) => Promise<{
    status: "ok";
    data: DraftBelief[];
  }>
>(() => Promise.resolve({ status: "ok", data: drafted }));

vi.mock("@/shared/bindings/bindings", () => ({
  commands: {
    draftBeliefs: (answers: { question: string; answer: string }[]) =>
      draftBeliefs(answers),
  },
}));

describe("ProfileChat", () => {
  it("asks five questions, then lets the user trim the drafts", async () => {
    const onApply = vi.fn<(drafts: DraftBelief[]) => void>();
    renderWithProviders(<ProfileChat onApply={onApply} onSkip={() => {}} />);

    for (const [at] of questions.entries()) {
      const box = await screen.findByLabelText(questions[at] ?? "", undefined, {
        timeout: 3000,
      });
      fireEvent.change(box, { target: { value: `answer ${at + 1}` } });
      if (at < questions.length - 1) {
        fireEvent.click(screen.getByRole("button", { name: "Next" }));
      }
    }
    fireEvent.click(screen.getByRole("button", { name: "Build my profile" }));

    expect(
      await screen.findByDisplayValue("I scroll at night"),
    ).toBeInTheDocument();
    expect(draftBeliefs.mock.calls[0]?.[0]).toHaveLength(5);
    expect(draftBeliefs.mock.calls[0]?.[0][2]?.answer).toBe("answer 3");

    fireEvent.click(screen.getByRole("button", { name: "Remove belief 2" }));
    fireEvent.click(screen.getByRole("button", { name: "Use these" }));

    await waitFor(() => expect(onApply).toHaveBeenCalledWith([drafted[0]]));
  });
});
