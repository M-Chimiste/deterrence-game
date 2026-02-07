use crate::ecs::components::InterceptorType;
use crate::engine::config;
use crate::engine::game_loop::{EngineCommand, GameEngine};
use crate::systems::arc_prediction::{self, ArcPrediction};
use crate::systems::input_system::PlayerCommand;

#[tauri::command]
pub fn launch_interceptor(
    engine: tauri::State<'_, GameEngine>,
    battery_id: u32,
    target_x: f32,
    target_y: f32,
    interceptor_type: Option<String>,
) {
    let itype = interceptor_type
        .map(|s| InterceptorType::parse(&s))
        .unwrap_or_default();
    engine.send_command(EngineCommand::Player(PlayerCommand::LaunchInterceptor {
        battery_id,
        target_x,
        target_y,
        interceptor_type: itype,
    }));
}

#[tauri::command]
pub fn predict_arc(
    battery_x: f32,
    battery_y: f32,
    target_x: f32,
    target_y: f32,
    interceptor_type: Option<String>,
    wind_x: Option<f32>,
) -> ArcPrediction {
    let itype = interceptor_type
        .map(|s| InterceptorType::parse(&s))
        .unwrap_or_default();
    let profile = config::interceptor_profile(itype);
    arc_prediction::predict_arc(battery_x, battery_y, target_x, target_y, &profile, wind_x.unwrap_or(0.0))
}
