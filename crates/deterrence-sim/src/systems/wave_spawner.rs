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
    /// Fixed spawn bearing (radians). None = random.
    pub spawn_bearing: Option<f64>,
    /// Spawn range override (meters). None = random 150-180km.
    pub spawn_range: Option<f64>,
}

impl WaveEntry {
    /// Create a wave entry with default (random) bearing and range.
    pub fn new(spawn_at_tick: u64, threats: Vec<(ThreatArchetype, u32)>) -> Self {
        Self {
            spawn_at_tick,
            threats,
            spawned: false,
            spawn_bearing: None,
            spawn_range: None,
        }
    }

    /// Create a wave entry with a fixed bearing.
    pub fn with_bearing(
        spawn_at_tick: u64,
        threats: Vec<(ThreatArchetype, u32)>,
        bearing: f64,
    ) -> Self {
        Self {
            spawn_at_tick,
            threats,
            spawned: false,
            spawn_bearing: Some(bearing),
            spawn_range: None,
        }
    }

    /// Create a wave entry with a fixed bearing and range.
    pub fn with_bearing_and_range(
        spawn_at_tick: u64,
        threats: Vec<(ThreatArchetype, u32)>,
        bearing: f64,
        range: f64,
    ) -> Self {
        Self {
            spawn_at_tick,
            threats,
            spawned: false,
            spawn_bearing: Some(bearing),
            spawn_range: Some(range),
        }
    }
}

/// The complete wave schedule for a mission.
#[derive(Debug, Clone, Default)]
pub struct WaveSchedule {
    pub waves: Vec<WaveEntry>,
}

impl WaveSchedule {
    /// Default 3-wave mission with escalating difficulty (legacy).
    pub fn default_mission() -> Self {
        Self {
            waves: vec![
                WaveEntry::new(0, vec![(ThreatArchetype::SeaSkimmerMk1, 3)]),
                WaveEntry::new(
                    300,
                    vec![
                        (ThreatArchetype::SeaSkimmerMk1, 2),
                        (ThreatArchetype::SupersonicCruiser, 1),
                    ],
                ),
                WaveEntry::new(
                    600,
                    vec![
                        (ThreatArchetype::SeaSkimmerMk2, 2),
                        (ThreatArchetype::SupersonicCruiser, 1),
                        (ThreatArchetype::SubsonicDrone, 1),
                    ],
                ),
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

    /// Whether all waves have been spawned.
    pub fn all_spawned(&self) -> bool {
        self.waves.iter().all(|w| w.spawned)
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
                    match (wave.spawn_bearing, wave.spawn_range) {
                        (Some(bearing), Some(range)) => {
                            crate::world_setup::spawn_threat_at_bearing(
                                world, rng, archetype, bearing, range,
                            );
                        }
                        (Some(bearing), None) => {
                            crate::world_setup::spawn_threat_at_bearing_random_range(
                                world, rng, archetype, bearing,
                            );
                        }
                        _ => {
                            crate::world_setup::spawn_threat(world, rng, archetype);
                        }
                    }
                }
            }
            wave.spawned = true;
        }
    }
}
