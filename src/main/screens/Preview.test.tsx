import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Preview } from "./Preview";

vi.mock("@/shared/bindings/bindings", () => ({
  commands: {
    getAppInfo: () =>
      Promise.resolve({
        status: "ok",
        data: { name: "Itqan", version: "0.1.0", schemaVersion: 1 },
      }),
  },
}));

describe("Preview", () => {
  it("shows the app info returned by the Rust core", async () => {
    render(
      <QueryClientProvider client={new QueryClient()}>
        <Preview />
      </QueryClientProvider>,
    );

    expect(
      await screen.findByText("Itqan v0.1.0, schema 1"),
    ).toBeInTheDocument();
  });

  it("has an accessible theme toggle", () => {
    render(
      <QueryClientProvider client={new QueryClient()}>
        <Preview />
      </QueryClientProvider>,
    );

    expect(
      screen.getByRole("button", { name: "Toggle theme" }),
    ).toBeInTheDocument();
  });
});
