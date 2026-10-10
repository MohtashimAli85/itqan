import type { RewardEarned } from "@/shared/bindings/bindings";

async function play(name: "xp" | "celebrate") {
  const { Howl } = await import("howler");
  new Howl({ src: [`/sounds/${name}.wav`], volume: 0.5 }).play();
}

async function confetti() {
  const { default: burst } = await import("canvas-confetti");
  await burst({
    particleCount: 120,
    spread: 75,
    startVelocity: 40,
    origin: { y: 0.7 },
    disableForReducedMotion: true,
  });
}

export function celebrateReward(reward: RewardEarned) {
  if (reward.sound) {
    void play(reward.celebration ? "celebrate" : "xp");
  }
  if (
    reward.celebration &&
    !window.matchMedia("(prefers-reduced-motion: reduce)").matches
  ) {
    void confetti();
  }
}
