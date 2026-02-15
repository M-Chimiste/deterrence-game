/**
 * Tauri IPC bridge — wrappers around invoke() and listen().
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { PlayerCommand } from "./commands";
import type {
  GameStateSnapshot,
  Classification,
  DoctrineMode,
  RadarMode,
} from "./state";

/** Send a player command to the Rust simulation. */
export async function sendCommand(command: PlayerCommand): Promise<void> {
  await invoke("send_command", { command });
}

/** Start the simulation with the given configuration. */
export async function startSimulation(): Promise<void> {
  await invoke("start_simulation");
}

/** Get the latest snapshot synchronously (polling). */
export async function getSnapshot(): Promise<GameStateSnapshot | null> {
  return invoke<GameStateSnapshot | null>("get_snapshot");
}

/** Listen for game state snapshot events from the simulation. */
export async function onSnapshot(
  callback: (snapshot: GameStateSnapshot) => void,
): Promise<UnlistenFn> {
  return listen<GameStateSnapshot>("game:state_snapshot", (event) => {
    callback(event.payload);
  });
}

/** Pause the simulation. */
export async function pauseSimulation(): Promise<void> {
  await sendCommand({ type: "Pause" });
}

/** Resume the simulation. */
export async function resumeSimulation(): Promise<void> {
  await sendCommand({ type: "Resume" });
}

/** Set simulation time scale. */
export async function setTimeScale(scale: number): Promise<void> {
  await sendCommand({ type: "SetTimeScale", scale });
}

/** Start a mission. */
export async function startMission(): Promise<void> {
  await sendCommand({ type: "StartMission" });
}

/** Hook (select) a track for detailed inspection. */
export async function hookTrack(trackNumber: number): Promise<void> {
  await sendCommand({ type: "HookTrack", track_number: trackNumber });
}

/** Unhook the currently selected track. */
export async function unhookTrack(): Promise<void> {
  await sendCommand({ type: "UnhookTrack" });
}

/** Manually classify a track. */
export async function classifyTrack(
  trackNumber: number,
  classification: Classification,
): Promise<void> {
  await sendCommand({
    type: "ClassifyTrack",
    track_number: trackNumber,
    classification,
  });
}

/** Adjust the radar search sector. */
export async function setRadarSector(
  centerBearing: number,
  width: number,
): Promise<void> {
  await sendCommand({
    type: "SetRadarSector",
    center_bearing: centerBearing,
    width,
  });
}

/** Set radar operating mode. */
export async function setRadarMode(mode: RadarMode): Promise<void> {
  await sendCommand({ type: "SetRadarMode", mode });
}

/** Veto (abort) an engagement. */
export async function vetoEngagement(engagementId: number): Promise<void> {
  await sendCommand({ type: "VetoEngagement", engagement_id: engagementId });
}

/** Confirm an engagement (skip remaining veto clock). */
export async function confirmEngagement(engagementId: number): Promise<void> {
  await sendCommand({
    type: "ConfirmEngagement",
    engagement_id: engagementId,
  });
}

/** Set engagement doctrine mode. */
export async function setDoctrine(mode: DoctrineMode): Promise<void> {
  await sendCommand({ type: "SetDoctrine", mode });
}
