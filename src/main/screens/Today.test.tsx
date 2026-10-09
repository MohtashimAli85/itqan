import { screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { renderWithProviders } from "@/shared/test/render";
import { Today } from "./Today";

const now = new Date().toISOString();

vi.mock("@/shared/bindings/bindings", () => ({
  commands: {
    listTasks: () =>
      Promise.resolve({
        status: "ok",
        data: [
          {
            id: 1,
            title: "Ship the parser",
            notes: null,
            categoryId: 5,
            kind: "output",
            priority: 0,
            dueAt: null,
            isTopThree: true,
            status: "open",
            completedAt: null,
            parentId: null,
            goalId: null,
            skillId: null,
            createdAt: now,
            updatedAt: now,
          },
        ],
      }),
    listCategories: () =>
      Promise.resolve({
        status: "ok",
        data: [
          {
            id: 5,
            name: "Building",
            colour: "brand",
            icon: "hammer",
            builtin: true,
          },
        ],
      }),
  },
}));

describe("Today", () => {
  it("shows top three tasks with an accessible checkbox and star", async () => {
    renderWithProviders(<Today />);

    expect(await screen.findByText("Ship the parser")).toBeInTheDocument();
    expect(
      screen.getByRole("checkbox", { name: "Complete Ship the parser" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Remove from top 3" }),
    ).toHaveAttribute("aria-pressed", "true");
  });
});
