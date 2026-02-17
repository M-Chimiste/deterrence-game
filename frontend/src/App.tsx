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
import { WorldScene } from "./world/WorldScene";
import { TrackBlock } from "./panels/TrackBlock";
import { RadarStatus } from "./panels/RadarStatus";
import { VetoClock } from "./panels/VetoClock";
import { ThreatTable } from "./panels/ThreatTable";
import { VLSStatus } from "./panels/VLSStatus";
import { IlluminatorStatus } from "./panels/IlluminatorStatus";
import { DoctrineDisplay } from "./panels/DoctrineDisplay";
import { ScenarioSelect } from "./screens/ScenarioSelect";
import { MissionComplete } from "./screens/MissionComplete";

export function App() {
  const snapshot = useGameStore((s) => s.snapshot);
  const connected = useGameStore((s) => s.connected);
  const viewMode = useGameStore((s) => s.viewMode);
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

  // Init audio on first user gesture (click or keydown)
  useEffect(() => {
    function initAudioOnGesture() {
      useGameStore.getState().initAudio();
      window.removeEventListener("click", initAudioOnGesture);
      window.removeEventListener("keydown", initAudioOnGesture);
    }

    window.addEventListener("click", initAudioOnGesture);
    window.addEventListener("keydown", initAudioOnGesture);

    return () => {
      window.removeEventListener("click", initAudioOnGesture);
      window.removeEventListener("keydown", initAudioOnGesture);
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
        case "w":
        case "W": {
          store.toggleViewMode();
          break;
        }
      }
    }

    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  }, []);

  const showCIC = phase === "Active" || phase === "Paused";
  const showMissionComplete = phase === "MissionComplete";

  return (
    <div class="app-root">
      {showCIC ? (
        <div class="cic-layout">
          {/* Main view: PPI radar scope or 3D world */}
          <div class="ppi-container">
            {viewMode === "ppi" ? <PPI /> : <WorldScene />}
          </div>

          {/* View mode indicator */}
          <div class="panel-overlay top-center">
            <span class="view-indicator">
              {viewMode === "ppi" ? "PPI" : "3D"} [W]
            </span>
          </div>

          {/* Overlay panels */}
          <div class="panel-overlay top-left">
            <RadarStatus />
          </div>

          <div class="panel-overlay bottom-left">
            <TrackBlock />
          </div>

          <div class="panel-overlay right-column">
            <DebugOverlay />
            <DoctrineDisplay />
            <VLSStatus />
            <VetoClock />
            <IlluminatorStatus />
          </div>

          <div class="panel-overlay bottom-center">
            <ThreatTable />
          </div>
        </div>
      ) : showMissionComplete ? (
        <MissionComplete />
      ) : connected ? (
        <ScenarioSelect />
      ) : (
        <div class="splash-screen">
          <h1 class="splash-title">DETERRENCE</h1>
          <p class="splash-subtitle">INTEGRATED AIR AND MISSILE DEFENSE</p>
          <div class="splash-status">AWAITING CONNECTION</div>
        </div>
      )}
    </div>
  );
}
