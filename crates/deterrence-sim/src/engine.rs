//! Simulation engine — the core of the game.
//!
//! `SimulationEngine` owns the hecs ECS world, processes player commands,
//! runs all systems, and produces `GameStateSnapshot`s. Completely headless
//! (no Tauri dependency), enabling deterministic testing.

use std::collections::{HashMap, VecDeque};

use hecs::World;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use deterrence_core::commands::PlayerCommand;
use deterrence_core::components::{OwnShip, RadarSystem, TrackInfo};
use deterrence_core::enums::{DoctrineMode, EngagementPhase, GamePhase};
use deterrence_core::events::AudioEvent;
use deterrence_core::state::GameStateSnapshot;
use deterrence_core::types::SimTime;

use crate::engagement::{Engagement, ScoreState};
use crate::systems;
use crate::systems::wave_spawner::WaveSchedule;
use crate::world_setup;

/// Configuration for starting a new simulation.
pub struct SimConfig {
    /// RNG seed for determinism. Same seed = same simulation.
    pub seed: u64,
    /// Initial time scale (1.0 = normal).
    pub time_scale: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            time_scale: 1.0,
        }
    }
}

/// The simulation engine. Owns the ECS world and all sim state.
pub struct SimulationEngine {
    world: World,
    time: SimTime,
    phase: GamePhase,
    doctrine: DoctrineMode,
    time_scale: f64,
    rng: ChaCha8Rng,
    next_track_number: u32,
    command_queue: VecDeque<PlayerCommand>,
    despawn_buffer: Vec<hecs::Entity>,
    audio_events: Vec<AudioEvent>,

    // --- Phase 5 additions ---
    engagements: HashMap<u32, Engagement>,
    next_engagement_id: u32,
    wave_schedule: WaveSchedule,
    score: ScoreState,
}

impl SimulationEngine {
    /// Create a new simulation engine with the given config.
    pub fn new(config: SimConfig) -> Self {
        Self {
            world: World::new(),
            time: SimTime::default(),
            phase: GamePhase::default(),
            doctrine: DoctrineMode::default(),
            time_scale: config.time_scale,
            rng: ChaCha8Rng::seed_from_u64(config.seed),
            next_track_number: 0,
            command_queue: VecDeque::new(),
            despawn_buffer: Vec::new(),
            audio_events: Vec::new(),
            engagements: HashMap::new(),
            next_engagement_id: 0,
            wave_schedule: WaveSchedule::default(),
            score: ScoreState::default(),
        }
    }

    /// Queue a player command for processing at the next tick boundary.
    pub fn queue_command(&mut self, command: PlayerCommand) {
        self.command_queue.push_back(command);
    }

    /// Queue multiple commands.
    pub fn queue_commands(&mut self, commands: impl IntoIterator<Item = PlayerCommand>) {
        self.command_queue.extend(commands);
    }

    /// Advance the simulation by one tick and return the resulting snapshot.
    pub fn tick(&mut self) -> GameStateSnapshot {
        self.process_commands();

        if self.phase == GamePhase::Active {
            self.run_systems();
            self.time.advance();
        }

        let audio_events = std::mem::take(&mut self.audio_events);
        systems::snapshot::build_snapshot(
            &self.world,
            &self.time,
            self.phase,
            self.doctrine,
            audio_events,
            &self.engagements,
            &self.score,
        )
    }

    /// Get the current game phase.
    pub fn phase(&self) -> GamePhase {
        self.phase
    }

    /// Get the current simulation time.
    pub fn time(&self) -> SimTime {
        self.time
    }

    /// Get the current time scale.
    pub fn time_scale(&self) -> f64 {
        self.time_scale
    }

    /// Get a read-only reference to the ECS world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Spawn additional undetected threats (for testing).
    #[cfg(test)]
    pub fn spawn_test_threats(&mut self, count: usize) {
        world_setup::spawn_threat_wave(&mut self.world, &mut self.rng, count);
    }

    /// Spawn additional pre-tracked threats (for tests needing immediate tracks).
    #[cfg(test)]
    pub fn spawn_tracked_threats(&mut self, count: usize) {
        for _ in 0..count {
            world_setup::spawn_tracked_threat(
                &mut self.world,
                &mut self.rng,
                &mut self.next_track_number,
                deterrence_core::enums::ThreatArchetype::SeaSkimmerMk1,
            );
        }
    }

    /// Get a read-only reference to the engagements map.
    #[cfg(test)]
    pub fn engagements(&self) -> &HashMap<u32, Engagement> {
        &self.engagements
    }

    /// Get a read-only reference to the score state.
    #[cfg(test)]
    pub fn score(&self) -> &ScoreState {
        &self.score
    }

    /// Process all queued commands.
    fn process_commands(&mut self) {
        while let Some(command) = self.command_queue.pop_front() {
            self.handle_command(command);
        }
    }

