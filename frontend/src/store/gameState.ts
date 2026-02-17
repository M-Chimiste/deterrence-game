import { create } from "zustand";
import type { GameStateSnapshot, TerrainDataPayload } from "../ipc/state";
import { getTerrainData } from "../ipc/bridge";
import { MusicManager, SfxEngine, consumeAudioEvents } from "../audio";
import type { MusicPhase } from "../audio";

/** Duration of the FPS measurement window in seconds. */
const FPS_WINDOW_SECS = 1.0;

interface GameStore {
  snapshot: GameStateSnapshot | null;
  previousSnapshot: GameStateSnapshot | null;
  connected: boolean;
  /** Wall-clock time when the current snapshot arrived (ms). */
  snapshotReceivedAt: number;
  /** Snapshots received in the current measurement window. */
  snapshotCount: number;
  /** Start of the current measurement window (ms). */
  fpsWindowStart: number;
  /** Computed snapshot receive rate (Hz). */
  snapshotRate: number;
  /** Currently focused engagement ID (for veto/confirm keybinds). */
  focusedEngagementId: number | null;
  /** Whether the audio system has been initialized (requires user gesture). */
  audioInitialized: boolean;
  /** Terrain data fetched once per mission for PPI overlay and 3D view. */
  terrainData: TerrainDataPayload | null;
  /** Whether terrain data has been fetched for the current mission. */
  terrainFetched: boolean;
  /** Current view mode: PPI radar scope or 3D world view. */
  viewMode: "ppi" | "world";

  musicManager: MusicManager;
  sfxEngine: SfxEngine;

  setSnapshot: (snapshot: GameStateSnapshot) => void;
  setConnected: (connected: boolean) => void;
  setFocusedEngagement: (id: number | null) => void;
  cycleFocusedEngagement: () => void;
  initAudio: () => void;
  toggleViewMode: () => void;
}

/** Determine music phase from game state. */
function getMusicPhase(snapshot: GameStateSnapshot): MusicPhase {
  switch (snapshot.phase) {
    case "MainMenu":
    case "MissionBriefing":
      return "menu";
    case "Active": {
      switch (snapshot.scenario) {
        case "Medium":
          return "level2";
        case "Hard":
          return "level3";
        default:
          return "level1";
      }
    }
    case "Paused":
      return "silent";
    case "MissionComplete":
      return "gameover";
    default:
      return "silent";
  }
}

export const useGameStore = create<GameStore>((set, get) => ({
  snapshot: null,
  previousSnapshot: null,
  connected: false,
  snapshotReceivedAt: 0,
  snapshotCount: 0,
  fpsWindowStart: performance.now(),
  snapshotRate: 0,
  focusedEngagementId: null,
  audioInitialized: false,
  terrainData: null,
  terrainFetched: false,
  viewMode: "ppi",

  musicManager: new MusicManager(),
  sfxEngine: new SfxEngine(),

  setSnapshot: (snapshot: GameStateSnapshot) => {
    const now = performance.now();
    const state = get();
    let { snapshotCount, fpsWindowStart, snapshotRate } = state;

    snapshotCount += 1;
    const elapsed = (now - fpsWindowStart) / 1000;

    if (elapsed >= FPS_WINDOW_SECS) {
      snapshotRate = snapshotCount / elapsed;
      snapshotCount = 0;
      fpsWindowStart = now;
    }

    // Auto-clear focused engagement if it no longer exists
    let { focusedEngagementId } = state;
    if (
      focusedEngagementId !== null &&
      !snapshot.engagements.some(
        (e) => e.engagement_id === focusedEngagementId,
      )
    ) {
      focusedEngagementId = null;
    }

    // Consume audio events from this snapshot
    if (state.audioInitialized && snapshot.audio_events.length > 0) {
      consumeAudioEvents(snapshot.audio_events, state.sfxEngine);
    }

    // Update music phase based on game state
    if (state.audioInitialized) {
      const musicPhase = getMusicPhase(snapshot);
      state.musicManager.setPhase(musicPhase);
    }

    // Fetch terrain data once when mission starts with terrain
    if (
      snapshot.phase === "Active" &&
      snapshot.terrain_meta !== null &&
      !state.terrainFetched
    ) {
      set({ terrainFetched: true });
      getTerrainData().then((data) => {
        if (data) set({ terrainData: data });
      });
    }

    // Clear terrain data when returning to menu
    if (snapshot.phase === "MainMenu" && state.terrainFetched) {
      set({ terrainData: null, terrainFetched: false });
    }

    set({
      previousSnapshot: state.snapshot,
      snapshot,
      snapshotReceivedAt: now,
      snapshotCount,
      fpsWindowStart,
      snapshotRate,
      focusedEngagementId,
    });
  },

  setConnected: (connected: boolean) => set({ connected }),

  setFocusedEngagement: (id: number | null) =>
    set({ focusedEngagementId: id }),

  cycleFocusedEngagement: () => {
    const state = get();
    const engagements = state.snapshot?.engagements ?? [];
    const active = engagements.filter(
      (e) => e.phase !== "Complete" && e.phase !== "Aborted",
    );
    if (active.length === 0) {
      set({ focusedEngagementId: null });
      return;
    }

    const currentIdx = active.findIndex(
      (e) => e.engagement_id === state.focusedEngagementId,
    );
    const nextIdx = (currentIdx + 1) % active.length;
    set({ focusedEngagementId: active[nextIdx].engagement_id });
  },

  toggleViewMode: () => {
    const current = get().viewMode;
    set({ viewMode: current === "ppi" ? "world" : "ppi" });
  },

  initAudio: () => {
    const state = get();
    if (state.audioInitialized) return;

    state.musicManager.init();
    state.sfxEngine.init();
    set({ audioInitialized: true });

    // Set initial music phase from current snapshot
    if (state.snapshot) {
      const musicPhase = getMusicPhase(state.snapshot);
      state.musicManager.setPhase(musicPhase);
    }
  },
}));
