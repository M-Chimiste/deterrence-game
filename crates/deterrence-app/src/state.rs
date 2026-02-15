//! Application state shared across Tauri commands and the game loop thread.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use deterrence_core::commands::PlayerCommand;
use deterrence_core::state::GameStateSnapshot;

/// Commands sent from the IPC layer to the game loop thread.
#[derive(Debug)]
pub enum GameLoopCommand {
    /// A player command to forward to the simulation engine.
    PlayerCommand(PlayerCommand),
    /// Shut down the game loop thread gracefully.
    Shutdown,
}

/// Shared application state, stored as Tauri managed state.
///
/// Tauri requires managed state to be Send + Sync. We achieve this by:
/// - Wrapping `mpsc::Sender` in `Mutex` (Sender is Send but not Sync)
/// - Using `Mutex<Option<...>>` for state that may not exist before `start_simulation`
/// - Using `Arc<Mutex<...>>` for the latest snapshot (shared with game loop thread)
pub struct AppState {
    /// Channel sender to forward commands to the game loop thread.
    /// `None` before `start_simulation` is called.
    pub command_tx: Mutex<Option<mpsc::Sender<GameLoopCommand>>>,
    /// Latest snapshot for synchronous `get_snapshot` queries.
    /// Updated by the game loop thread after each tick.
    pub latest_snapshot: Arc<Mutex<Option<GameStateSnapshot>>>,
    /// Whether the game loop is currently running.
    pub running: Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            command_tx: Mutex::new(None),
            latest_snapshot: Arc::new(Mutex::new(None)),
            running: Mutex::new(false),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        assert!(state.command_tx.lock().unwrap().is_none());
        assert!(state.latest_snapshot.lock().unwrap().is_none());
        assert!(!*state.running.lock().unwrap());
    }
}
