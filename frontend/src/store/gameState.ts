import { create } from "zustand";
import type { GameStateSnapshot } from "../ipc/state";

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

  setSnapshot: (snapshot: GameStateSnapshot) => void;
  setConnected: (connected: boolean) => void;
  setFocusedEngagement: (id: number | null) => void;
  cycleFocusedEngagement: () => void;
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
}));
