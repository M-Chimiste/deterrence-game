//! Game state snapshot — the complete visible state sent to the frontend each tick.

use serde::{Deserialize, Serialize};

use crate::enums::*;
use crate::events::{Alert, AudioEvent};
use crate::types::{Position, SimTime};

/// Complete game state broadcast to the frontend after each tick.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameStateSnapshot {
    pub time: SimTime,
    pub phase: GamePhase,
    pub doctrine: DoctrineMode,
    pub scenario: Option<ScenarioId>,
    pub tracks: Vec<TrackView>,
    pub engagements: Vec<EngagementView>,
    pub own_ship: OwnShipView,
    pub radar: RadarView,
    pub vls: VlsView,
    pub illuminators: Vec<IlluminatorView>,
    pub alerts: Vec<Alert>,
    pub audio_events: Vec<AudioEvent>,
    pub score: ScoreView,
}

/// A visible track on the tactical display.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackView {
    pub track_number: u32,
    pub position: Position,
    /// Bearing from own ship (radians, 0 = North).
    pub bearing: f64,
    /// Range from own ship (meters).
    pub range: f64,
    /// Altitude (meters).
    pub altitude: f64,
    /// Speed (m/s).
    pub speed: f64,
    /// Heading (radians, 0 = North).
    pub heading: f64,
    pub classification: Classification,
    pub iff_status: IffStatus,
    pub quality: f64,
    pub hooked: bool,
    /// Position history for trail dots.
    pub history: Vec<Position>,
}

/// Engagement status for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementView {
    pub engagement_id: u32,
    pub track_number: u32,
    pub phase: EngagementPhase,
    pub weapon_type: WeaponType,
    /// Probability of kill estimate (0.0 - 1.0).
    pub pk: f64,
    /// Veto clock remaining seconds (only relevant in Ready phase).
    pub veto_remaining_secs: f64,
    /// Veto clock total duration.
    pub veto_total_secs: f64,
    /// Assigned illuminator channel (if in terminal phase).
    pub illuminator_channel: Option<u8>,
    /// Time to intercept estimate (seconds).
    pub time_to_intercept: f64,
    /// Intercept result (if engagement is complete).
    pub result: Option<InterceptResult>,
}

/// Own ship/battery position and status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OwnShipView {
    pub position: Position,
}

/// Radar system status for display.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RadarView {
    pub mode: RadarMode,
    /// Total energy budget.
    pub energy_total: f64,
    /// Energy allocated to search.
    pub energy_search: f64,
    /// Energy allocated to tracking.
    pub energy_track: f64,
    /// Current sweep angle (radians) for PPI sweep line.
    pub sweep_angle: f64,
    /// Search sector center bearing (radians).
    pub sector_center: f64,
    /// Search sector width (radians).
    pub sector_width: f64,
    /// Number of active tracks consuming energy.
    pub active_track_count: u32,
}

/// VLS (launcher) status for display.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VlsView {
    pub cells: Vec<CellStatus>,
    /// Ready rounds by weapon type.
    pub ready_standard: u32,
    pub ready_extended_range: u32,
    pub ready_point_defense: u32,
    pub total_ready: u32,
    pub total_capacity: u32,
}

/// Illuminator channel status for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IlluminatorView {
    pub channel_id: u8,
    pub status: IlluminatorStatus,
    pub assigned_engagement: Option<u32>,
    /// Number of engagements waiting for this channel type.
    pub queue_depth: u32,
}

/// Running score for display.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreView {
    pub threats_killed: u32,
    pub threats_total: u32,
    pub interceptors_fired: u32,
    pub assets_protected: bool,
    pub mission_time_secs: f64,
}
