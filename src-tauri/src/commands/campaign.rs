use crate::engine::game_loop::{EngineCommand, GameEngine};

#[tauri::command]
pub fn start_wave(engine: tauri::State<'_, GameEngine>) {
    engine.send_command(EngineCommand::StartWave);
}

#[tauri::command]
pub fn continue_to_strategic(engine: tauri::State<'_, GameEngine>) {
    engine.send_command(EngineCommand::ContinueToStrategic);
}

#[tauri::command]
pub fn expand_region(engine: tauri::State<'_, GameEngine>, region_id: u32) {
    engine.send_command(EngineCommand::ExpandRegion { region_id });
}

#[tauri::command]
pub fn place_battery(engine: tauri::State<'_, GameEngine>, region_id: u32, slot_index: u32) {
    engine.send_command(EngineCommand::PlaceBattery {
        region_id,
        slot_index,
    });
}

#[tauri::command]
pub fn restock_all_batteries(engine: tauri::State<'_, GameEngine>) {
    engine.send_command(EngineCommand::RestockAllBatteries);
}

#[tauri::command]
pub fn repair_city(engine: tauri::State<'_, GameEngine>, city_index: u32) {
    engine.send_command(EngineCommand::RepairCity { city_index });
}

#[tauri::command]
pub fn unlock_interceptor(engine: tauri::State<'_, GameEngine>, interceptor_type: String) {
    engine.send_command(EngineCommand::UnlockInterceptor { interceptor_type });
}

#[tauri::command]
pub fn upgrade_interceptor(engine: tauri::State<'_, GameEngine>, interceptor_type: String, axis: String) {
    engine.send_command(EngineCommand::UpgradeInterceptor { interceptor_type, axis });
}

#[tauri::command]
pub fn get_campaign_state(engine: tauri::State<'_, GameEngine>) {
    engine.send_command(EngineCommand::GetCampaignState);
}

#[tauri::command]
pub fn new_game(engine: tauri::State<'_, GameEngine>) {
    engine.send_command(EngineCommand::NewGame);
}
