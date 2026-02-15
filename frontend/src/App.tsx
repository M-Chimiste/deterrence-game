import { useEffect } from "preact/hooks";
import { useGameStore } from "./store/gameState";
import {
  onSnapshot,
  startSimulation,
  vetoEngagement,
  confirmEngagement,
} from "./ipc/bridge";
import { DebugOverlay } from "./debug/DebugOverlay";
import { PPI } from "./tactical/PPI";
import { TrackBlock } from "./panels/TrackBlock";
import { RadarStatus } from "./panels/RadarStatus";
import { VetoClock } from "./panels/VetoClock";
import { ThreatTable } from "./panels/ThreatTable";

export function App() {
  const snapshot = useGameStore((s) => s.snapshot);
  const connected = useGameStore((s) => s.connected);
  const phase = snapshot?.phase;

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    async function init() {
      unlisten = await onSnapshot((snap) => {
        useGameStore.getState().setSnapshot(snap);
        if (!useGameStore.getState().connected) {
          useGameStore.getState().setConnected(true);
        }
      });

      try {
        await startSimulation();
      } catch (e) {
        console.error("Failed to start simulation:", e);
      }
    }

    init();

    return () => {
      unlisten?.();
    };
  }, []);

  // Keyboard shortcuts for engagement management
  useEffect(() => {
    function handleKeydown(e: KeyboardEvent) {
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      const store = useGameStore.getState();

      switch (e.key) {
        case "Tab": {
          e.preventDefault();
          store.cycleFocusedEngagement();
          break;
        }
        case "v":
        case "V": {
          if (store.focusedEngagementId !== null) {
            vetoEngagement(store.focusedEngagementId);
          }
          break;
        }
        case "c":
        case "C": {
          if (store.focusedEngagementId !== null) {
            confirmEngagement(store.focusedEngagementId);
          }
          break;
        }
      }
    }

    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  }, []);

  const showPPI = phase === "Active" || phase === "Paused";

  return (
    <div class="app-root">
      {showPPI ? (
        <div class="cic-layout">
          {/* PPI fills the center */}
          <div class="ppi-container">
            <PPI />
          </div>

          {/* Overlay panels */}
          <div class="panel-overlay top-left">
            <RadarStatus />
          </div>

          <div class="panel-overlay bottom-left">
            <TrackBlock />
          </div>

          <div class="panel-overlay top-right">
            <DebugOverlay />
          </div>

          <div class="panel-overlay right-side">
            <VetoClock />
          </div>

          <div class="panel-overlay bottom-center">
            <ThreatTable />
          </div>
        </div>
      ) : (
        <div class="splash-screen">
          <h1 class="splash-title">DETERRENCE</h1>
          <p class="splash-subtitle">INTEGRATED AIR AND MISSILE DEFENSE</p>
          <div class="splash-status">
            {connected
              ? `TICK ${snapshot?.time.tick ?? 0} | ${snapshot?.tracks.length ?? 0} TRACKS`
              : "AWAITING CONNECTION"}
          </div>
          <DebugOverlay />
        </div>
      )}
    </div>
  );
}
