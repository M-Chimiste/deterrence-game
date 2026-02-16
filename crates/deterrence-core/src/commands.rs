//! Player commands sent from the frontend to the simulation.
//!
//! Commands are validated and queued for processing at the next tick boundary.

use serde::{Deserialize, Serialize};

use crate::enums::*;

/// All possible player actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlayerCommand {
    // --- Track management ---
    /// Hook (select) a track for detailed inspection.
    HookTrack { track_number: u32 },
    /// Unhook the currently selected track.
    UnhookTrack,
    /// Manually classify a track.
    ClassifyTrack {
        track_number: u32,
        classification: Classification,
    },

    // --- Engagement management ---
    /// Veto an engagement (cancel before launch).
    VetoEngagement { engagement_id: u32 },
    /// Confirm an engagement (skip remaining veto timer).
    ConfirmEngagement { engagement_id: u32 },

    // --- Radar control ---
    /// Adjust the radar search sector.
    SetRadarSector { center_bearing: f64, width: f64 },
    /// Set radar operating mode.
    SetRadarMode { mode: RadarMode },

    // --- Doctrine ---
    /// Set engagement doctrine.
    SetDoctrine { mode: DoctrineMode },

    // --- Simulation control ---
    /// Set time scale (1.0 = normal, 2.0 = double, 0.0 = paused).
    SetTimeScale { scale: f64 },
    /// Select a scenario before starting a mission.
    SelectScenario { scenario: ScenarioId },
    /// Start a new mission with the selected scenario.
    StartMission,
    /// Return to the main menu from mission complete.
    ReturnToMenu,
    /// Pause the simulation.
    Pause,
    /// Resume the simulation.
    Resume,
}
