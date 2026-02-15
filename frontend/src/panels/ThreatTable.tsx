import { useGameStore } from "../store/gameState";
import type {
  TrackView,
  EngagementView,
  Classification,
  WeaponType,
} from "../ipc/state";

const RAD_TO_DEG = 180 / Math.PI;

function classLabel(c: Classification): string {
  switch (c) {
    case "Hostile":
      return "HOS";
    case "Suspect":
      return "SUS";
    case "Unknown":
      return "UNK";
    case "Pending":
      return "PND";
    case "Friend":
      return "FRN";
    case "AssumedFriend":
      return "AFR";
    case "Neutral":
      return "NEU";
  }
}

function weaponLabel(wt: WeaponType): string {
  switch (wt) {
    case "Standard":
      return "SM";
    case "ExtendedRange":
      return "ER";
    case "PointDefense":
      return "PD";
  }
}

function formatRange(meters: number): string {
  if (meters >= 1000) {
    return `${(meters / 1000).toFixed(1)}km`;
  }
  return `${meters.toFixed(0)}m`;
}

function formatBearing(radians: number): string {
  const deg = ((radians * RAD_TO_DEG) % 360 + 360) % 360;
  return deg.toFixed(0).padStart(3, "0");
}

function formatSpeed(mps: number): string {
  return `${mps.toFixed(0)}`;
}

interface ThreatRow {
  track: TrackView;
  engagement: EngagementView | undefined;
}

export function ThreatTable() {
  const snapshot = useGameStore((s) => s.snapshot);
  if (!snapshot) return null;

  // Show hostile, suspect, and unknown tracks
  const threatTracks = snapshot.tracks
    .filter(
      (t) =>
        t.classification === "Hostile" ||
        t.classification === "Suspect" ||
        t.classification === "Unknown",
    )
    .sort((a, b) => a.range - b.range);

  if (threatTracks.length === 0) return null;

  // Map engagements by track number
  const engByTrack = new Map<number, EngagementView>();
  for (const eng of snapshot.engagements) {
    if (eng.phase !== "Complete" && eng.phase !== "Aborted") {
      engByTrack.set(eng.track_number, eng);
    }
  }

  const rows: ThreatRow[] = threatTracks.map((track) => ({
    track,
    engagement: engByTrack.get(track.track_number),
  }));

  return (
    <div class="panel threat-table">
      <div class="panel-header">THREAT BOARD</div>
      <table>
        <thead>
          <tr>
            <th>TRK</th>
            <th>BRG</th>
            <th>RNG</th>
            <th>SPD</th>
            <th>CLS</th>
            <th>ENG</th>
            <th>WPN</th>
            <th>Pk</th>
          </tr>
        </thead>
        <tbody>
          {rows.map(({ track, engagement }) => (
            <tr
              key={track.track_number}
              class={track.hooked ? "row-hooked" : ""}
            >
              <td>{track.track_number}</td>
              <td>{formatBearing(track.bearing)}</td>
              <td>{formatRange(track.range)}</td>
              <td>{formatSpeed(track.speed)}</td>
              <td class={`cls-${track.classification.toLowerCase()}`}>
                {classLabel(track.classification)}
              </td>
              <td>
                {engagement
                  ? engagement.phase === "Ready"
                    ? `RDY ${engagement.veto_remaining_secs.toFixed(1)}s`
                    : engagement.phase === "SolutionCalc"
                      ? "CALC"
                      : engagement.phase === "Launched" ||
                          engagement.phase === "Midcourse"
                        ? "FLT"
                        : engagement.phase
                  : "—"}
              </td>
              <td>
                {engagement ? weaponLabel(engagement.weapon_type) : "—"}
              </td>
              <td>
                {engagement
                  ? `${(engagement.pk * 100).toFixed(0)}%`
                  : "—"}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <div class="threat-table-footer">
        SCORE: {snapshot.score.threats_killed}/{snapshot.score.threats_total} KILL
        {" | "}
        {snapshot.score.interceptors_fired} FIRED
      </div>
    </div>
  );
}
