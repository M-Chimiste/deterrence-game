/**
 * Radar Status Panel — displays radar mode, sector, energy budget, and track count.
 */

import { useGameStore } from "../store/gameState";

const RAD_TO_DEG = 180 / Math.PI;

export function RadarStatus() {
  const snapshot = useGameStore((s) => s.snapshot);
  if (!snapshot) return null;

  const { radar } = snapshot;
  const searchPct = Math.round((radar.energy_search / radar.energy_total) * 100);
  const trackPct = Math.round((radar.energy_track / radar.energy_total) * 100);
  const sectorDeg = Math.round(radar.sector_width * RAD_TO_DEG);

  return (
    <div class="panel radar-status">
      <div class="panel-header">RADAR</div>
      <div class="panel-row">
        <span>MODE</span>
        <span>{radar.mode}</span>
      </div>
      <div class="panel-row">
        <span>SECTOR</span>
        <span>{sectorDeg}&deg;</span>
      </div>
      <div class="panel-row">
        <span>TRACKS</span>
        <span>{radar.active_track_count}</span>
      </div>
      <div class="energy-bar-container">
        <div class="energy-label">ENERGY</div>
        <div class="energy-bar">
          <div
            class="energy-bar-search"
            style={{ width: `${searchPct}%` }}
          />
          <div
            class="energy-bar-track"
            style={{ width: `${trackPct}%` }}
          />
        </div>
        <div class="energy-labels">
          <span>SRCH {searchPct}%</span>
          <span>TRK {trackPct}%</span>
        </div>
      </div>
    </div>
  );
}
