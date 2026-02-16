//! Scenario definitions — hardcoded mission wave schedules.
//!
//! Each scenario defines wave composition, timing, spawn bearings,
//! and difficulty progression.

use std::f64::consts::PI;

use deterrence_core::constants::TICK_RATE;
use deterrence_core::enums::{ScenarioId, ThreatArchetype};

use crate::systems::wave_spawner::{WaveEntry, WaveSchedule};

/// Build the wave schedule for a given scenario.
pub fn build_schedule(scenario: ScenarioId) -> WaveSchedule {
    match scenario {
        ScenarioId::Easy => build_easy(),
        ScenarioId::Medium => build_medium(),
        ScenarioId::Hard => build_hard(),
    }
}

/// Easy: "Training Exercise"
/// 3 waves, 8 total threats, single axis (North), 20s spacing.
/// Only SeaSkimmerMk1 and SubsonicDrone.
fn build_easy() -> WaveSchedule {
    let north = 0.0; // 0 radians = North

    WaveSchedule {
        waves: vec![
            // Wave 1 (t=0): 3x SeaSkimmerMk1 from North
            WaveEntry::with_bearing(0, vec![(ThreatArchetype::SeaSkimmerMk1, 3)], north),
            // Wave 2 (t=20s): 2x SeaSkimmerMk1 + 1x SubsonicDrone from North
            WaveEntry::with_bearing(
                secs_to_ticks(20.0),
                vec![
                    (ThreatArchetype::SeaSkimmerMk1, 2),
                    (ThreatArchetype::SubsonicDrone, 1),
                ],
                north,
            ),
            // Wave 3 (t=40s): 2x SeaSkimmerMk1 from North
            WaveEntry::with_bearing(
                secs_to_ticks(40.0),
                vec![(ThreatArchetype::SeaSkimmerMk1, 2)],
                north,
            ),
        ],
    }
}

/// Medium: "Multi-Axis Raid"
/// 5 waves, 15 total threats, dual axis (North + East).
/// Includes SupersonicCruiser. Time-on-top coordination on wave 3.
fn build_medium() -> WaveSchedule {
    let north = 0.0;
    let east = PI / 2.0;

    // Time-on-top calculation for wave 3:
    // SeaSkimmerMk1 at 165km, ~290 m/s → arrival in ~569s
    // SupersonicCruiser at 180km, ~850 m/s → arrival in ~212s
    // To arrive together, offset supersonic spawn by ~357s later
    // But we use wave-level timing, so we spawn the supersonic wave later.
    let tot_offset = secs_to_ticks(357.0);

    WaveSchedule {
        waves: vec![
            // Wave 1 (t=0): 3x SeaSkimmerMk1 from North
            WaveEntry::with_bearing(0, vec![(ThreatArchetype::SeaSkimmerMk1, 3)], north),
            // Wave 2 (t=15s): 2x SeaSkimmerMk1 from East
            WaveEntry::with_bearing(
                secs_to_ticks(15.0),
                vec![(ThreatArchetype::SeaSkimmerMk1, 2)],
                east,
            ),
            // Wave 3a (t=30s): 3x SeaSkimmerMk1 from North (time-on-top pair)
            WaveEntry::with_bearing_and_range(
                secs_to_ticks(30.0),
                vec![(ThreatArchetype::SeaSkimmerMk1, 3)],
                north,
                165_000.0,
            ),
            // Wave 3b (t=30s + offset): 2x SupersonicCruiser from East (arrives same time as 3a)
            WaveEntry::with_bearing_and_range(
                secs_to_ticks(30.0) + tot_offset,
                vec![(ThreatArchetype::SupersonicCruiser, 2)],
                east,
                180_000.0,
            ),
            // Wave 4 (t=60s): 2x SeaSkimmerMk2 from North
            WaveEntry::with_bearing(
                secs_to_ticks(60.0),
                vec![(ThreatArchetype::SeaSkimmerMk2, 2)],
                north,
            ),
        ],
    }
}

/// Hard: "Saturation Attack"
/// 7 waves, 25+ total threats, 3 axes (North, East, Southwest).
/// Includes TacticalBallistic, saturation waves, tight timing.
fn build_hard() -> WaveSchedule {
    let north = 0.0;
    let east = PI / 2.0;
    let southwest = PI * 1.25; // 225 degrees

    // Time-on-top: SupersonicCruiser from east + SeaSkimmerMk1 from north
    // synchronized arrival for wave 4
    let tot_offset = secs_to_ticks(357.0);

    WaveSchedule {
        waves: vec![
            // Wave 1 (t=0): 3x SeaSkimmerMk1 from North
            WaveEntry::with_bearing(0, vec![(ThreatArchetype::SeaSkimmerMk1, 3)], north),
            // Wave 2 (t=10s): 3x SeaSkimmerMk1 from East
            WaveEntry::with_bearing(
                secs_to_ticks(10.0),
                vec![(ThreatArchetype::SeaSkimmerMk1, 3)],
                east,
            ),
            // Wave 3 (t=20s): 2x SeaSkimmerMk2 + 1x SubsonicDrone from Southwest
            WaveEntry::with_bearing(
                secs_to_ticks(20.0),
                vec![
                    (ThreatArchetype::SeaSkimmerMk2, 2),
                    (ThreatArchetype::SubsonicDrone, 1),
                ],
                southwest,
            ),
            // Wave 4a (t=35s): 4x SeaSkimmerMk1 from North — saturation wave
            WaveEntry::with_bearing_and_range(
                secs_to_ticks(35.0),
                vec![(ThreatArchetype::SeaSkimmerMk1, 4)],
                north,
                165_000.0,
            ),
            // Wave 4b (t=35s + offset): 2x SupersonicCruiser from East — time-on-top
            WaveEntry::with_bearing_and_range(
                secs_to_ticks(35.0) + tot_offset,
                vec![(ThreatArchetype::SupersonicCruiser, 2)],
                east,
                180_000.0,
            ),
            // Wave 5 (t=50s): 1x TacticalBallistic from Southwest
            WaveEntry::with_bearing_and_range(
                secs_to_ticks(50.0),
                vec![(ThreatArchetype::TacticalBallistic, 1)],
                southwest,
                180_000.0,
            ),
            // Wave 6 (t=65s): 5x SeaSkimmerMk2 from North — final saturation
            WaveEntry::with_bearing(
                secs_to_ticks(65.0),
                vec![(ThreatArchetype::SeaSkimmerMk2, 5)],
                north,
            ),
        ],
    }
}

/// Convert seconds to ticks.
fn secs_to_ticks(secs: f64) -> u64 {
    (secs * TICK_RATE as f64) as u64
}
