use crate::engine::config;

#[derive(Debug, Clone)]
pub struct WaveDefinition {
    pub missile_count: u32,
    pub spawn_interval_ticks: u32,
    pub flight_time_min: f32,
    pub flight_time_max: f32,
    pub mirv_count: u32,
    pub mirv_child_count: u32,
}

impl WaveDefinition {
    pub fn for_wave(wave_number: u32) -> Self {
        Self {
            missile_count: config::WAVE_BASE_MISSILES
                + wave_number.saturating_sub(1) * config::WAVE_MISSILES_PER_LEVEL,
            spawn_interval_ticks: config::WAVE_BASE_SPAWN_INTERVAL
                .saturating_sub(wave_number * 5)
                .max(30),
            flight_time_min: (config::MISSILE_FLIGHT_TIME_MIN - wave_number as f32 * 0.3).max(3.0),
            flight_time_max: (config::MISSILE_FLIGHT_TIME_MAX - wave_number as f32 * 0.5).max(5.0),
            mirv_count: 0,
            mirv_child_count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WaveState {
    pub definition: WaveDefinition,
    pub missiles_spawned: u32,
    pub missiles_destroyed: u32,
    pub missiles_impacted: u32,
    pub interceptors_launched: u32,
    pub mirv_spawned: u32,
    pub spawn_timer: u32,
}

impl WaveState {
    pub fn new(definition: WaveDefinition) -> Self {
        Self {
            definition,
            missiles_spawned: 0,
            missiles_destroyed: 0,
            missiles_impacted: 0,
            interceptors_launched: 0,
            mirv_spawned: 0,
            spawn_timer: 0,
        }
    }

    pub fn all_spawned(&self) -> bool {
        self.missiles_spawned >= self.definition.missile_count
    }
}
