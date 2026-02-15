//! Wave spawning system — spawns threat waves at scheduled times.

use hecs::World;
use rand_chacha::ChaCha8Rng;

use deterrence_core::enums::ThreatArchetype;

/// A single wave definition.
#[derive(Debug, Clone)]
pub struct WaveEntry {
    /// Tick at which this wave spawns.
    pub spawn_at_tick: u64,
    /// Threats to spawn: (archetype, count).
    pub threats: Vec<(ThreatArchetype, u32)>,
    /// Whether this wave has already been spawned.
    pub spawned: bool,
}

/// The complete wave schedule for a mission.
#[derive(Debug, Clone, Default)]
pub struct WaveSchedule {
    pub waves: Vec<WaveEntry>,
}

impl WaveSchedule {
    /// Default 3-wave mission with escalating difficulty.
    pub fn default_mission() -> Self {
        Self {
            waves: vec![
                WaveEntry {
                    spawn_at_tick: 0,
                    threats: vec![(ThreatArchetype::SeaSkimmerMk1, 3)],
                    spawned: false,
                },
                WaveEntry {
                    spawn_at_tick: 300, // 10 seconds
                    threats: vec![
                        (ThreatArchetype::SeaSkimmerMk1, 2),
                        (ThreatArchetype::SupersonicCruiser, 1),
                    ],
                    spawned: false,
                },
                WaveEntry {
                    spawn_at_tick: 600, // 20 seconds
                    threats: vec![
                        (ThreatArchetype::SeaSkimmerMk2, 2),
                        (ThreatArchetype::SupersonicCruiser, 1),
                        (ThreatArchetype::SubsonicDrone, 1),
                    ],
                    spawned: false,
                },
            ],
        }
    }

    /// Total number of threats across all waves.
    pub fn total_threats(&self) -> u32 {
        self.waves
            .iter()
            .flat_map(|w| w.threats.iter())
            .map(|(_, count)| count)
            .sum()
    }
}

/// Check schedule and spawn any due waves.
pub fn run(
    world: &mut World,
    rng: &mut ChaCha8Rng,
    schedule: &mut WaveSchedule,
    current_tick: u64,
) {
    for wave in &mut schedule.waves {
        if !wave.spawned && current_tick >= wave.spawn_at_tick {
            for &(archetype, count) in &wave.threats {
                for _ in 0..count {
                    crate::world_setup::spawn_threat(world, rng, archetype);
                }
            }
            wave.spawned = true;
        }
    }
}
