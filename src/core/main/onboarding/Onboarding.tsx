import { useState } from "react";
import { FormProvider, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import type { PrayerSettings } from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { onboardingKey } from "../onboardingStatus";
import { saveOnboarding } from "./saveOnboarding";
import {
  defaultValues,
  onboardingSchema,
  type OnboardingValues,
} from "./schema";
import { AiStep } from "./steps/AiStep";
import { MotivationStep } from "./steps/MotivationStep";
import { NotificationsStep } from "./steps/NotificationsStep";
import { OrbStep } from "./steps/OrbStep";
import { ScheduleStep } from "./steps/ScheduleStep";
import { WelcomeStep } from "./steps/WelcomeStep";
import { useOnboardingData } from "./useOnboardingData";

const steps = [
  { Component: WelcomeStep, fields: ["name"] },
  { Component: AiStep, fields: [] },
  {
    Component: MotivationStep,
    fields: ["motivators", "coachStyle", "freeHours", "age"],
  },
  {
    Component: ScheduleStep,
    fields: [
      "workDays",
      "familyEnabled",
      "familyStart",
      "familyEnd",
      "cityId",
      "madhab",
    ],
  },
  { Component: OrbStep, fields: ["followMode"] },
  { Component: NotificationsStep, fields: [] },
] as const satisfies {
  Component: () => React.ReactNode;
  fields: readonly (keyof OnboardingValues)[];
}[];

export function Onboarding() {
  const { data, error } = useOnboardingData();
  if (error) {
    return (
      <p role="alert" className="p-10 text-critical">
        {error.message}
      </p>
    );
  }
  if (!data) return null;
  return (
    <OnboardingForm
      defaults={defaultValues(
        data.profile,
        data.workDays,
        data.prayer,
        data.followMode,
      )}
      prayer={data.prayer}
    />
  );
}

type OnboardingFormProps = {
  defaults: OnboardingValues;
  prayer: PrayerSettings;
};

function OnboardingForm({ defaults, prayer }: OnboardingFormProps) {
  const [index, setIndex] = useState(0);
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const form = useForm<OnboardingValues>({
    resolver: zodResolver(onboardingSchema),
    defaultValues: defaults,
  });
  const save = useMutation({
    mutationFn: (values: OnboardingValues) => saveOnboarding(values, prayer),
    onSuccess: async () => {
      queryClient.setQueryData(onboardingKey, true);
      await navigate({ to: "/" });
    },
  });

  const step = steps[index] ?? steps[0];
  const last = index === steps.length - 1;

  async function next() {
    if (!(await form.trigger([...step.fields]))) return;
    if (last) {
      await form.handleSubmit((values) => save.mutate(values))();
    } else {
      setIndex(index + 1);
    }
  }

  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-6">
      <FormProvider {...form}>
        <form
          className="flex w-full max-w-2xl flex-col gap-8 rounded-3xl bg-card p-10 shadow-sm ring-1 ring-border"
          onSubmit={(event) => {
            event.preventDefault();
            void next();
          }}
        >
          <p className="font-mono text-xs text-caption">
            Step {index + 1} of {steps.length}
          </p>
          <step.Component />
          {save.error && (
            <p role="alert" className="text-sm text-critical">
              {save.error.message}
            </p>
          )}
          <div className="flex justify-between">
            <Button
              type="button"
              variant="ghost"
              disabled={index === 0}
              onClick={() => setIndex(index - 1)}
            >
              Back
            </Button>
            <Button type="submit" disabled={save.isPending}>
              {last ? "Finish" : "Continue"}
            </Button>
          </div>
        </form>
      </FormProvider>
    </main>
  );
}
