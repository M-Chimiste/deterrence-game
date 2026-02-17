//! Simulation constants and tuning parameters.

/// Simulation tick rate (Hz).
pub const TICK_RATE: u32 = 30;

/// Seconds per tick.
pub const DT: f64 = 1.0 / TICK_RATE as f64;

// --- World bounds ---

/// Simulation area width in meters (~100 nautical miles).
pub const WORLD_RADIUS: f64 = 185_000.0;

// --- Radar ---

/// Base radar power (abstract energy units).
pub const RADAR_TOTAL_ENERGY: f64 = 100.0;

/// Energy cost per tracked contact.
pub const RADAR_ENERGY_PER_TRACK: f64 = 2.0;

/// Minimum search energy before detection degrades severely.
pub const RADAR_MIN_SEARCH_ENERGY: f64 = 10.0;

/// Maximum detection range in meters (~200 nm for SPY-1 class).
pub const RADAR_MAX_RANGE: f64 = 370_000.0;

/// Sweep rate (radians per second) — full rotation in ~4 seconds.
pub const RADAR_SWEEP_RATE: f64 = std::f64::consts::TAU / 4.0;

/// Default search sector width (full 360°).
pub const RADAR_DEFAULT_SECTOR_WIDTH: f64 = std::f64::consts::TAU;

/// Radar beam half-width in sweep-tick multiples.
/// The beam checks entities within sweep_rate * dt * this factor of the sweep angle.
pub const RADAR_BEAM_HALF_WIDTH_TICKS: f64 = 1.5;

/// Radar equation calibration constant (RADAR_MAX_RANGE^4).
/// Chosen so SNR=1.0 at max range for a 1 m² target with full search energy.
pub const RADAR_K: f64 = RADAR_MAX_RANGE * RADAR_MAX_RANGE * RADAR_MAX_RANGE * RADAR_MAX_RANGE;

/// Minimum range below which detection is guaranteed (meters).
pub const RADAR_MIN_RANGE: f64 = 500.0;

// --- Tracking ---

/// Number of consecutive detections to initiate a track.
pub const TRACK_INITIATE_HITS: u32 = 3;

/// Number of consecutive misses to drop a track.
pub const TRACK_DROP_MISSES: u32 = 5;

/// Track quality increase per detection.
pub const TRACK_QUALITY_HIT_GAIN: f64 = 0.15;

/// Track quality decrease per missed sweep.
pub const TRACK_QUALITY_MISS_LOSS: f64 = 0.10;

/// Maximum track quality.
pub const TRACK_QUALITY_MAX: f64 = 1.0;

/// Quality threshold for "firm" track (eligible for engagement).
pub const TRACK_FIRM_QUALITY: f64 = 0.6;

/// Quality assigned to a newly initiated track.
pub const TRACK_INITIAL_QUALITY: f64 = 0.5;

// --- Fire control ---

/// Default veto clock duration in seconds (AUTO-SPECIAL).
pub const VETO_CLOCK_DURATION: f64 = 8.0;

/// Solution calculation time in seconds.
pub const SOLUTION_CALC_TIME: f64 = 2.0;

/// BDA assessment delay after intercept (seconds).
pub const BDA_DELAY: f64 = 1.5;

// --- Illuminators ---

/// Number of illuminator channels.
pub const ILLUMINATOR_COUNT: u8 = 3;

/// Duration an illuminator must dwell on a target (seconds).
pub const ILLUMINATOR_DWELL_TIME: f64 = 3.0;

// --- Missile kinematics ---

/// Boost phase duration for all weapon types (seconds).
pub const MISSILE_BOOST_DURATION_SECS: f64 = 5.0;

/// Range from target at which interceptor enters terminal guidance (meters).
pub const TERMINAL_GUIDANCE_RANGE: f64 = 20_000.0;

// --- VLS ---

/// Default number of VLS cells.
pub const VLS_CELL_COUNT: usize = 64;

// --- Interceptor performance ---

/// Standard missile speed (m/s) — ~Mach 3.5.
pub const STANDARD_MISSILE_SPEED: f64 = 1200.0;

/// Standard missile max range (meters) — ~90 nm.
pub const STANDARD_MISSILE_MAX_RANGE: f64 = 167_000.0;

/// Standard missile boost duration (seconds).
pub const STANDARD_MISSILE_BOOST_TIME: f64 = 5.0;

