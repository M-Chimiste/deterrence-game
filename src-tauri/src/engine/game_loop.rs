use crate::campaign::upgrades::UpgradeAxis;
use crate::ecs::components::InterceptorType;
use crate::engine::config;
use crate::engine::simulation::Simulation;
use crate::events::game_events::GameEvent;
use crate::persistence::save_load::{self, SaveData};
use crate::state::game_state::GamePhase;
use crate::systems::input_system::PlayerCommand;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

/// Shared handle for sending commands to the game loop from Tauri commands.
pub struct GameEngine {
    command_tx: Mutex<mpsc::Sender<EngineCommand>>,
}

#[derive(Debug)]
pub enum EngineCommand {
    Player(PlayerCommand),
    StartWave,
    ContinueToStrategic,
    ExpandRegion { region_id: u32 },
    PlaceBattery { region_id: u32, slot_index: u32 },
    RestockAllBatteries,
    RepairCity { city_index: u32 },
    UnlockInterceptor { interceptor_type: String },
    UpgradeInterceptor { interceptor_type: String, axis: String },
    GetCampaignState,
    SaveGame { slot_name: String, app_data_dir: PathBuf },
    LoadGame { save_data: SaveData },
    NewGame,
    ReturnToMainMenu,
}

impl GameEngine {
    pub fn send_command(&self, cmd: EngineCommand) {
        if let Ok(tx) = self.command_tx.lock() {
            tx.send(cmd).ok();
        }
    }
}

/// Start the game loop on a background thread.
/// Returns a GameEngine handle for sending commands.
pub fn start(app_handle: AppHandle) -> GameEngine {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        run_loop(rx, app_handle);
    });

    GameEngine {
        command_tx: Mutex::new(tx),
    }
}

