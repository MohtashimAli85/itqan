import type { ReactNode } from "react";
import { FormProvider, useForm, useWatch } from "react-hook-form";
import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { AiStatus } from "@/shared/bindings/bindings";
import { renderWithProviders } from "@/shared/test/render";
import type { OnboardingValues } from "../schema";
import { MotivationStep } from "./MotivationStep";

let ready = true;

vi.mock("@/shared/hooks/useAi", () => ({
  useAiStatus: () => ({
    data: {
      settings: { enabled: ready },
      hasKey: ready,
      needsKey: true,
    } as unknown as AiStatus,
  }),
}));

const base: OnboardingValues = {
  name: "",
  motivators: {
    learning: 4,
    building: 4,
    health: 3,
    money: 0,
    recognition: 0,
    family: 0,
    freedom: 0,
    status: 0,
  },
  coachStyle: "mentor",
  freeHours: 8,
  age: null,
  workDays: [],
  familyEnabled: false,
  familyStart: "19:00",
  familyEnd: "21:00",
  cityId: "",
  madhab: "hanafi",
  followMode: "follow",
  beliefNotes: [],
  chat: "idle",
};

class NoResize {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver ??= NoResize;

function ChatState() {
  const chat = useWatch<OnboardingValues, "chat">({ name: "chat" });
  return <output aria-label="chat state">{chat}</output>;
}

function Harness({
  values,
  children,
}: {
  values: OnboardingValues;
  children: ReactNode;
}) {
  const methods = useForm<OnboardingValues>({ defaultValues: values });
  return (
    <FormProvider {...methods}>
      {children}
      <ChatState />
    </FormProvider>
  );
}

describe("MotivationStep", () => {
  it("opens the questions when AI is ready", async () => {
    ready = true;
    renderWithProviders(
      <Harness values={base}>
        <MotivationStep />
      </Harness>,
    );
    expect(
      await screen.findByText(
        /What are you trying to get better at/,
        undefined,
        {
          timeout: 3000,
        },
      ),
    ).toBeInTheDocument();
  });

  it("falls back to the sliders and unblocks Continue when AI goes away", async () => {
    ready = false;
    renderWithProviders(
      <Harness values={{ ...base, chat: "open" }}>
        <MotivationStep />
      </Harness>,
    );
    expect(
      await screen.findByText("Learning", undefined, { timeout: 3000 }),
    ).toBeInTheDocument();
    await waitFor(() =>
      expect(screen.getByLabelText("chat state")).toHaveTextContent("idle"),
    );
  });
});
