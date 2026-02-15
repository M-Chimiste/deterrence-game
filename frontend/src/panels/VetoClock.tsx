import { useGameStore } from "../store/gameState";
import { vetoEngagement, confirmEngagement } from "../ipc/bridge";
import type { EngagementView, WeaponType } from "../ipc/state";

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

function vetoColor(remaining: number, total: number): string {
  const ratio = total > 0 ? remaining / total : 0;
  if (ratio > 0.375) return "var(--cic-green)";
  if (ratio > 0.125) return "#ffaa00";
  return "#ff3333";
}

function EngagementCard({
  eng,
  focused,
}: {
  eng: EngagementView;
  focused: boolean;
}) {
  const borderColor = focused ? "var(--cic-green)" : "var(--cic-border)";

  if (eng.phase === "SolutionCalc") {
    return (
      <div
        class="veto-card"
        style={{ borderColor }}
        onClick={() =>
          useGameStore.getState().setFocusedEngagement(eng.engagement_id)
        }
      >
        <div class="veto-card-header">
          <span>
            TRK {eng.track_number} | {weaponLabel(eng.weapon_type)}
          </span>
          <span class="veto-pk">Pk {(eng.pk * 100).toFixed(0)}%</span>
        </div>
        <div class="veto-status computing">COMPUTING SOLUTION...</div>
      </div>
    );
  }

  if (eng.phase === "Ready") {
    const color = vetoColor(eng.veto_remaining_secs, eng.veto_total_secs);
    const pct =
      eng.veto_total_secs > 0
        ? (eng.veto_remaining_secs / eng.veto_total_secs) * 100
        : 0;

    return (
      <div
        class="veto-card"
        style={{ borderColor }}
        onClick={() =>
          useGameStore.getState().setFocusedEngagement(eng.engagement_id)
        }
      >
        <div class="veto-card-header">
          <span>
            TRK {eng.track_number} | {weaponLabel(eng.weapon_type)}
          </span>
          <span class="veto-pk">Pk {(eng.pk * 100).toFixed(0)}%</span>
        </div>
        <div class="veto-timer" style={{ color }}>
          {eng.veto_remaining_secs.toFixed(1)}s
        </div>
        <div class="veto-bar-bg">
          <div class="veto-bar-fill" style={{ width: `${pct}%`, background: color }} />
        </div>
        <div class="veto-actions">
          <button
            class="veto-btn veto-btn-veto"
            onClick={(e) => {
              e.stopPropagation();
              vetoEngagement(eng.engagement_id);
            }}
          >
            VETO
          </button>
          <button
            class="veto-btn veto-btn-confirm"
            onClick={(e) => {
              e.stopPropagation();
              confirmEngagement(eng.engagement_id);
            }}
          >
            CONFIRM
          </button>
        </div>
      </div>
    );
  }

  if (eng.phase === "Launched" || eng.phase === "Midcourse") {
    return (
      <div
        class="veto-card"
        style={{ borderColor }}
        onClick={() =>
          useGameStore.getState().setFocusedEngagement(eng.engagement_id)
        }
      >
        <div class="veto-card-header">
          <span>
            TRK {eng.track_number} | {weaponLabel(eng.weapon_type)}
          </span>
          <span class="veto-pk">Pk {(eng.pk * 100).toFixed(0)}%</span>
        </div>
        <div class="veto-status launched">
          BIRD AWAY — TTI {eng.time_to_intercept.toFixed(1)}s
        </div>
      </div>
    );
  }

  if (eng.phase === "Complete" && eng.result) {
    const resultClass = eng.result === "Hit" ? "splash-hit" : "splash-miss";
    return (
      <div
        class="veto-card"
        style={{ borderColor }}
      >
        <div class="veto-card-header">
          <span>
            TRK {eng.track_number} | {weaponLabel(eng.weapon_type)}
          </span>
        </div>
        <div class={`veto-status ${resultClass}`}>
          SPLASH — {eng.result === "Hit" ? "KILL" : "MISS"}
        </div>
      </div>
    );
  }

  return null;
}

export function VetoClock() {
  const engagements = useGameStore((s) => s.snapshot?.engagements ?? []);
  const focusedId = useGameStore((s) => s.focusedEngagementId);

  // Show active engagements + recently completed
  const visible = engagements.filter(
    (e) => e.phase !== "Aborted",
  );

  if (visible.length === 0) return null;

  return (
    <div class="panel veto-panel">
      <div class="panel-header">ENGAGEMENTS</div>
      {visible.map((eng) => (
        <EngagementCard
          key={eng.engagement_id}
          eng={eng}
          focused={eng.engagement_id === focusedId}
        />
      ))}
    </div>
  );
}
