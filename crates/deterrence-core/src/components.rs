//! ECS components for hecs entities.
//!
//! Components are plain data structs with no methods.
//! Game logic lives in systems, not components.

use serde::{Deserialize, Serialize};

use crate::enums::*;
use crate::types::Position;

/// Radar cross section — how visible an entity is to radar.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RadarCrossSection {
    /// Base RCS in square meters. Smaller = harder to detect.
    pub base_rcs_m2: f64,
}

/// Track information maintained by the radar system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackInfo {
    /// Unique track number assigned by radar.
    pub track_number: u32,
    /// Track quality (0.0 = about to drop, 1.0 = firm).
    pub quality: f64,
    /// Current classification.
    pub classification: Classification,
    /// IFF interrogation status.
    pub iff_status: IffStatus,
    /// Whether this track is currently hooked (selected) by the player.
    pub hooked: bool,
    /// Number of consecutive radar sweeps with detection.
    pub hits: u32,
    /// Number of consecutive radar sweeps without detection.
    pub misses: u32,
}

/// Threat behavior profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatProfile {
    pub archetype: ThreatArchetype,
    pub phase: ThreatPhase,
    /// Target position this threat is heading toward.
    pub target: Position,
    /// Tick at which the current phase began (for timed transitions like pop-up).
    pub phase_start_tick: u64,
    /// Whether this threat has an active engagement against it.
    #[serde(default)]
    pub is_engaged: bool,
}

/// Interceptor missile state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissileState {
    pub phase: MissilePhase,
    /// Entity ID of the target track (if any).
    pub target_track: Option<u32>,
    /// Engagement ID this missile belongs to.
    pub engagement_id: u32,
    /// Remaining fuel (seconds of burn).
    pub fuel_secs: f64,
    /// Weapon type.
    pub weapon_type: WeaponType,
    /// Tick at which the current phase started.
    pub phase_start_tick: u64,
}

/// Radar system component (attached to own-ship or battery).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarSystem {
    /// Total energy budget (abstract units).
    pub energy_budget: f64,
    /// Energy currently allocated to search.
    pub search_energy: f64,
    /// Energy currently allocated to tracking.
    pub track_energy: f64,
    /// Search sector center bearing (radians, 0 = North).
    pub sector_center: f64,
    /// Search sector width (radians).
    pub sector_width: f64,
    /// Current operating mode.
    pub mode: RadarMode,
    /// Current sweep angle (radians, for PPI display).
    pub sweep_angle: f64,
}

/// Launcher system (VLS or similar).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherSystem {
    /// Status of each cell.
    pub cells: Vec<CellStatus>,
}

/// Illuminator channel for semi-active terminal guidance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Illuminator {
    pub channel_id: u8,
    pub status: IlluminatorStatus,
    /// Engagement ID currently being illuminated (if any).
    pub assigned_engagement: Option<u32>,
    /// Remaining dwell time on current assignment (seconds, for time-sharing rotation).
    pub dwell_remaining_secs: f64,
}

/// Marks an entity as the player's own ship/battery.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OwnShip;

/// Marks an entity as a threat (enemy missile/aircraft).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Threat;

/// Marks an entity as a friendly interceptor.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Interceptor;

/// Marks an entity as a civilian aircraft.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Civilian;

/// History of positions for trail rendering.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionHistory {
    /// Recent positions (newest first), up to MAX_HISTORY_DOTS.
    pub positions: Vec<Position>,
}

/// Pre-track detection accumulator.
/// Attached to entities that are detectable but not yet tracked.
/// After TRACK_INITIATE_HITS consecutive hits, entity is promoted to a full TrackInfo.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetectionCounter {
    /// Cumulative consecutive detection hits from radar sweeps.
    pub hits: u32,
    /// Cumulative consecutive detection misses.
    pub misses: u32,
    /// Tick of last sweep evaluation (prevents double-counting in one sweep).
    pub last_sweep_tick: u64,
}

// Re-export Position and Velocity as components too
// (they're defined in types.rs but used as ECS components)
