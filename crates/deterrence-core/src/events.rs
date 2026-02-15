//! Events emitted by the simulation for audio and UI feedback.

use serde::{Deserialize, Serialize};

use crate::enums::*;

/// Audio events for the frontend sound system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AudioEvent {
    /// New contact detected on radar.
    NewContact { bearing: f64, track_number: u32 },
    /// Contact lost (track dropped).
    ContactLost { track_number: u32 },
    /// Track classified as hostile.
    ThreatEvaluated {
        track_number: u32,
        classification: Classification,
    },
    /// Veto clock started for an engagement.
    VetoClockStart {
        engagement_id: u32,
        duration_secs: f64,
    },
    /// Veto clock warning (approaching expiry).
    VetoClockWarning {
        engagement_id: u32,
        remaining_secs: f64,
    },
    /// Missile launched.
    BirdAway { weapon_type: WeaponType },
    /// Intercept result.
    Splash {
        result: InterceptResult,
        track_number: u32,
    },
    /// Threat impact on defended asset.
    VampireImpact { bearing: f64 },
}

/// Alert for the UI alert queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub tick: u64,
}
