/**
 * TypeScript mirrors of Rust PlayerCommand variants.
 * These MUST be kept in sync with deterrence-core/src/commands.rs.
 */

import type {
  Classification,
  DoctrineMode,
  RadarMode,
  ScenarioId,
} from "./state";

export type PlayerCommand =
  | { type: "HookTrack"; track_number: number }
  | { type: "UnhookTrack" }
  | {
      type: "ClassifyTrack";
      track_number: number;
      classification: Classification;
    }
  | { type: "VetoEngagement"; engagement_id: number }
  | { type: "ConfirmEngagement"; engagement_id: number }
  | { type: "SetRadarSector"; center_bearing: number; width: number }
  | { type: "SetRadarMode"; mode: RadarMode }
  | { type: "SetDoctrine"; mode: DoctrineMode }
  | { type: "SetTimeScale"; scale: number }
  | { type: "SelectScenario"; scenario: ScenarioId }
  | { type: "StartMission" }
  | { type: "ReturnToMenu" }
  | { type: "Pause" }
  | { type: "Resume" };
