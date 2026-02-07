export interface DetonationEvent {
  entity_id: number;
  x: number;
  y: number;
  yield_force: number;
  tick: number;
}

export interface ImpactEvent {
  entity_id: number;
  x: number;
  y: number;
  tick: number;
}

export interface CityDamagedEvent {
  city_id: number;
  damage: number;
  remaining_health: number;
  tick: number;
}

export interface WaveCompleteEvent {
  wave_number: number;
  missiles_destroyed: number;
  missiles_impacted: number;
  interceptors_launched: number;
  cities_remaining: number;
  tick: number;
}

export interface MirvSplitEvent {
  carrier_id: number;
  x: number;
  y: number;
  child_count: number;
  tick: number;
}
