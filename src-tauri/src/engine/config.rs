/// Fixed timestep: 60 Hz
pub const TICK_RATE: f32 = 60.0;
pub const DT: f32 = 1.0 / TICK_RATE;

/// Gravity (m/s²) — pointing downward (positive Y is up in our coordinate system)
pub const GRAVITY: f32 = 9.81;

/// World dimensions in meters
pub const WORLD_WIDTH: f32 = 1280.0;
pub const WORLD_HEIGHT: f32 = 720.0;
pub const GROUND_Y: f32 = 50.0;

/// Atmospheric drag model
/// Air density at sea level (kg/m³)
pub const AIR_DENSITY_SEA_LEVEL: f32 = 1.225;
/// Scale height for exponential density falloff (meters)
pub const ATMOSPHERE_SCALE_HEIGHT: f32 = 500.0;

/// Out-of-bounds margin for entity cleanup
pub const OOB_MARGIN: f32 = 200.0;

/// Standard interceptor defaults
/// High thrust + short burn = rapid acceleration then coast/decelerate
pub const INTERCEPTOR_THRUST: f32 = 500.0;
pub const INTERCEPTOR_BURN_TIME: f32 = 1.0;
pub const INTERCEPTOR_CEILING: f32 = 700.0;

/// Standard warhead defaults
pub const WARHEAD_YIELD: f32 = 100.0;
pub const WARHEAD_BLAST_RADIUS: f32 = 40.0;

/// Shockwave expansion rate (units/second)
pub const SHOCKWAVE_EXPANSION_RATE: f32 = 200.0;
/// Shockwave default max radius
pub const SHOCKWAVE_MAX_RADIUS: f32 = 60.0;
/// Shockwave visual linger time (ticks after full expansion before despawn)
pub const SHOCKWAVE_LIFETIME_TICKS: u32 = 30;

// --- Phase 3: World Layout ---

/// City positions (3 cities at ground level, evenly spaced)
pub const CITY_POSITIONS: [(f32, f32); 3] = [
    (320.0, GROUND_Y),
    (640.0, GROUND_Y),
    (960.0, GROUND_Y),
];
pub const CITY_MAX_HEALTH: f32 = 100.0;

/// Battery positions (2 batteries flanking the cities)
pub const BATTERY_POSITIONS: [(f32, f32); 2] = [
    (160.0, GROUND_Y),
    (1120.0, GROUND_Y),
];
pub const BATTERY_MAX_AMMO: u32 = 10;

// --- Interceptor ballistic properties ---
pub const INTERCEPTOR_MASS: f32 = 30.0;
pub const INTERCEPTOR_DRAG_COEFF: f32 = 0.35;
pub const INTERCEPTOR_CROSS_SECTION: f32 = 0.3;
/// Proximity threshold for interceptor detonation at target
pub const INTERCEPTOR_DETONATION_PROXIMITY: f32 = 15.0;

// --- Enemy missile properties ---
pub const MISSILE_MASS: f32 = 50.0;
pub const MISSILE_DRAG_COEFF: f32 = 0.3;
pub const MISSILE_CROSS_SECTION: f32 = 0.5;

// --- Wave spawning ---
pub const WAVE_BASE_MISSILES: u32 = 3;
pub const WAVE_MISSILES_PER_LEVEL: u32 = 2;
/// Ticks between missile spawns (1.5s at 60Hz)
pub const WAVE_BASE_SPAWN_INTERVAL: u32 = 90;
/// Min flight time in seconds (controls arc steepness)
pub const MISSILE_FLIGHT_TIME_MIN: f32 = 6.0;
/// Max flight time in seconds (controls arc height)
pub const MISSILE_FLIGHT_TIME_MAX: f32 = 12.0;

// --- Damage ---
pub const GROUND_IMPACT_BASE_DAMAGE: f32 = 50.0;
pub const GROUND_IMPACT_DAMAGE_RADIUS: f32 = 120.0;

// --- Interceptor Type Profiles ---
use crate::ecs::components::InterceptorType;

#[derive(Debug, Clone, Copy)]
pub struct InterceptorProfile {
    pub thrust: f32,
    pub burn_time: f32,
    pub ceiling: f32,
    pub mass: f32,
    pub drag_coeff: f32,
    pub cross_section: f32,
    pub yield_force: f32,
    pub blast_radius: f32,
}

/// Sprint: very fast burn, short range, small blast (terminal defense)
pub const SPRINT_THRUST: f32 = 900.0;
pub const SPRINT_BURN_TIME: f32 = 0.5;
pub const SPRINT_CEILING: f32 = 350.0;
pub const SPRINT_MASS: f32 = 15.0;
pub const SPRINT_DRAG_COEFF: f32 = 0.25;
pub const SPRINT_CROSS_SECTION: f32 = 0.2;
pub const SPRINT_YIELD: f32 = 60.0;
pub const SPRINT_BLAST_RADIUS: f32 = 25.0;

/// Exoatmospheric: slow launch, very high ceiling, wide high-altitude blast
pub const EXO_THRUST: f32 = 300.0;
pub const EXO_BURN_TIME: f32 = 2.5;
pub const EXO_CEILING: f32 = 900.0;
pub const EXO_MASS: f32 = 60.0;
pub const EXO_DRAG_COEFF: f32 = 0.4;
pub const EXO_CROSS_SECTION: f32 = 0.5;
pub const EXO_YIELD: f32 = 80.0;
pub const EXO_BLAST_RADIUS: f32 = 70.0;

