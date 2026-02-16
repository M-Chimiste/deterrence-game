/**
 * Mission complete summary screen.
 */

import { useGameStore } from "../store/gameState";
import { returnToMenu } from "../ipc/bridge";

export function MissionComplete() {
  const score = useGameStore((s) => s.snapshot?.score);
  const scenario = useGameStore((s) => s.snapshot?.scenario);

  if (!score) return null;

  const missionTime = formatTime(score.mission_time_secs);
  const efficiency =
    score.interceptors_fired > 0
      ? ((score.threats_killed / score.interceptors_fired) * 100).toFixed(0)
      : "N/A";

  const grade = computeGrade(score.threats_killed, score.threats_total, score.assets_protected);

  return (
    <div class="mission-complete">
      <h2 class="mission-complete-header">MISSION COMPLETE</h2>
      <div class="mission-grade">{grade}</div>
      {scenario && (
        <div class="mission-scenario">{scenario.toUpperCase()} SCENARIO</div>
      )}
      <div class="mission-stats">
        <div class="stat-row">
          <span class="stat-label">THREATS KILLED</span>
          <span class="stat-value">
            {score.threats_killed} / {score.threats_total}
          </span>
        </div>
        <div class="stat-row">
          <span class="stat-label">INTERCEPTORS FIRED</span>
          <span class="stat-value">{score.interceptors_fired}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">EFFICIENCY</span>
          <span class="stat-value">{efficiency}%</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">MISSION TIME</span>
          <span class="stat-value">{missionTime}</span>
        </div>
        <div class="stat-row">
          <span class="stat-label">ASSETS</span>
          <span class={`stat-value ${score.assets_protected ? "stat-good" : "stat-bad"}`}>
            {score.assets_protected ? "PROTECTED" : "DAMAGED"}
          </span>
        </div>
      </div>
      <button class="mission-btn" onClick={() => returnToMenu()}>
        RETURN TO MENU
      </button>
    </div>
  );
}

function formatTime(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

function computeGrade(killed: number, total: number, assetsProtected: boolean): string {
  if (!assetsProtected) return "F";
  const ratio = total > 0 ? killed / total : 0;
  if (ratio >= 0.95) return "S";
  if (ratio >= 0.8) return "A";
  if (ratio >= 0.6) return "B";
  if (ratio >= 0.4) return "C";
  return "D";
}
