pub mod campaign;
pub mod commands;
pub mod ecs;
pub mod engine;
pub mod events;
pub mod persistence;
pub mod state;
pub mod systems;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::tactical::launch_interceptor,
            commands::tactical::predict_arc,
            commands::campaign::start_wave,
            commands::campaign::continue_to_strategic,
            commands::campaign::expand_region,
            commands::campaign::place_battery,
            commands::campaign::restock_all_batteries,
            commands::campaign::repair_city,
            commands::campaign::unlock_interceptor,
            commands::campaign::upgrade_interceptor,
            commands::campaign::get_campaign_state,
            commands::campaign::new_game,
            commands::campaign::return_to_main_menu,
            commands::persistence::save_game,
            commands::persistence::load_game,
            commands::persistence::list_saves,
            commands::persistence::delete_save,
        ])
        .setup(|app| {
            // Start game loop on background thread
            let game_engine = engine::game_loop::start(app.handle().clone());
            app.manage(game_engine);

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