/// AreaDenial: moderate speed, creates lingering shockwave zone
pub const AREA_DENIAL_THRUST: f32 = 400.0;
pub const AREA_DENIAL_BURN_TIME: f32 = 1.2;
pub const AREA_DENIAL_CEILING: f32 = 600.0;
pub const AREA_DENIAL_MASS: f32 = 40.0;
pub const AREA_DENIAL_DRAG_COEFF: f32 = 0.3;
pub const AREA_DENIAL_CROSS_SECTION: f32 = 0.4;
pub const AREA_DENIAL_YIELD: f32 = 50.0;
pub const AREA_DENIAL_BLAST_RADIUS: f32 = 55.0;
pub const AREA_DENIAL_LINGER_TICKS: u32 = 180;
pub const AREA_DENIAL_EXPANSION_RATE: f32 = 80.0;

// --- Chain Reaction / Shockwave Collision ---
/// Ratio of shockwave radius that is the "destroy" zone (inner). Beyond this is deflect zone.
pub const SHOCKWAVE_DESTROY_RATIO: f32 = 0.7;
/// Multiplier for chain reaction shockwave power (radius and force)
pub const CHAIN_REACTION_MULTIPLIER: f32 = 0.7;
/// Force multiplier for deflection in the outer shockwave zone
pub const SHOCKWAVE_DEFLECT_FORCE: f32 = 0.1;

// --- MIRV defaults ---
pub const MIRV_SPLIT_ALTITUDE_MIN: f32 = 300.0;
pub const MIRV_SPLIT_ALTITUDE_MAX: f32 = 400.0;
pub const MIRV_SPREAD_ANGLE: f32 = 0.5; // radians
pub const MIRV_CHILD_YIELD: f32 = 80.0;
pub const MIRV_CHILD_BLAST_RADIUS: f32 = 30.0;
pub const MIRV_DEFAULT_CHILD_COUNT: u32 = 3;
pub const MIRV_FIRST_WAVE: u32 = 26;

// --- Weather + Wind ---
/// First wave where weather effects can appear
pub const WEATHER_FIRST_WAVE: u32 = 16;
/// Wind speeds for each weather condition (m/s)
pub const WIND_SPEED_OVERCAST: f32 = 5.0;
pub const WIND_SPEED_STORM: f32 = 15.0;
pub const WIND_SPEED_SEVERE: f32 = 30.0;
/// Wind scales linearly with altitude: wind_effect = wind_speed * y * WIND_ALTITUDE_FACTOR
pub const WIND_ALTITUDE_FACTOR: f32 = 0.003;
/// Storm/Severe missile count multipliers
pub const STORM_MISSILE_MULT: f32 = 1.15;
pub const SEVERE_MISSILE_MULT: f32 = 1.3;

// --- Radar / Detection ---
/// Base radar detection range from any battery (in world units)
pub const RADAR_BASE_RANGE: f32 = 500.0;
/// Radar range multipliers per weather condition
pub const RADAR_MULT_CLEAR: f32 = 1.0;
pub const RADAR_MULT_OVERCAST: f32 = 0.85;
pub const RADAR_MULT_STORM: f32 = 0.6;
pub const RADAR_MULT_SEVERE: f32 = 0.4;
/// Glow visibility per weather condition (0 = glow invisible)
pub const GLOW_VIS_CLEAR: f32 = 1.0;
pub const GLOW_VIS_OVERCAST: f32 = 0.3;
pub const GLOW_VIS_STORM: f32 = 0.0;
pub const GLOW_VIS_SEVERE: f32 = 0.0;

pub fn interceptor_profile(itype: InterceptorType) -> InterceptorProfile {
    match itype {
        InterceptorType::Standard => InterceptorProfile {
            thrust: INTERCEPTOR_THRUST,
            burn_time: INTERCEPTOR_BURN_TIME,
            ceiling: INTERCEPTOR_CEILING,
            mass: INTERCEPTOR_MASS,
            drag_coeff: INTERCEPTOR_DRAG_COEFF,
            cross_section: INTERCEPTOR_CROSS_SECTION,
            yield_force: WARHEAD_YIELD,
            blast_radius: WARHEAD_BLAST_RADIUS,
        },
        InterceptorType::Sprint => InterceptorProfile {
            thrust: SPRINT_THRUST,
            burn_time: SPRINT_BURN_TIME,
            ceiling: SPRINT_CEILING,
            mass: SPRINT_MASS,
            drag_coeff: SPRINT_DRAG_COEFF,
            cross_section: SPRINT_CROSS_SECTION,
            yield_force: SPRINT_YIELD,
            blast_radius: SPRINT_BLAST_RADIUS,
        },
        InterceptorType::Exoatmospheric => InterceptorProfile {
            thrust: EXO_THRUST,
            burn_time: EXO_BURN_TIME,
            ceiling: EXO_CEILING,
            mass: EXO_MASS,
            drag_coeff: EXO_DRAG_COEFF,
            cross_section: EXO_CROSS_SECTION,
            yield_force: EXO_YIELD,
            blast_radius: EXO_BLAST_RADIUS,
        },
        InterceptorType::AreaDenial => InterceptorProfile {
            thrust: AREA_DENIAL_THRUST,
            burn_time: AREA_DENIAL_BURN_TIME,
            ceiling: AREA_DENIAL_CEILING,
            mass: AREA_DENIAL_MASS,
            drag_coeff: AREA_DENIAL_DRAG_COEFF,
            cross_section: AREA_DENIAL_CROSS_SECTION,
            yield_force: AREA_DENIAL_YIELD,
            blast_radius: AREA_DENIAL_BLAST_RADIUS,
        },
    }
}