    /// Handle a single player command.
    fn handle_command(&mut self, command: PlayerCommand) {
        match command {
            PlayerCommand::StartMission => {
                if matches!(self.phase, GamePhase::MainMenu | GamePhase::MissionBriefing) {
                    world_setup::setup_mission(&mut self.world);
                    self.wave_schedule = WaveSchedule::default_mission();
                    self.score.threats_total = self.wave_schedule.total_threats();
                    self.engagements.clear();
                    self.next_engagement_id = 0;
                    self.phase = GamePhase::Active;
                    self.time = SimTime::default();
                }
            }
            PlayerCommand::Pause => {
                if self.phase == GamePhase::Active {
                    self.phase = GamePhase::Paused;
                }
            }
            PlayerCommand::Resume => {
                if self.phase == GamePhase::Paused {
                    self.phase = GamePhase::Active;
                }
            }
            PlayerCommand::SetTimeScale { scale } => {
                self.time_scale = scale.clamp(0.0, 4.0);
            }
            PlayerCommand::HookTrack { track_number } => {
                for (_entity, track_info) in self.world.query_mut::<&mut TrackInfo>() {
                    track_info.hooked = track_info.track_number == track_number;
                }
            }
            PlayerCommand::UnhookTrack => {
                for (_entity, track_info) in self.world.query_mut::<&mut TrackInfo>() {
                    track_info.hooked = false;
                }
            }
            PlayerCommand::ClassifyTrack {
                track_number,
                classification,
            } => {
                for (_entity, track_info) in self.world.query_mut::<&mut TrackInfo>() {
                    if track_info.track_number == track_number {
                        track_info.classification = classification;
                    }
                }
            }
            PlayerCommand::SetRadarSector {
                center_bearing,
                width,
            } => {
                for (_entity, (_own, radar)) in
                    self.world.query_mut::<(&OwnShip, &mut RadarSystem)>()
                {
                    radar.sector_center = center_bearing.rem_euclid(std::f64::consts::TAU);
                    radar.sector_width = width.clamp(0.1, std::f64::consts::TAU);
                }
            }
            PlayerCommand::SetRadarMode { mode } => {
                for (_entity, (_own, radar)) in
                    self.world.query_mut::<(&OwnShip, &mut RadarSystem)>()
                {
                    radar.mode = mode;
                }
            }
            PlayerCommand::SetDoctrine { mode } => {
                self.doctrine = mode;
            }
            PlayerCommand::VetoEngagement { engagement_id } => {
                if let Some(eng) = self.engagements.get_mut(&engagement_id) {
                    if matches!(
                        eng.phase,
                        EngagementPhase::SolutionCalc | EngagementPhase::Ready
                    ) {
                        eng.phase = EngagementPhase::Aborted;
                        eng.phase_start_tick = self.time.tick;
                        // Release assigned VLS cell
                        if let Some(cell_idx) = eng.assigned_cell.take() {
                            systems::fire_control::release_vls_cell(
                                &mut self.world,
                                cell_idx,
                                eng.weapon_type,
                            );
                        }
                    }
                }
            }
            PlayerCommand::ConfirmEngagement { engagement_id } => {
                if let Some(eng) = self.engagements.get_mut(&engagement_id) {
                    if eng.phase == EngagementPhase::Ready {
                        eng.veto_remaining_secs = 0.0;
                    }
                }
            }
        }
    }

    /// Run all systems in order.
    fn run_systems(&mut self) {
        // 1. Wave spawning
        systems::wave_spawner::run(
            &mut self.world,
            &mut self.rng,
            &mut self.wave_schedule,
            self.time.tick,
        );
        // 2. Threat AI
        systems::threat_ai::run(&mut self.world, self.time.tick, &mut self.audio_events);
        // 3. Radar energy allocation + sweep advance
        systems::radar::energy::run(&mut self.world);
        // 4. Radar detection (sweep-based Pd checks)
        systems::radar::detection::run(&mut self.world, &mut self.rng, self.time.tick);
        // 5. Track lifecycle (promote/drop)
        let events = systems::radar::tracking::run(
            &mut self.world,
            &mut self.next_track_number,
            &mut self.despawn_buffer,
        );
        self.audio_events.extend(events);
        // 6. Fire control (engagement creation, veto clock, launch)
        systems::fire_control::run(
            &mut self.world,
            &mut self.engagements,
            &mut self.next_engagement_id,
            &mut self.next_track_number,
            &mut self.rng,
            &mut self.audio_events,
            &mut self.score,
            self.doctrine,
            self.time.tick,
        );
        // 7. Intercept (proximity check, Pk roll)
        systems::intercept::run(
            &mut self.world,
            &mut self.engagements,
            &mut self.rng,
            &mut self.audio_events,
            &mut self.score,
            &mut self.despawn_buffer,
        );
        // 8. Movement integration
        systems::movement::run(&mut self.world);
        // 9. Position history
        systems::movement::update_history(&mut self.world, self.time.tick);
        // 10. Cleanup (OOB, destroyed, completed)
        systems::cleanup::run(&mut self.world, &mut self.despawn_buffer);
    }
}
