import { useGameStore } from "../store/gameState";
import type { IlluminatorStatus as IllumStatus } from "../ipc/state";

function statusColor(status: IllumStatus): string {
  switch (status) {
    case "Idle":
      return "var(--cic-green-dim)";
    case "Active":
      return "var(--cic-green)";
    case "TimeSharing":
      return "#ffaa00";
  }
}

export function IlluminatorStatus() {
  const illuminators = useGameStore((s) => s.snapshot?.illuminators ?? []);

  if (illuminators.length === 0) return null;

  const queueDepth = illuminators[0]?.queue_depth ?? 0;

  return (
    <div class="panel illum-panel">
      <div class="panel-header">ILLUMINATORS</div>
      {illuminators.map((illum) => (
        <div key={illum.channel_id} class="illum-channel">
          <div
            class="illum-indicator"
            style={{ background: statusColor(illum.status) }}
          />
          <span class="illum-label">CH {illum.channel_id}</span>
          <span class="illum-assign">
            {illum.status === "Idle"
              ? "—"
              : illum.assigned_engagement !== null
                ? `ENG ${illum.assigned_engagement}`
                : "—"}
          </span>
          {illum.status === "TimeSharing" && (
            <span class="illum-ts">TS</span>
          )}
        </div>
      ))}
      {queueDepth > 0 && (
        <div class="illum-queue">QUEUE: {queueDepth}</div>
      )}
    </div>
  );
}
