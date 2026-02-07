import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import type { PingResponse, ArcPrediction, SaveMetadata } from "../types/commands";

export async function ping(): Promise<PingResponse> {
  return await invoke<PingResponse>("ping");
}

export async function launchInterceptor(
  batteryId: number,
  targetX: number,
  targetY: number,
  interceptorType?: string
): Promise<void> {
  await invoke("launch_interceptor", {
    batteryId,
    targetX,
    targetY,
    interceptorType,
  });
}

export async function predictArc(
  batteryX: number,
  batteryY: number,
  targetX: number,
  targetY: number,
  interceptorType?: string,
  windX?: number
): Promise<ArcPrediction> {
  return await invoke<ArcPrediction>("predict_arc", {
    batteryX,
    batteryY,
    targetX,
    targetY,
    interceptorType,
    windX,
  });
}

export async function startWave(): Promise<void> {
  await invoke("start_wave");
}

export async function continueToStrategic(): Promise<void> {
  await invoke("continue_to_strategic");
}

export async function expandRegion(regionId: number): Promise<void> {
  await invoke("expand_region", { regionId });
}

export async function placeBattery(
  regionId: number,
  slotIndex: number
): Promise<void> {
  await invoke("place_battery", { regionId, slotIndex });
}

export async function restockBattery(batteryIndex: number): Promise<void> {
  await invoke("restock_battery", { batteryIndex });
}

export async function repairCity(cityIndex: number): Promise<void> {
  await invoke("repair_city", { cityIndex });
}

export async function unlockInterceptor(interceptorType: string): Promise<void> {
  await invoke("unlock_interceptor", { interceptorType });
}

export async function upgradeInterceptor(interceptorType: string, axis: string): Promise<void> {
  await invoke("upgrade_interceptor", { interceptorType, axis });
}

export async function getCampaignState(): Promise<void> {
  await invoke("get_campaign_state");
}

export async function newGame(): Promise<void> {
  await invoke("new_game");
}

export async function saveGame(slotName: string): Promise<void> {
  await invoke("save_game", { slotName });
}

export async function loadGame(slotName: string): Promise<void> {
  await invoke("load_game", { slotName });
}

export async function listSaves(): Promise<SaveMetadata[]> {
  return await invoke<SaveMetadata[]>("list_saves");
}

export async function deleteSave(slotName: string): Promise<void> {
  await invoke("delete_save", { slotName });
}

export async function setWindowResolution(width: number, height: number): Promise<void> {
  const win = getCurrentWindow();
  await win.setSize(new LogicalSize(width, height));
}

export async function setFullscreen(fullscreen: boolean): Promise<void> {
  const win = getCurrentWindow();
  await win.setFullscreen(fullscreen);
}

export async function isFullscreen(): Promise<boolean> {
  const win = getCurrentWindow();
  return await win.isFullscreen();
}
