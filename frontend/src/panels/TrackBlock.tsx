/**
 * Track Block — displays detailed information for the hooked (selected) track.
 * Shows bearing, range, altitude, speed, heading, classification, IFF, and quality.
 */

import { useGameStore } from "../store/gameState";

const RAD_TO_DEG = 180 / Math.PI;
const M_TO_NM = 1 / 1852;
const M_TO_FT = 3.281;
const MS_TO_KTS = 1.944;

export function TrackBlock() {
  const snapshot = useGameStore((s) => s.snapshot);
  if (!snapshot) return null;

  const hooked = snapshot.tracks.find((t) => t.hooked);
  if (!hooked) return null;

  const bearingDeg = (hooked.bearing * RAD_TO_DEG).toFixed(1);
  const rangeNm = (hooked.range * M_TO_NM).toFixed(1);
  const altFt = (hooked.altitude * M_TO_FT).toFixed(0);
  const speedKts = (hooked.speed * MS_TO_KTS).toFixed(0);
  const headingDeg = (hooked.heading * RAD_TO_DEG).toFixed(1);
  const qualityPct = (hooked.quality * 100).toFixed(0);

  return (
    <div class="panel track-block">
      <div class="panel-header">TRACK {hooked.track_number}</div>
      <table>
        <tbody>
          <tr>
            <td>BRG</td>
            <td>{bearingDeg}&deg;</td>
          </tr>
          <tr>
            <td>RNG</td>
            <td>{rangeNm} nm</td>
          </tr>
          <tr>
            <td>ALT</td>
            <td>{altFt} ft</td>
          </tr>
          <tr>
            <td>SPD</td>
            <td>{speedKts} kts</td>
          </tr>
          <tr>
            <td>HDG</td>
            <td>{headingDeg}&deg;</td>
          </tr>
          <tr>
            <td>CLASS</td>
            <td>{hooked.classification}</td>
          </tr>
          <tr>
            <td>IFF</td>
            <td>{hooked.iff_status}</td>
          </tr>
          <tr>
            <td>QUAL</td>
            <td>{qualityPct}%</td>
          </tr>
        </tbody>
      </table>
    </div>
  );
}
