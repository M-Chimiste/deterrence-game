// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use deterrence_app::ipc;
use deterrence_app::state::AppState;

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            ipc::start_simulation,
            ipc::send_command,
            ipc::get_snapshot,
            ipc::get_terrain_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DETERRENCE");
}
