//! Engagement data model — tracks the lifecycle of a fire control engagement.
//!
//! Stored in `SimulationEngine`'s engagement map, NOT as ECS entities.

use deterrence_core::enums::{EngagementPhase, InterceptResult, WeaponType};
use deterrence_core::types::Position;

/// A fire control engagement linking a target to a weapon assignment.
#[derive(Debug, Clone)]
pub struct Engagement {
    pub id: u32,
    /// The hecs entity of the target.
    pub target_entity: hecs::Entity,
    /// The track number of the target.
    pub target_track_number: u32,
    /// Current engagement phase.
    pub phase: EngagementPhase,
    /// Weapon type assigned to this engagement.
    pub weapon_type: WeaponType,
    /// Estimated probability of kill.
    pub pk: f64,
    /// VLS cell index assigned (if any).
    pub assigned_cell: Option<usize>,

    // --- Timing ---
    /// Tick at which the current phase started.
    pub phase_start_tick: u64,
    /// Veto clock remaining seconds (only in Ready phase).
    pub veto_remaining_secs: f64,
    /// Veto clock total duration.
    pub veto_total_secs: f64,
    /// Whether the 3-second warning has been emitted.
    pub warned_3s: bool,
    /// Whether the 1-second warning has been emitted.
    pub warned_1s: bool,

    // --- Interceptor ---
    /// The hecs entity of the launched interceptor (if launched).
    pub interceptor_entity: Option<hecs::Entity>,
    /// Estimated time to intercept (seconds).
    pub time_to_intercept: f64,

    // --- Result ---
    /// Result of the engagement (set when Complete).
    pub result: Option<InterceptResult>,
    /// Predicted intercept point.
    pub pip: Position,
}

/// Running score state tracked by the engine.
#[derive(Debug, Clone, Default)]
pub struct ScoreState {
    pub threats_killed: u32,
    pub threats_total: u32,
    pub interceptors_fired: u32,
    pub threats_impacted: u32,
}
