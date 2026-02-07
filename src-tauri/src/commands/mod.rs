pub mod campaign;
pub mod persistence;
pub mod tactical;

use serde::Serialize;

#[derive(Serialize)]
pub struct PingResponse {
    pub message: String,
    pub tick: u64,
}

#[tauri::command]
pub fn ping() -> PingResponse {
    PingResponse {
        message: "pong".to_string(),
        tick: 0,
    }
}
