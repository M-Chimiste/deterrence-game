use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ballistic {
    pub drag_coefficient: f32,
    pub mass: f32,
    pub cross_section: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarheadType {
    Standard,
    Mirv,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Warhead {
    pub yield_force: f32,
    pub blast_radius_base: f32,
    pub warhead_type: WarheadType,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterceptorType {
    #[default]
    Standard,
    Sprint,
    Exoatmospheric,
    AreaDenial,
}

impl InterceptorType {
    pub fn parse(s: &str) -> Self {
        match s {
            "Sprint" => InterceptorType::Sprint,
            "Exoatmospheric" => InterceptorType::Exoatmospheric,
            "AreaDenial" => InterceptorType::AreaDenial,
            _ => InterceptorType::Standard,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            InterceptorType::Standard => "Standard",
            InterceptorType::Sprint => "Sprint",
            InterceptorType::Exoatmospheric => "Exoatmospheric",
            InterceptorType::AreaDenial => "AreaDenial",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Interceptor {
    pub interceptor_type: InterceptorType,
    pub thrust: f32,
    pub burn_time: f32,
    pub burn_remaining: f32,
    pub ceiling: f32,
    pub battery_id: u32,
    pub target_x: f32,
    pub target_y: f32,
    /// Proximity fuse: auto-detonate when within this radius of any missile. 0.0 = disabled.
    pub proximity_fuse_radius: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MirvCarrier {
    pub child_count: u32,
    pub split_altitude: f32,
    pub spread_angle: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Lifetime {
    pub remaining_ticks: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReentryGlow {
    pub intensity: f32,
    pub altitude_threshold: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Shockwave {
    pub radius: f32,
    pub max_radius: f32,
    pub force: f32,
    pub expansion_rate: f32,
    pub damage_applied: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BatteryState {
    pub ammo: u32,
    pub max_ammo: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    Missile,
    Interceptor,
    Shockwave,
    City,
    Battery,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntityMarker {
    pub kind: EntityKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Detected {
    pub by_radar: bool,
    pub by_glow: bool,
}
