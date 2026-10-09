import { useEffect, type RefObject } from "react";
import type { OrbState } from "@/shared/bindings/bindings";

const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

const hop: Keyframe[] = [
  { transform: "translateY(0)" },
  { transform: "translateY(-10px)", offset: 0.4 },
  { transform: "translateY(0)" },
];
const wiggle: Keyframe[] = [
  { transform: "rotate(0)" },
  { transform: "rotate(-10deg)" },
  { transform: "rotate(10deg)" },
  { transform: "rotate(-6deg)" },
  { transform: "rotate(0)" },
];
const blink: Keyframe[] = [
  { transform: "scaleY(1)" },
  { transform: "scaleY(0.1)" },
  { transform: "scaleY(1)" },
];
const floatZ: Keyframe[] = [
  { opacity: 0, transform: "translate(0, 0) scale(0.6)" },
  { opacity: 1, transform: "translate(4px, -10px) scale(1)", offset: 0.3 },
  { opacity: 0, transform: "translate(10px, -26px) scale(1.1)" },
];

const eyesOpen: OrbState[] = [
  "idle",
  "alert",
  "critical",
  "listening",
  "evening",
];

function play(
  element: Element | null,
  keyframes: Keyframe[],
  duration: number,
) {
  if (!element || reducedMotion.matches) return;
  element.animate(keyframes, { duration, easing: "ease-in-out" });
}

function nextDelay() {
  return 3000 + Math.random() * 4000;
}

type Targets = {
  body: RefObject<HTMLElement | null>;
  eyes: RefObject<HTMLElement | null>;
  z: RefObject<HTMLElement | null>;
};

export function useOrbAnimations(state: OrbState, targets: Targets) {
  const { body, eyes, z } = targets;

  useEffect(() => {
    if (state === "happy" || state === "alert") {
      play(body.current, hop, 420);
    }
  }, [state, body]);

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout>;
    const tick = () => {
      if (state === "critical") play(body.current, wiggle, 500);
      if (state === "resting") play(z.current, floatZ, 2200);
      if (eyesOpen.includes(state)) play(eyes.current, blink, 160);
      timer = setTimeout(tick, state === "critical" ? 2500 : nextDelay());
    };
    timer = setTimeout(tick, nextDelay());
    return () => clearTimeout(timer);
  }, [state, body, eyes, z]);
}