fn run_loop(rx: mpsc::Receiver<EngineCommand>, app: AppHandle) {
    let mut sim = Simulation::new();
    sim.setup_world();

    // Start in MainMenu phase (Simulation defaults to Strategic for tests;
    // we override here so the frontend shows the menu on launch)
    sim.phase = GamePhase::MainMenu;

    let tick_duration = Duration::from_secs_f64(1.0 / config::TICK_RATE as f64);

    // Resolve saves directory for auto-save
    let saves_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("saves");

    // Emit initial snapshot (MainMenu phase — no campaign emit until NewGame)
    let snapshot = sim.build_snapshot();
    let _ = app.emit("game:state_snapshot", &snapshot);

    loop {
        let start = Instant::now();

        // Drain all pending commands
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                EngineCommand::StartWave => {
                    if sim.phase == GamePhase::Strategic {
                        sim.start_wave();
                    }
                }
                EngineCommand::ContinueToStrategic => {
                    if sim.phase == GamePhase::WaveResult {
                        // Sync ECS state back to campaign, calculate income
                        sim.sync_to_campaign();
                        let income = sim.apply_wave_income();
                        sim.phase = GamePhase::Strategic;

                        // Rebuild world from updated campaign state
                        sim.rebuild_world();

                        let snapshot = sim.build_snapshot();
                        let _ = app.emit("game:state_snapshot", &snapshot);

                        let mut campaign = sim.build_campaign_snapshot();
                        // Include income in the snapshot for frontend display
                        campaign.wave_income = Some(income);
                        let _ = app.emit("campaign:state_update", &campaign);
                    }
                }
                EngineCommand::ExpandRegion { region_id } => {
                    if sim.phase == GamePhase::Strategic
                        && sim.expand_region(region_id).is_ok() {
                            let snapshot = sim.build_snapshot();
                            let _ = app.emit("game:state_snapshot", &snapshot);
                            let campaign = sim.build_campaign_snapshot();
                            let _ = app.emit("campaign:state_update", &campaign);
                        }
                }
                EngineCommand::PlaceBattery {
                    region_id,
                    slot_index,
                } => {
                    if sim.phase == GamePhase::Strategic
                        && sim.place_battery(region_id, slot_index).is_ok() {
                            let snapshot = sim.build_snapshot();
                            let _ = app.emit("game:state_snapshot", &snapshot);
                            let campaign = sim.build_campaign_snapshot();
                            let _ = app.emit("campaign:state_update", &campaign);
                        }
                }
                EngineCommand::RestockAllBatteries => {
                    if sim.phase == GamePhase::Strategic
                        && sim.restock_all_batteries().is_ok() {
                            let snapshot = sim.build_snapshot();
                            let _ = app.emit("game:state_snapshot", &snapshot);
                            let campaign = sim.build_campaign_snapshot();
                            let _ = app.emit("campaign:state_update", &campaign);
                        }
                }
                EngineCommand::RepairCity { city_index } => {
                    if sim.phase == GamePhase::Strategic
                        && sim.repair_city(city_index).is_ok() {
                            let snapshot = sim.build_snapshot();
                            let _ = app.emit("game:state_snapshot", &snapshot);
                            let campaign = sim.build_campaign_snapshot();
                            let _ = app.emit("campaign:state_update", &campaign);
                        }
                }
                EngineCommand::UnlockInterceptor { interceptor_type } => {
                    if sim.phase == GamePhase::Strategic {
                        let itype = InterceptorType::parse(&interceptor_type);
                        if sim.unlock_interceptor(itype).is_ok() {
                            let campaign = sim.build_campaign_snapshot();
                            let _ = app.emit("campaign:state_update", &campaign);
                        }
                    }
                }
                EngineCommand::UpgradeInterceptor { interceptor_type, axis } => {
                    if sim.phase == GamePhase::Strategic {
                        let itype = InterceptorType::parse(&interceptor_type);
                        let ax = UpgradeAxis::parse(&axis);
                        if sim.upgrade_interceptor(itype, ax).is_ok() {
                            let campaign = sim.build_campaign_snapshot();
                            let _ = app.emit("campaign:state_update", &campaign);
                        }
                    }
                }
                EngineCommand::GetCampaignState => {
                    let campaign = sim.build_campaign_snapshot();
                    let _ = app.emit("campaign:state_update", &campaign);
                }
                EngineCommand::SaveGame {
                    slot_name,
                    app_data_dir,
                } => {
                    let data = sim.to_save_data(&slot_name);
                    if let Err(e) = save_load::save_to_file(&app_data_dir, &slot_name, &data) {
                        eprintln!("Failed to save game: {e}");
                    }
                }
                EngineCommand::LoadGame { save_data } => {
                    sim = Simulation::from_save_data(save_data);

                    let snapshot = sim.build_snapshot();
                    let _ = app.emit("game:state_snapshot", &snapshot);
                    let campaign = sim.build_campaign_snapshot();
                    let _ = app.emit("campaign:state_update", &campaign);
                }
                EngineCommand::NewGame => {
                    sim = Simulation::new();
                    sim.setup_world();
                    sim.phase = GamePhase::Strategic;

                    let snapshot = sim.build_snapshot();
                    let _ = app.emit("game:state_snapshot", &snapshot);
                    let campaign = sim.build_campaign_snapshot();
                    let _ = app.emit("campaign:state_update", &campaign);
                }
                EngineCommand::ReturnToMainMenu => {
                    sim = Simulation::new();
                    sim.setup_world();
                    sim.phase = GamePhase::MainMenu;

                    let snapshot = sim.build_snapshot();
                    let _ = app.emit("game:state_snapshot", &snapshot);
                }
                EngineCommand::Player(player_cmd) => {
                    sim.push_command(player_cmd);
                }
            }
        }

        // Only tick when a wave is active
        if sim.phase == GamePhase::WaveActive {
            let snapshot = sim.tick();
            let _ = app.emit("game:state_snapshot", &snapshot);

            // Emit discrete game events
            for event in sim.drain_events() {
                match &event {
                    GameEvent::Detonation(e) => {
                        let _ = app.emit("game:detonation", e);
                    }
                    GameEvent::Impact(e) => {
                        let _ = app.emit("game:impact", e);
                    }
                    GameEvent::CityDamaged(e) => {
                        let _ = app.emit("game:city_damaged", e);
                    }
                    GameEvent::WaveComplete(e) => {
                        let _ = app.emit("game:wave_complete", e);
                        let final_snapshot = sim.build_snapshot();
                        let _ = app.emit("game:state_snapshot", &final_snapshot);

                        // Auto-save after each wave
                        let autosave = sim.to_save_data("autosave");
                        if let Err(e) = save_load::save_to_file(&saves_dir, "autosave", &autosave) {
                            eprintln!("Auto-save failed: {e}");
                        }
                    }
                    GameEvent::MirvSplit(e) => {
                        let _ = app.emit("game:mirv_split", e);
                    }
                }
            }
        }

        let elapsed = start.elapsed();
        if elapsed < tick_duration {
            thread::sleep(tick_duration - elapsed);
        }
    }
}
