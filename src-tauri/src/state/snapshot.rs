use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Missile,
    Interceptor,
    Shockwave,
    City,
    Battery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: u32,
    pub entity_type: EntityType,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub vx: f32,
    pub vy: f32,
    pub extra: Option<EntityExtra>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityExtra {
    Shockwave { radius: f32, max_radius: f32 },
    City { health: f32, max_health: f32 },
    Battery { ammo: u32, max_ammo: u32 },
    Interceptor { burn_remaining: f32, burn_time: f32, interceptor_type: String },
    Missile { is_mirv: bool, detected_by_radar: bool, detected_by_glow: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub tick: u64,
    pub wave_number: u32,
    pub phase: String,
    pub entities: Vec<EntitySnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weather: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind_x: Option<f32>,
}
