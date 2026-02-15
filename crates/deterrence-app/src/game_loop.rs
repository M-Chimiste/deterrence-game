//! Game loop thread — runs the simulation engine at 30Hz and emits snapshots.
//!
//! The engine is created inside this thread because it's cleaner for ownership.
//! Commands arrive via `mpsc` channel. Snapshots are emitted via Tauri `AppHandle`
//! events and stored in shared state for synchronous polling.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use deterrence_core::constants::TICK_RATE;
use deterrence_core::state::GameStateSnapshot;
use deterrence_sim::engine::{SimConfig, SimulationEngine};

use crate::state::GameLoopCommand;

/// Nominal duration of one tick at 1x speed.
const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TICK_RATE as u64);

/// Spawns the game loop in a new thread.
///
/// Returns the command sender for the IPC layer to use.
pub fn spawn_game_loop(
    app_handle: AppHandle,
    latest_snapshot: Arc<Mutex<Option<GameStateSnapshot>>>,
) -> mpsc::Sender<GameLoopCommand> {
    let (cmd_tx, cmd_rx) = mpsc::channel::<GameLoopCommand>();

    std::thread::Builder::new()
        .name("deterrence-game-loop".into())
        .spawn(move || {
            run_game_loop(app_handle, cmd_rx, &latest_snapshot);
        })
        .expect("Failed to spawn game loop thread");

    cmd_tx
}

/// The game loop. Runs until Shutdown command or channel disconnect.
fn run_game_loop(
    app_handle: AppHandle,
    cmd_rx: mpsc::Receiver<GameLoopCommand>,
    latest_snapshot: &Mutex<Option<GameStateSnapshot>>,
) {
    let mut engine = SimulationEngine::new(SimConfig::default());
    let mut next_tick_time = Instant::now();

    loop {
        // 1. Drain all pending commands
        loop {
            match cmd_rx.try_recv() {
                Ok(GameLoopCommand::PlayerCommand(cmd)) => {
                    engine.queue_command(cmd);
                }
                Ok(GameLoopCommand::Shutdown) => return,
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => return,
            }
        }

        // 2. Advance one tick (engine handles pause semantics internally)
        let snapshot = engine.tick();

        // 3. Emit snapshot to frontend via Tauri event
        let _ = app_handle.emit("game:state_snapshot", &snapshot);

        // 4. Store latest snapshot for synchronous polling
        if let Ok(mut lock) = latest_snapshot.lock() {
            *lock = Some(snapshot);
        }

        // 5. Sleep until next tick, adjusting for time_scale
        let time_scale = engine.time_scale();
        let effective_tick_duration = if time_scale > 0.001 {
            TICK_DURATION.div_f64(time_scale)
        } else {
            TICK_DURATION
        };

        next_tick_time += effective_tick_duration;
        let now = Instant::now();
        if next_tick_time > now {
            std::thread::sleep(next_tick_time - now);
        } else if now - next_tick_time > effective_tick_duration * 2 {
            // Too far behind — reset to avoid catch-up spiral
            next_tick_time = now;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deterrence_core::commands::PlayerCommand;
    use deterrence_core::enums::GamePhase;
    use std::time::Duration;

    #[test]
    fn test_command_channel_round_trip() {
        let (tx, rx) = mpsc::channel::<GameLoopCommand>();

        tx.send(GameLoopCommand::PlayerCommand(PlayerCommand::StartMission))
            .unwrap();
        tx.send(GameLoopCommand::PlayerCommand(PlayerCommand::Pause))
            .unwrap();
        tx.send(GameLoopCommand::Shutdown).unwrap();

        let mut commands = Vec::new();
        while let Ok(cmd) = rx.try_recv() {
            commands.push(cmd);
        }

        assert_eq!(commands.len(), 3);
        assert!(matches!(
            commands[0],
            GameLoopCommand::PlayerCommand(PlayerCommand::StartMission)
        ));
        assert!(matches!(
            commands[1],
            GameLoopCommand::PlayerCommand(PlayerCommand::Pause)
        ));
        assert!(matches!(commands[2], GameLoopCommand::Shutdown));
    }

    #[test]
    fn test_snapshot_serialization_under_3ms() {
        let mut engine = SimulationEngine::new(SimConfig::default());
        engine.queue_command(PlayerCommand::StartMission);

        // Run enough ticks to populate entities
        for _ in 0..50 {
            engine.tick();
        }

        let snapshot = engine.tick();
        let start = Instant::now();
        let json = serde_json::to_string(&snapshot).unwrap();
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(3),
            "Snapshot serialization took {:?}, should be <3ms",
            elapsed
        );
        assert!(!json.is_empty());
    }

    #[test]
    fn test_pause_resume_via_commands() {
        let mut engine = SimulationEngine::new(SimConfig::default());

        // Start mission
        engine.queue_command(PlayerCommand::StartMission);
        let snap = engine.tick();
        assert_eq!(snap.phase, GamePhase::Active);

        // Pause
        engine.queue_command(PlayerCommand::Pause);
        let snap = engine.tick();
        assert_eq!(snap.phase, GamePhase::Paused);
        let paused_tick = snap.time.tick;

        // Tick while paused — time should not advance
        let snap = engine.tick();
        assert_eq!(snap.time.tick, paused_tick);

        // Resume
        engine.queue_command(PlayerCommand::Resume);
        let snap = engine.tick();
        assert_eq!(snap.phase, GamePhase::Active);
        assert!(snap.time.tick > paused_tick);
    }

    #[test]
    fn test_tick_duration_constant() {
        // 30Hz = 33.333ms per tick
        let expected_nanos = 1_000_000_000u64 / 30;
        assert_eq!(TICK_DURATION.as_nanos(), expected_nanos as u128);
    }
}
