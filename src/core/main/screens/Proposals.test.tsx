import { fireEvent, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { renderWithProviders } from "@/shared/test/render";
import { Proposals } from "./Proposals";

const proposal = {
  id: 4,
  kind: "core:belief",
  title: "Make 30 minutes the default focus length?",
  reason: "You finished 9 of 10 thirty minute sessions.",
  effect: "New focus sessions last 30 minutes.",
  status: "pending",
  createdAt: "2026-10-10T09:00:00Z",
  expiresAt: "2026-10-24T09:00:00Z",
};

const accept = vi.fn<(id: number) => Promise<{ status: "ok"; data: [] }>>(() =>
  Promise.resolve({ status: "ok", data: [] }),
);

vi.mock("@/shared/bindings/bindings", () => ({
  commands: {
    listProposals: () => Promise.resolve({ status: "ok", data: [proposal] }),
    acceptProposal: (id: number) => accept(id),
    rejectProposal: () => Promise.resolve({ status: "ok", data: [] }),
  },
  events: {
    proposalsChanged: { listen: () => Promise.resolve(() => undefined) },
  },
}));

describe("Proposals", () => {
  it("shows why and what changes, and accepts on yes", async () => {
    renderWithProviders(<Proposals />);

    expect(
      await screen.findByText(proposal.title, undefined, { timeout: 3000 }),
    ).toBeInTheDocument();
    expect(screen.getByText(proposal.reason)).toBeInTheDocument();
    expect(screen.getByText(proposal.effect)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Yes, change it" }));

    await waitFor(() => expect(accept).toHaveBeenCalledWith(4));
    expect(
      await screen.findByText(/Nothing to review/, undefined, {
        timeout: 3000,
      }),
    ).toBeInTheDocument();
  });
});
