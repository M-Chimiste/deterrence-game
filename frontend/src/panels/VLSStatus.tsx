import { useGameStore } from "../store/gameState";
import type { CellStatus } from "../ipc/state";

function cellColor(cell: CellStatus): string {
  if (cell === "Expended") return "#ff3333";
  if (cell === "Empty") return "rgba(255,255,255,0.05)";
  if (typeof cell === "object") {
    if ("Ready" in cell) {
      switch (cell.Ready) {
        case "Standard":
          return "var(--cic-green)";
        case "ExtendedRange":
          return "#3388ff";
        case "PointDefense":
          return "#00cccc";
      }
    }
    if ("Assigned" in cell) return "#ffaa00";
  }
  return "rgba(255,255,255,0.05)";
}

export function VLSStatus() {
  const vls = useGameStore((s) => s.snapshot?.vls);
  if (!vls) return null;

  return (
    <div class="panel vls-panel">
      <div class="panel-header">VLS MAGAZINE</div>
      <div class="vls-grid">
        {vls.cells.map((cell, i) => (
          <div
            key={i}
            class="vls-cell"
            style={{ background: cellColor(cell) }}
          />
        ))}
      </div>
      <div class="vls-summary">
        SM: {vls.ready_standard} | ER: {vls.ready_extended_range} | PD:{" "}
        {vls.ready_point_defense} | RDY: {vls.total_ready}/{vls.total_capacity}
      </div>
    </div>
  );
}
