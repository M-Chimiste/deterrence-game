//! Archetype-specific behavioral profiles.
//!
//! Consolidates per-archetype parameters for the threat FSM.

use deterrence_core::enums::ThreatArchetype;

/// Behavioral profile for a threat archetype.
pub struct ThreatBehaviorProfile {
    /// Cruise speed (m/s).
    pub cruise_speed: f64,
    /// Cruise altitude (m).
    pub cruise_altitude: f64,
    /// Radar cross section (m²).
    pub rcs: f64,
    /// Range at which threat transitions to terminal phase (m).
    pub terminal_range: f64,
    /// Speed multiplier during terminal phase.
    pub terminal_speed_factor: f64,
    /// Range at which sea-skimmers begin pop-up (m), None for non-sea-skimmers.
    pub popup_range: Option<f64>,
    /// Pop-up altitude (m).
    pub popup_altitude: f64,
    /// Whether this archetype can perform evasive maneuvers when engaged.
    pub can_evade: bool,
    /// Whether terminal phase includes a dive (e.g., ballistic).
    pub terminal_dive: bool,
}

/// Get the behavioral profile for a given archetype.
pub fn get_profile(archetype: ThreatArchetype) -> ThreatBehaviorProfile {
    use deterrence_core::constants::*;

    match archetype {
        ThreatArchetype::SeaSkimmerMk1 => ThreatBehaviorProfile {
            cruise_speed: SEA_SKIMMER_SPEED,
            cruise_altitude: SEA_SKIMMER_ALTITUDE,
            rcs: SEA_SKIMMER_RCS,
            terminal_range: THREAT_TERMINAL_RANGE,
            terminal_speed_factor: THREAT_TERMINAL_SPEED_FACTOR,
            popup_range: Some(THREAT_POPUP_RANGE),
            popup_altitude: THREAT_POPUP_ALTITUDE,
            can_evade: false,
            terminal_dive: false,
        },
        ThreatArchetype::SeaSkimmerMk2 => ThreatBehaviorProfile {
            cruise_speed: SEA_SKIMMER_SPEED * 1.1,
            cruise_altitude: SEA_SKIMMER_ALTITUDE * 0.8,
            rcs: SEA_SKIMMER_RCS * 0.7,
            terminal_range: THREAT_TERMINAL_RANGE,
            terminal_speed_factor: THREAT_TERMINAL_SPEED_FACTOR,
            popup_range: Some(THREAT_POPUP_RANGE),
            popup_altitude: THREAT_POPUP_ALTITUDE,
            can_evade: true,
            terminal_dive: false,
        },
        ThreatArchetype::SupersonicCruiser => ThreatBehaviorProfile {
            cruise_speed: SUPERSONIC_CRUISER_SPEED,
            cruise_altitude: 5000.0,
            rcs: SUPERSONIC_CRUISER_RCS,
            terminal_range: THREAT_TERMINAL_RANGE,
            terminal_speed_factor: THREAT_TERMINAL_SPEED_FACTOR,
            popup_range: None,
            popup_altitude: 0.0,
            can_evade: false,
            terminal_dive: false,
        },
        ThreatArchetype::SubsonicDrone => ThreatBehaviorProfile {
            cruise_speed: 100.0,
            cruise_altitude: 3000.0,
            rcs: 0.5,
            terminal_range: 0.0, // no terminal phase, stays cruise until impact
            terminal_speed_factor: 1.0,
            popup_range: None,
            popup_altitude: 0.0,
            can_evade: false,
            terminal_dive: false,
        },
        ThreatArchetype::TacticalBallistic => ThreatBehaviorProfile {
            cruise_speed: 1500.0,
            cruise_altitude: 30_000.0,
            rcs: 1.0,
            terminal_range: THREAT_TERMINAL_RANGE,
            terminal_speed_factor: 1.5,
            popup_range: None,
            popup_altitude: 0.0,
            can_evade: false,
            terminal_dive: true,
        },
    }
}
