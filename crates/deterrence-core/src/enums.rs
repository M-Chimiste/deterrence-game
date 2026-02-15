//! Enumeration types used throughout the simulation.

use serde::{Deserialize, Serialize};

/// Track classification following NATO identification doctrine.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Classification {
    #[default]
    Unknown,
    Pending,
    AssumedFriend,
    Friend,
    Neutral,
    Suspect,
    Hostile,
}

/// IFF interrogation status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum IffStatus {
    /// Not yet interrogated.
    #[default]
    NoResponse,
    /// Friendly response received.
    FriendlyResponse,
    /// Hostile / no valid response after interrogation.
    NoValidResponse,
    /// Conflicting or suspicious response.
    Suspicious,
}

/// Engagement doctrine mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoctrineMode {
    /// Player must manually authorize every engagement.
    Manual,
    /// System auto-engages hostiles with a veto window.
    #[default]
    AutoSpecial,
    /// System auto-engages hostiles and suspects with a veto window.
    AutoComposite,
}

/// Radar operating mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RadarMode {
    /// Full search, no dedicated tracking.
    Search,
    /// Track-while-scan (default operating mode).
    #[default]
    TrackWhileScan,
    /// Burn-through mode: concentrated energy on specific bearing.
    BurnThrough,
}

/// Weapon type for interceptors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponType {
    /// Standard missile (SM-2 equivalent): semi-active terminal, medium range.
    #[default]
    Standard,
    /// Extended range missile (SM-6 equivalent): active terminal, long range.
    ExtendedRange,
    /// Point defense (ESSM equivalent): short range, fast reaction.
    PointDefense,
}

/// Engagement lifecycle phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngagementPhase {
    /// Computing fire control solution.
    SolutionCalc,
    /// Solution ready, veto clock counting down.
    Ready,
    /// Missile launched, in boost phase.
    Launched,
    /// Missile in midcourse (command guided).
    Midcourse,
    /// Missile in terminal phase (seeker active, illuminator required for semi-active).
    Terminal,
    /// Intercept evaluated, awaiting BDA.
    Intercept,
    /// Engagement complete with result.
    Complete,
    /// Engagement aborted (vetoed, lost track, etc.).
    Aborted,
}

/// Threat archetype category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatArchetype {
    /// Subsonic anti-ship cruise missile, sea-skimming.
    SeaSkimmerMk1,
    /// Improved sea-skimmer with terminal weave.
    SeaSkimmerMk2,
    /// Supersonic cruise missile, high RCS but fast.
    SupersonicCruiser,
    /// Subsonic reconnaissance drone.
    SubsonicDrone,
    /// Tactical ballistic missile.
    TacticalBallistic,
}

/// Threat behavior phase.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatPhase {
    /// Inbound at cruise altitude/speed.
    #[default]
    Cruise,
    /// Climbing for radar acquisition before terminal dive.
    PopUp,
    /// Final attack run, typically sea-skimming or diving.
    Terminal,
    /// Evasive maneuvers (weave, jink).
    Evasive,
    /// Destroyed by intercept.
    Destroyed,
    /// Reached target (impact).
    Impact,
}

/// Missile (interceptor) flight phase.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissilePhase {
    /// High-G boost out of launcher.
    #[default]
    Boost,
    /// Command-guided flight toward predicted intercept point.
    Midcourse,
    /// Seeker-active terminal phase (requires illuminator for semi-active).
    Terminal,
    /// Intercept evaluated.
    Complete,
}

/// Illuminator channel status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum IlluminatorStatus {
    /// Channel available.
    #[default]
    Idle,
    /// Actively illuminating for a terminal engagement.
    Active,
    /// Time-sharing between multiple engagements.
    TimeSharing,
}

/// VLS cell status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellStatus {
    /// Missile loaded and ready to fire.
    Ready(WeaponType),
    /// Cell assigned to an engagement, awaiting launch command.
    Assigned(WeaponType),
    /// Missile has been fired, cell empty.
    Expended,
    /// Cell is empty (no missile loaded).
    Empty,
}

/// Intercept result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterceptResult {
    Hit,
    Miss,
}

/// Game phase (top-level state).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    #[default]
    MainMenu,
    MissionBriefing,
    Active,
    Paused,
    MissionComplete,
}

/// Alert severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}
