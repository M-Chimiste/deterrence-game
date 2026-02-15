//! Tauri IPC command handlers.
//!
//! These `#[tauri::command]` functions are invoked by the frontend via `invoke()`.
//! They bridge frontend requests to the game loop thread via channels.

use tauri::{AppHandle, State};

use deterrence_core::commands::PlayerCommand;
use deterrence_core::state::GameStateSnapshot;

use crate::game_loop;
use crate::state::{AppState, GameLoopCommand};

/// Start the simulation. Spawns the game loop thread if not already running.
///
/// Frontend: `invoke("start_simulation")`
#[tauri::command]
pub fn start_simulation(app_handle: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut running = state.running.lock().map_err(|e| e.to_string())?;

    if *running {
        return Err("Simulation already running".into());
    }

    let cmd_tx = game_loop::spawn_game_loop(app_handle, state.latest_snapshot.clone());

    let mut tx_lock = state.command_tx.lock().map_err(|e| e.to_string())?;
    *tx_lock = Some(cmd_tx);
    *running = true;

    Ok(())
}

/// Send a player command to the simulation.
///
/// Frontend: `invoke("send_command", { command })`
#[tauri::command]
pub fn send_command(command: PlayerCommand, state: State<'_, AppState>) -> Result<(), String> {
    let tx_lock = state.command_tx.lock().map_err(|e| e.to_string())?;

    match tx_lock.as_ref() {
        Some(tx) => tx
            .send(GameLoopCommand::PlayerCommand(command))
            .map_err(|e| format!("Failed to send command: {}", e)),
        None => Err("Simulation not started".into()),
    }
}

/// Get the latest snapshot synchronously (for polling / initial state).
///
/// Frontend: `invoke("get_snapshot")`
#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> Result<Option<GameStateSnapshot>, String> {
    let lock = state.latest_snapshot.lock().map_err(|e| e.to_string())?;
    Ok(lock.clone())
}
