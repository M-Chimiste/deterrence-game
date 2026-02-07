import { listen } from "@tauri-apps/api/event";
import type { StateSnapshot } from "../types/snapshot";
import type { DetonationEvent, ImpactEvent, CityDamagedEvent, WaveCompleteEvent, MirvSplitEvent } from "../types/events";
import type { CampaignSnapshot } from "../types/campaign";

export function onStateSnapshot(callback: (snapshot: StateSnapshot) => void) {
  return listen<StateSnapshot>("game:state_snapshot", (event) => {
    callback(event.payload);
  });
}

export function onDetonation(callback: (event: DetonationEvent) => void) {
  return listen<DetonationEvent>("game:detonation", (e) => {
    callback(e.payload);
  });
}

export function onImpact(callback: (event: ImpactEvent) => void) {
  return listen<ImpactEvent>("game:impact", (e) => {
    callback(e.payload);
  });
}

export function onCityDamaged(callback: (event: CityDamagedEvent) => void) {
  return listen<CityDamagedEvent>("game:city_damaged", (e) => {
    callback(e.payload);
  });
}

export function onWaveComplete(callback: (event: WaveCompleteEvent) => void) {
  return listen<WaveCompleteEvent>("game:wave_complete", (e) => {
    callback(e.payload);
  });
}

export function onMirvSplit(callback: (event: MirvSplitEvent) => void) {
  return listen<MirvSplitEvent>("game:mirv_split", (e) => {
    callback(e.payload);
  });
}

export function onCampaignUpdate(callback: (snapshot: CampaignSnapshot) => void) {
  return listen<CampaignSnapshot>("campaign:state_update", (e) => {
    callback(e.payload);
  });
}
