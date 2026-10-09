import type { OrbState } from "@/shared/bindings/bindings";

type EyesProps = {
  state: OrbState;
  pupilRef: (index: number) => (element: HTMLSpanElement | null) => void;
};

const shapes: Partial<Record<OrbState, string>> = {
  happy: "h-2 w-3.5 rounded-t-full border-t-[3px] border-orb-eye",
  focus: "h-[3px] w-3.5 rounded-full bg-orb-eye",
  resting: "h-2 w-3.5 rounded-b-full border-b-[3px] border-orb-eye",
};

export function Eyes({ state, pupilRef }: EyesProps) {
  const shape = shapes[state];
  const wide = state === "alert";

  return (
    <>
      {[0, 1].map((index) =>
        shape ? (
          <span key={index} className={shape} />
        ) : (
          <span
            key={index}
            className={
              wide
                ? "bg-orb-eye flex h-5 w-4 items-center justify-center rounded-full"
                : "bg-orb-eye flex h-4 w-3.5 items-center justify-center rounded-full"
            }
          >
            <span
              ref={pupilRef(index)}
              className={
                wide
                  ? "bg-orb-pupil size-1.5 rounded-full"
                  : "bg-orb-pupil size-2 rounded-full"
              }
            />
          </span>
        ),
      )}
    </>
  );
}
