use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    MainMenu,
    Strategic,
    WaveActive,
    WaveResult,
    RegionLost,
    CampaignOver,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub phase: GamePhase,
    pub tick: u64,
    pub wave_number: u32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            phase: GamePhase::MainMenu,
            tick: 0,
            wave_number: 0,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
