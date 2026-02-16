/**
 * Scenario selection screen — choose Easy, Medium, or Hard.
 */

import { selectScenario, startMission } from "../ipc/bridge";
import type { ScenarioId } from "../ipc/state";

interface ScenarioInfo {
  id: ScenarioId;
  name: string;
  subtitle: string;
  threats: string;
  description: string;
}

const SCENARIOS: ScenarioInfo[] = [
  {
    id: "Easy",
    name: "TRAINING EXERCISE",
    subtitle: "EASY",
    threats: "8 THREATS / 3 WAVES",
    description: "Single axis attack from the North. Sea-skimmers and drones only.",
  },
  {
    id: "Medium",
    name: "MULTI-AXIS RAID",
    subtitle: "MEDIUM",
    threats: "12 THREATS / 5 WAVES",
    description: "Dual axis attack. Supersonic cruisers with time-on-top coordination.",
  },
  {
    id: "Hard",
    name: "SATURATION ATTACK",
    subtitle: "HARD",
    threats: "21 THREATS / 7 WAVES",
    description: "Three-axis saturation raid. Ballistic missiles and massed sea-skimmers.",
  },
];

export function ScenarioSelect() {
  async function handleSelect(id: ScenarioId) {
    await selectScenario(id);
    await startMission();
  }

  return (
    <div class="scenario-select">
      <h2 class="scenario-header">SELECT SCENARIO</h2>
      <div class="scenario-cards">
        {SCENARIOS.map((s) => (
          <button
            key={s.id}
            class="scenario-card"
            onClick={() => handleSelect(s.id)}
          >
            <div class="scenario-subtitle">{s.subtitle}</div>
            <div class="scenario-name">{s.name}</div>
            <div class="scenario-threats">{s.threats}</div>
            <div class="scenario-desc">{s.description}</div>
          </button>
        ))}
      </div>
    </div>
  );
}
