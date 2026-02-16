/**
 * Current doctrine mode indicator panel.
 */

import { useGameStore } from "../store/gameState";

const DOCTRINE_LABELS: Record<string, string> = {
  AutoSpecial: "AUTO-SPECIAL",
  AutoComposite: "AUTO-COMPOSITE",
  Manual: "MANUAL",
};

const DOCTRINE_COLORS: Record<string, string> = {
  AutoSpecial: "var(--cic-green)",
  AutoComposite: "#ffaa00",
  Manual: "#ff3333",
};

export function DoctrineDisplay() {
  const doctrine = useGameStore((s) => s.snapshot?.doctrine ?? "AutoSpecial");

  return (
    <div class="panel doctrine-panel">
      <div class="panel-header">DOCTRINE</div>
      <div
        class="doctrine-value"
        style={{ color: DOCTRINE_COLORS[doctrine] ?? "var(--cic-green)" }}
      >
        {DOCTRINE_LABELS[doctrine] ?? doctrine}
      </div>
    </div>
  );
}
