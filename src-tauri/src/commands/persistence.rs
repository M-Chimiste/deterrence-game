use crate::engine::game_loop::{EngineCommand, GameEngine};
use crate::persistence::save_load::{self, SaveMetadata};
use std::path::PathBuf;
use tauri::Manager;

fn saves_dir(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("saves")
}

#[tauri::command]
pub fn save_game(engine: tauri::State<'_, GameEngine>, app: tauri::AppHandle, slot_name: String) {
    let app_data_dir = saves_dir(&app);
    engine.send_command(EngineCommand::SaveGame {
        slot_name,
        app_data_dir,
    });
}

#[tauri::command]
pub fn load_game(engine: tauri::State<'_, GameEngine>, app: tauri::AppHandle, slot_name: String) {
    let dir = saves_dir(&app);
    match save_load::load_from_file(&dir, &slot_name) {
        Ok(save_data) => {
            engine.send_command(EngineCommand::LoadGame { save_data });
        }
        Err(e) => {
            eprintln!("Failed to load game: {e}");
        }
    }
}

#[tauri::command]
pub fn list_saves(app: tauri::AppHandle) -> Vec<SaveMetadata> {
    let dir = saves_dir(&app);
    save_load::list_saves(&dir)
}

#[tauri::command]
pub fn delete_save(app: tauri::AppHandle, slot_name: String) {
    let dir = saves_dir(&app);
    if let Err(e) = save_load::delete_save(&dir, &slot_name) {
        eprintln!("Failed to delete save: {e}");
    }
}
