/**
 * Interpolation utilities for smooth rendering between snapshots.
 *
 * The simulation runs at 30Hz but the frontend renders at 60fps.
 * We lerp track positions between the previous and current snapshot
 * based on elapsed time since the current snapshot arrived.
 */

import type { Position, TrackView } from "../ipc/state";
import { useGameStore } from "./gameState";

/** Nominal tick interval in milliseconds (30Hz). */
const TICK_INTERVAL_MS = 1000 / 30;

/**
 * Compute the interpolation factor (0..1) for the current frame.
 *
 * 0.0 = at the current snapshot's time.
 * 1.0 = one full tick interval has passed (next snapshot expected).
 * Clamped to [0, 1] to prevent extrapolation beyond one tick.
 */
export function getInterpolationFactor(): number {
  const state = useGameStore.getState();
  if (!state.snapshot || !state.previousSnapshot) return 0;

  const elapsed = performance.now() - state.snapshotReceivedAt;
  return Math.min(Math.max(elapsed / TICK_INTERVAL_MS, 0), 1);
}

/** Linearly interpolate between two positions. */
export function lerpPosition(a: Position, b: Position, t: number): Position {
  return {
    x: a.x + (b.x - a.x) * t,
    y: a.y + (b.y - a.y) * t,
    z: a.z + (b.z - a.z) * t,
  };
}

/**
 * Get interpolated track positions for the current frame.
 *
 * Matches tracks between previous and current snapshot by track_number,
 * then lerps positions. New tracks (only in current) use their exact position.
 */
export function getInterpolatedTracks(): TrackView[] {
  const state = useGameStore.getState();
  const current = state.snapshot;
  if (!current) return [];

  const previous = state.previousSnapshot;
  if (!previous) return current.tracks;

  const t = getInterpolationFactor();
  if (t <= 0.001) return current.tracks;

  const prevMap = new Map<number, TrackView>();
  for (const track of previous.tracks) {
    prevMap.set(track.track_number, track);
  }

  return current.tracks.map((track) => {
    const prev = prevMap.get(track.track_number);
    if (!prev) return track;

    return {
      ...track,
      position: lerpPosition(prev.position, track.position, t),
    };
  });
}
