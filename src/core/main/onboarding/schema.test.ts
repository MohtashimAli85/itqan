import { describe, expect, it } from "vitest";
import type {
  PrayerSettings,
  Profile,
  WorkDay,
} from "@/shared/bindings/bindings";
import {
  applyDrafts,
  defaultValues,
  onboardingSchema,
  toPrayerSettings,
  toProfile,
  toWorkDays,
} from "./schema";

const profile: Profile = {
  name: null,
  motivators: [],
  situation: null,
  freeHoursPerWeek: null,
  age: null,
  coachStyle: "mentor",
  familyStartMinute: null,
  familyEndMinute: null,
  updatedAt: null,
};

const workDays: WorkDay[] = [0, 1, 2, 3, 4, 5, 6].map((weekday) => ({
  weekday,
  enabled: weekday < 5,
  startMinute: 540,
  endMinute: 1020,
}));

const prayer: PrayerSettings = {
  enabled: false,
  city: null,
  latitude: null,
  longitude: null,
  method: "karachi",
  madhab: "hanafi",
  highLatitudeRule: "middleOfTheNight",
  pauseBeforeMinutes: 5,
  pauseAfterMinutes: 20,
  jumuahBreak: true,
};

describe("onboarding schema", () => {
  it("starts from sensible defaults that validate", () => {
    const values = defaultValues(profile, workDays, prayer, "follow");
    expect(values.motivators.learning).toBe(4);
    expect(values.workDays[0]?.start).toBe("09:00");
    expect(onboardingSchema.safeParse(values).success).toBe(true);
  });

  it("needs at least one motivator and valid work hours", () => {
    const values = defaultValues(profile, workDays, prayer, "follow");
    const silent = {
      ...values,
      motivators: { ...values.motivators, learning: 0, building: 0, health: 0 },
    };
    expect(onboardingSchema.safeParse(silent).success).toBe(false);

    const backwards = {
      ...values,
      workDays: values.workDays.map((day) => ({ ...day, end: "08:00" })),
    };
    expect(onboardingSchema.safeParse(backwards).success).toBe(false);
  });

  it("maps form values to commands", () => {
    const values = {
      ...defaultValues(profile, workDays, prayer, "follow"),
      name: "Mohtashim",
      familyEnabled: true,
      cityId: "karachi",
    };
    const saved = toProfile(values);
    expect(saved.name).toBe("Mohtashim");
    expect(saved.familyStartMinute).toBe(19 * 60);
    expect(toWorkDays(values)[0]).toEqual(workDays[0]);

    const settings = toPrayerSettings(values, prayer);
    expect(settings?.enabled).toBe(true);
    expect(settings?.city).toBe("Karachi");
    expect(toPrayerSettings({ ...values, cityId: "" }, prayer)).toBeNull();
  });
});

describe("applyDrafts", () => {
  const values = defaultValues(profile, workDays, prayer, "follow");

  it("turns motivator strengths into weights and keeps notes aside", () => {
    const next = applyDrafts(values, [
      {
        statement: "Rust matters a lot",
        kind: "motivator",
        subject: { type: "motivator", motivator: "learning" },
        strength: "high",
      },
      {
        statement: "Family time",
        kind: "motivator",
        subject: { type: "motivator", motivator: "family" },
        strength: "low",
      },
      {
        statement: "Be direct",
        kind: "preference",
        subject: { type: "coachStyle", style: "manager" },
        strength: "medium",
      },
      {
        statement: "I lose evenings to YouTube",
        kind: "pattern",
        subject: { type: "note" },
        strength: "medium",
      },
    ]);

    expect(next.motivators.learning).toBe(9);
    expect(next.motivators.family).toBe(3);
    expect(next.motivators.building).toBe(0);
    expect(next.coachStyle).toBe("manager");
    expect(next.beliefNotes.map((note) => note.statement)).toEqual([
      "I lose evenings to YouTube",
    ]);
  });

  it("keeps what was there when the chat named no motivator or style", () => {
    const next = applyDrafts(values, []);
    expect(next.motivators).toEqual(values.motivators);
    expect(next.coachStyle).toBe(values.coachStyle);
  });
});
