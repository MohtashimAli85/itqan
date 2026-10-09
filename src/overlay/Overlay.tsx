import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type CSSProperties,
} from "react";
import { commands, events } from "@/shared/bindings/bindings";
import { useRewardSync } from "@/shared/hooks/useProgress";
import { useTasksSync } from "@/shared/hooks/useTasks";
import { Bubble } from "./components/Bubble";
import { CompactPanel } from "./components/CompactPanel";
import { Orb } from "./components/Orb";
import {
  createFollower,
  ORB_SIZE,
  type Follower,
  type Frame,
} from "./follower";
import { reportHitAreas } from "./hitAreas";
import { celebrateReward } from "./rewardEffects";
import { useOverlaySnapshot } from "./hooks/useOverlaySnapshot";
import "./overlay.css";

const POPOVER_WIDTH = 340;
const POPOVER_HEIGHT = 440;
const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

export function Overlay() {
  const { data: snapshot } = useOverlaySnapshot();
  useTasksSync();
  useRewardSync(celebrateReward);
  const anchor = useRef<HTMLDivElement>(null);
  const body = useRef<HTMLDivElement>(null);
  const pupils = useRef<(HTMLSpanElement | null)[]>([]);
  const [follower, setFollower] = useState<Follower | null>(null);

  useEffect(() => {
    const render = ({ position, pupils: offset, tilt }: Frame) => {
      const element = anchor.current;
      if (element) {
        element.style.transform = `translate3d(${position.x}px, ${position.y}px, 0)`;
        element.dataset.flipX = String(
          position.x > window.innerWidth - POPOVER_WIDTH,
        );
        element.dataset.flipY = String(
          position.y > window.innerHeight - POPOVER_HEIGHT,
        );
      }
      if (body.current) body.current.style.rotate = `${tilt}deg`;
      for (const pupil of pupils.current) {
        if (pupil) {
          pupil.style.transform = `translate(${offset.x}px, ${offset.y}px)`;
        }
      }
    };
    const instance = createFollower({
      render,
      onMove: () => reportHitAreas(null),
      onSettle: () => reportHitAreas(anchor.current),
      reducedMotion: () => reducedMotion.matches,
      viewport: () => ({
        width: window.innerWidth,
        height: window.innerHeight,
      }),
    });
    setFollower(instance);
    const unlisten = events.overlayCursor.listen((event) =>
      instance.setCursor(event.payload),
    );
    const onResize = () => instance.retarget();
    window.addEventListener("resize", onResize);
    return () => {
      instance.stop();
      window.removeEventListener("resize", onResize);
      void unlisten.then((stop) => stop());
    };
  }, []);

  const followMode = snapshot?.followMode ?? "follow";
  const mode =
    snapshot?.docked && followMode === "follow" ? "corner" : followMode;
  const panelOpen = snapshot?.panelOpen ?? false;
  const bubble = panelOpen ? null : (snapshot?.bubble ?? null);

  const reportSettled = useCallback(() => {
    if (follower && !follower.isMoving()) reportHitAreas(anchor.current);
  }, [follower]);

  const closePanel = useCallback(() => void commands.setPanelOpen(false), []);

  useEffect(() => {
    if (!panelOpen) return;
    window.addEventListener("blur", closePanel);
    return () => window.removeEventListener("blur", closePanel);
  }, [panelOpen, closePanel]);

  useEffect(() => {
    follower?.setMode(mode);
  }, [follower, mode]);

  useLayoutEffect(() => {
    follower?.setPinned(bubble !== null || panelOpen);
    reportSettled();
  }, [follower, bubble, panelOpen, reportSettled]);

  if (!snapshot || mode === "hidden") return null;

  const pupilRef = (index: number) => (element: HTMLSpanElement | null) => {
    pupils.current[index] = element;
  };

  return (
    <div
      ref={anchor}
      className="orb-anchor fixed top-0 left-0"
      style={{ "--orb-size": `${ORB_SIZE}px` } as CSSProperties}
    >
      <Orb
        state={snapshot.orbState}
        progress={snapshot.progress}
        bodyRef={body}
        pupilRef={pupilRef}
        onClick={() => void commands.setPanelOpen(!panelOpen)}
      />
      {panelOpen && (
        <CompactPanel onClose={closePanel} onResize={reportSettled} />
      )}
      {bubble && (
        <Bubble
          bubble={bubble}
          onResolve={(actionId) =>
            void commands.resolveBubble(bubble.id, actionId)
          }
        />
      )}
    </div>
  );
}