/// Lethal radius for intercept (meters).
pub const INTERCEPT_LETHAL_RADIUS: f64 = 20.0;

// --- Threat defaults ---

/// Subsonic sea-skimmer speed (m/s) — ~Mach 0.85.
pub const SEA_SKIMMER_SPEED: f64 = 290.0;

/// Sea-skimmer cruise altitude (meters).
pub const SEA_SKIMMER_ALTITUDE: f64 = 15.0;

/// Sea-skimmer RCS (square meters).
pub const SEA_SKIMMER_RCS: f64 = 0.1;

/// Supersonic cruiser speed (m/s) — ~Mach 2.5.
pub const SUPERSONIC_CRUISER_SPEED: f64 = 850.0;

/// Supersonic cruiser RCS (square meters).
pub const SUPERSONIC_CRUISER_RCS: f64 = 3.0;

// --- Threat AI ---

/// Range from target at which sea-skimmers begin pop-up maneuver (meters).
pub const THREAT_POPUP_RANGE: f64 = 50_000.0;

/// Range from target at which threats transition to terminal phase (meters).
pub const THREAT_TERMINAL_RANGE: f64 = 30_000.0;

/// Duration of pop-up maneuver (seconds).
pub const THREAT_POPUP_DURATION_SECS: f64 = 3.0;

/// Pop-up altitude for sea-skimmers (meters).
pub const THREAT_POPUP_ALTITUDE: f64 = 300.0;

/// Speed multiplier during terminal phase.
pub const THREAT_TERMINAL_SPEED_FACTOR: f64 = 1.2;

/// Range at which threat is considered to have impacted target (meters).
pub const THREAT_IMPACT_RANGE: f64 = 50.0;

// --- Wave spawning ---

/// Default interval between waves (ticks). ~10 seconds at 30Hz.
pub const WAVE_INTERVAL_TICKS: u64 = 300;

// --- Fire control (additional) ---

/// Veto clock warning threshold — first warning (seconds remaining).
pub const VETO_WARNING_THRESHOLD_1: f64 = 3.0;

/// Veto clock warning threshold — final warning (seconds remaining).
pub const VETO_WARNING_THRESHOLD_2: f64 = 1.0;

/// Maximum simultaneous engagements.
pub const MAX_SIMULTANEOUS_ENGAGEMENTS: usize = 12;

// --- Pk model ---

/// Base Pk for Standard missile (SM-2 equiv).
pub const PK_STANDARD_BASE: f64 = 0.70;

/// Base Pk for Extended Range missile (SM-6 equiv).
pub const PK_EXTENDED_RANGE_BASE: f64 = 0.65;

/// Base Pk for Point Defense missile (ESSM equiv).
pub const PK_POINT_DEFENSE_BASE: f64 = 0.80;

// --- Extended range interceptor performance ---

/// Extended range missile speed (m/s).
pub const EXTENDED_RANGE_MISSILE_SPEED: f64 = 1400.0;

/// Extended range missile max range (meters).
pub const EXTENDED_RANGE_MISSILE_MAX_RANGE: f64 = 250_000.0;

// --- Point defense interceptor performance ---

/// Point defense missile speed (m/s).
pub const POINT_DEFENSE_MISSILE_SPEED: f64 = 1000.0;

/// Point defense missile max range (meters).
pub const POINT_DEFENSE_MISSILE_MAX_RANGE: f64 = 30_000.0;

// --- Proportional navigation ---

/// Navigation constant for PN guidance (dimensionless, typically 3-5).
pub const PN_NAVIGATION_CONSTANT: f64 = 4.0;

/// Maximum turn rate for PN-guided missiles (rad/s, ~28.6 deg/s).
pub const PN_MAX_TURN_RATE: f64 = 0.5;

// --- Terrain ---

/// Line-of-sight sampling interval in meters.
pub const TERRAIN_LOS_SAMPLE_INTERVAL: f64 = 100.0;

/// Mean Earth radius in meters.
pub const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Effective Earth radius for radar propagation (4/3 model for atmospheric refraction).
pub const EFFECTIVE_EARTH_RADIUS_M: f64 = EARTH_RADIUS_M * 4.0 / 3.0;

// --- Display ---

/// Maximum number of position history dots per track.
pub const MAX_HISTORY_DOTS: usize = 12;

/// History dot interval in ticks (one dot every N ticks).
pub const HISTORY_DOT_INTERVAL: u32 = 15;
