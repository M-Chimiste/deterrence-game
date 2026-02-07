use crate::engine::config;
use crate::state::wave_state::WaveDefinition;
use crate::state::weather::{WeatherCondition, WeatherState};

/// Compose a wave definition based on wave number, territory size, and weather.
/// More owned regions = more missiles (stretched defenses).
/// Storm/Severe weather increases missile count.
pub fn compose_wave(wave_number: u32, owned_region_count: u32, weather: &WeatherState) -> WaveDefinition {
    let territory_factor = 1.0 + (owned_region_count as f32 - 1.0) * 0.15;
    let base_missiles = config::WAVE_BASE_MISSILES as f32
        + (wave_number.saturating_sub(1) * config::WAVE_MISSILES_PER_LEVEL) as f32;
    let weather_mult = match weather.condition {
        WeatherCondition::Storm => config::STORM_MISSILE_MULT,
        WeatherCondition::Severe => config::SEVERE_MISSILE_MULT,
        _ => 1.0,
    };
    let missile_count = (base_missiles * territory_factor * weather_mult).ceil() as u32;

    let spawn_interval = config::WAVE_BASE_SPAWN_INTERVAL
        .saturating_sub(wave_number * 5)
        .max(30);

    let flight_time_min = (config::MISSILE_FLIGHT_TIME_MIN - wave_number as f32 * 0.3).max(3.0);
    let flight_time_max = (config::MISSILE_FLIGHT_TIME_MAX - wave_number as f32 * 0.5).max(5.0);

    // MIRVs appear starting at wave MIRV_FIRST_WAVE
    let (mirv_count, mirv_child_count) = if wave_number >= config::MIRV_FIRST_WAVE {
        let waves_past = wave_number - config::MIRV_FIRST_WAVE + 1;
        let count = waves_past.min(missile_count / 3).max(1);
        let children = if wave_number >= 35 { 5 } else { 3 };
        (count, children)
    } else {
        (0, 0)
    };

    WaveDefinition {
        missile_count,
        spawn_interval_ticks: spawn_interval,
        flight_time_min,
        flight_time_max,
        mirv_count,
        mirv_child_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_weather() -> WeatherState {
        WeatherState::default()
    }

    #[test]
    fn wave1_single_region_matches_original() {
        let def = compose_wave(1, 1, &clear_weather());
        let original = WaveDefinition::for_wave(1);
        assert_eq!(def.missile_count, original.missile_count);
        assert_eq!(def.spawn_interval_ticks, original.spawn_interval_ticks);
    }

    #[test]
    fn more_territory_means_more_missiles() {
        let def_1 = compose_wave(3, 1, &clear_weather());
        let def_3 = compose_wave(3, 3, &clear_weather());
        assert!(
            def_3.missile_count > def_1.missile_count,
            "3 regions ({}) should have more missiles than 1 region ({})",
            def_3.missile_count,
            def_1.missile_count
        );
    }

    #[test]
    fn wave_difficulty_increases_with_wave_number() {
        let def_1 = compose_wave(1, 1, &clear_weather());
        let def_5 = compose_wave(5, 1, &clear_weather());
        assert!(def_5.missile_count > def_1.missile_count);
        assert!(def_5.flight_time_max < def_1.flight_time_max);
    }

    #[test]
    fn no_mirv_before_wave_26() {
        let def = compose_wave(25, 1, &clear_weather());
        assert_eq!(def.mirv_count, 0, "No MIRVs before wave 26");
    }

    #[test]
    fn mirv_at_wave_26() {
        let def = compose_wave(26, 1, &clear_weather());
        assert!(def.mirv_count > 0, "MIRVs should appear at wave 26");
        assert_eq!(def.mirv_child_count, 3);
    }

    #[test]
    fn mirv_children_increase_at_wave_35() {
        let def = compose_wave(35, 1, &clear_weather());
        assert_eq!(def.mirv_child_count, 5, "Wave 35+ should have 5 MIRV children");
    }

    #[test]
    fn storm_increases_missile_count() {
        let storm = WeatherState {
            condition: WeatherCondition::Storm,
            wind_x: 15.0,
            wind_y: 0.0,
        };
        let clear_def = compose_wave(5, 1, &clear_weather());
        let storm_def = compose_wave(5, 1, &storm);
        assert!(
            storm_def.missile_count > clear_def.missile_count,
            "Storm ({}) should have more missiles than Clear ({})",
            storm_def.missile_count,
            clear_def.missile_count
        );
    }

    #[test]
    fn severe_increases_missile_count() {
        let severe = WeatherState {
            condition: WeatherCondition::Severe,
            wind_x: 30.0,
            wind_y: 0.0,
        };
        let clear_def = compose_wave(5, 1, &clear_weather());
        let severe_def = compose_wave(5, 1, &severe);
        assert!(
            severe_def.missile_count > clear_def.missile_count,
            "Severe ({}) should have more missiles than Clear ({})",
            severe_def.missile_count,
            clear_def.missile_count
        );
    }
}
