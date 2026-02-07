export interface CampaignSnapshot {
  resources: number;
  wave_number: number;
  owned_region_ids: number[];
  regions: RegionSnapshot[];
  available_actions: AvailableAction[];
  tech_tree: TechTreeSnapshot;
  wave_income?: number;
}

export interface TechTreeSnapshot {
  unlocked_types: string[];
  upgrades: TypeUpgradeSnapshot[];
}

export interface TypeUpgradeSnapshot {
  interceptor_type: string;
  thrust_level: number;
  yield_level: number;
  guidance_level: number;
}

export interface RegionSnapshot {
  id: number;
  name: string;
  terrain: string;
  owned: boolean;
  expandable: boolean;
  cities: CitySnapshotCampaign[];
  battery_slots: BatterySlotSnapshot[];
  map_x: number;
  map_y: number;
  expansion_cost: number;
}

export interface CitySnapshotCampaign {
  x: number;
  y: number;
  population: number;
  health: number;
  max_health: number;
}

export interface BatterySlotSnapshot {
  x: number;
  y: number;
  occupied: boolean;
  ammo: number | null;
  max_ammo: number | null;
}

export type AvailableAction =
  | { ExpandRegion: { region_id: number; cost: number } }
  | { PlaceBattery: { region_id: number; slot_index: number; cost: number } }
  | { RestockBattery: { region_id: number; slot_index: number; cost: number } }
  | { RepairCity: { region_id: number; city_index: number; cost: number; health_to_restore: number } }
  | { UnlockInterceptor: { interceptor_type: string; cost: number; min_wave: number } }
  | { UpgradeInterceptor: { interceptor_type: string; axis: string; cost: number; current_level: number } }
  | "StartWave";
