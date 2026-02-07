use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetonationEvent {
    pub entity_id: u32,
    pub x: f32,
    pub y: f32,
    pub yield_force: f32,
    pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactEvent {
    pub entity_id: u32,
    pub x: f32,
    pub y: f32,
    pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityDamagedEvent {
    pub city_id: u32,
    pub damage: f32,
    pub remaining_health: f32,
    pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveCompleteEvent {
    pub wave_number: u32,
    pub missiles_destroyed: u32,
    pub missiles_impacted: u32,
    pub interceptors_launched: u32,
    pub cities_remaining: u32,
    pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirvSplitEvent {
    pub carrier_id: u32,
    pub x: f32,
    pub y: f32,
    pub child_count: u32,
    pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    Detonation(DetonationEvent),
    Impact(ImpactEvent),
    CityDamaged(CityDamagedEvent),
    WaveComplete(WaveCompleteEvent),
    MirvSplit(MirvSplitEvent),
}
