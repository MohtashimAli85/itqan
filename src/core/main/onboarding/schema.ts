import { z } from "zod";
import type {
  DraftBelief,
  FollowMode,
  Motivator,
  PrayerSettings,
  Profile,
  WorkDay,
} from "@/shared/bindings/bindings";
import { cityById } from "@/shared/lib/cities";
import { minutesToTime, timeToMinutes } from "@/shared/lib/time";

export const motivatorOptions: { id: Motivator; label: string }[] = [
  { id: "learning", label: "Learning" },
  { id: "building", label: "Building" },
  { id: "health", label: "Health" },
  { id: "money", label: "Money" },
  { id: "recognition", label: "Recognition" },
  { id: "family", label: "Family" },
  { id: "freedom", label: "Freedom" },
  { id: "status", label: "Status" },
];

const motivatorIds = [
  "learning",
  "building",
  "health",
  "money",
  "recognition",
  "family",
  "freedom",
  "status",
] as const satisfies readonly Motivator[];

const clock = z.string().refine((value) => timeToMinutes(value) !== null, {
  message: "Use a time like 09:00",
});

const workDay = z.object({
  weekday: z.number().int().min(0).max(6),
  enabled: z.boolean(),
  start: clock,
  end: clock,
});

export const onboardingSchema = z
  .object({
    name: z.string().trim().max(60),
    motivators: z.record(z.enum(motivatorIds), z.number().min(0).max(10)),
    coachStyle: z.enum(["mentor", "manager", "trainer"]),
    freeHours: z.number().int().min(0).max(168),
    age: z.number().int().min(13).max(110).nullable(),
    workDays: z.array(workDay).length(7),
    familyEnabled: z.boolean(),
    familyStart: clock,
    familyEnd: clock,
    cityId: z.string(),
    madhab: z.enum(["hanafi", "shafi"]),
    followMode: z.enum(["follow", "corner", "hidden"]),
    beliefNotes: z.array(z.custom<DraftBelief>()),
    chat: z.enum(["idle", "open", "done"]),
  })
  .refine((values) => values.chat !== "open", {
    path: ["motivators"],
    message: "Finish the questions, or choose Set it myself",
  })
  .refine(
    (values) => Object.values(values.motivators).some((weight) => weight > 0),
    {
      path: ["motivators"],
      message: "Pick at least one thing that drives you",
    },
  )
  .refine(
    (values) =>
      values.workDays.every(
        (day) =>
          !day.enabled ||
          (timeToMinutes(day.end) ?? 0) > (timeToMinutes(day.start) ?? 0),
      ),
    { path: ["workDays"], message: "Work must end after it starts" },
  )
  .refine(
    (values) =>
      !values.familyEnabled ||
      (timeToMinutes(values.familyEnd) ?? 0) >
        (timeToMinutes(values.familyStart) ?? 0),
    { path: ["familyEnd"], message: "Family time must end after it starts" },
  );

export type OnboardingValues = z.infer<typeof onboardingSchema>;

export function defaultValues(
  profile: Profile,
  workDays: WorkDay[],
  prayer: PrayerSettings,
  followMode: FollowMode,
): OnboardingValues {
  const weights = Object.fromEntries(
    motivatorIds.map((id) => [
      id,
      profile.motivators.find((weight) => weight.motivator === id)?.weight ?? 0,
    ]),
  ) as Record<Motivator, number>;
  const hasWeights = Object.values(weights).some((weight) => weight > 0);
  return {
    name: profile.name ?? "",
    motivators: hasWeights
      ? weights
      : { ...weights, learning: 4, building: 4, health: 3 },
    coachStyle: profile.coachStyle,
    freeHours: profile.freeHoursPerWeek ?? 8,
    age: profile.age,
    workDays: workDays.map((day) => ({
      weekday: day.weekday,
      enabled: day.enabled,
      start: minutesToTime(day.startMinute),
      end: minutesToTime(day.endMinute),
    })),
    familyEnabled: profile.familyStartMinute !== null,
    familyStart: minutesToTime(profile.familyStartMinute ?? 19 * 60),
    familyEnd: minutesToTime(profile.familyEndMinute ?? 21 * 60),
    cityId: "",
    madhab: prayer.madhab,
    followMode,
    beliefNotes: [],
    chat: "idle",
  };
}

const strengthWeight = { high: 9, medium: 6, low: 3 } as const;

export function applyDrafts(
  values: Pick<OnboardingValues, "motivators" | "coachStyle">,
  drafts: DraftBelief[],
): Pick<OnboardingValues, "motivators" | "coachStyle" | "beliefNotes"> {
  const picked = drafts.flatMap((draft) =>
    draft.subject.type === "motivator"
      ? [
          {
            id: draft.subject.motivator,
            weight: strengthWeight[draft.strength],
          },
        ]
      : [],
  );
  const motivators =
    picked.length === 0
      ? values.motivators
      : (Object.fromEntries(
          motivatorIds.map((id) => [
            id,
            picked.find((pick) => pick.id === id)?.weight ?? 0,
          ]),
        ) as Record<Motivator, number>);
  const style = drafts.find((draft) => draft.subject.type === "coachStyle");
  return {
    motivators,
    coachStyle:
      style?.subject.type === "coachStyle"
        ? style.subject.style
        : values.coachStyle,
    beliefNotes: drafts.filter((draft) => draft.subject.type === "note"),
  };
}

export function toProfile(values: OnboardingValues): Profile {
  return {
    name: values.name || null,
    motivators: motivatorIds.map((motivator) => ({
      motivator,
      weight: values.motivators[motivator],
    })),
    situation: null,
    freeHoursPerWeek: values.freeHours,
    age: values.age,
    coachStyle: values.coachStyle,
    familyStartMinute: values.familyEnabled
      ? timeToMinutes(values.familyStart)
      : null,
    familyEndMinute: values.familyEnabled
      ? timeToMinutes(values.familyEnd)
      : null,
    updatedAt: null,
  };
}

export function toWorkDays(values: OnboardingValues): WorkDay[] {
  return values.workDays.map((day) => ({
    weekday: day.weekday,
    enabled: day.enabled,
    startMinute: timeToMinutes(day.start) ?? 540,
    endMinute: timeToMinutes(day.end) ?? 1020,
  }));
}

export function toPrayerSettings(
  values: OnboardingValues,
  current: PrayerSettings,
): PrayerSettings | null {
  const city = cityById(values.cityId);
  if (!city) return null;
  return {
    ...current,
    enabled: true,
    city: city.name,
    latitude: city.latitude,
    longitude: city.longitude,
    method: city.method,
    madhab: values.madhab,
  };
}
