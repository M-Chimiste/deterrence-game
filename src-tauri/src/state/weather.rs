use rand::Rng;
use rand_chacha::ChaChaRng;
use serde::{Deserialize, Serialize};

use crate::engine::config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    Clear,
    Overcast,
    Storm,
    Severe,
}

impl WeatherCondition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clear => "Clear",
            Self::Overcast => "Overcast",
            Self::Storm => "Storm",
            Self::Severe => "Severe",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WeatherState {
    pub condition: WeatherCondition,
    pub wind_x: f32,
    pub wind_y: f32,
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            condition: WeatherCondition::Clear,
            wind_x: 0.0,
            wind_y: 0.0,
        }
    }
}

/// Generate weather for a wave. Waves before WEATHER_FIRST_WAVE are always Clear.
/// Higher waves have increasing storm probability.
pub fn generate_weather(rng: &mut ChaChaRng, wave_number: u32) -> WeatherState {
    if wave_number < config::WEATHER_FIRST_WAVE {
        return WeatherState::default();
    }

    let roll: f32 = rng.r#gen();
    let waves_past = (wave_number - config::WEATHER_FIRST_WAVE) as f32;
    // Storm probability increases with wave number: base 20% + 2% per wave past threshold, capped at 60%
    let storm_chance = (0.20 + waves_past * 0.02).min(0.60);
    // Severe is a subset of storm chance: 5% + 1% per wave past, capped at 20%
    let severe_chance = (0.05 + waves_past * 0.01).min(0.20);
    // Overcast fills the gap
    let overcast_chance = 0.30_f32;

    let condition = if roll < severe_chance {
        WeatherCondition::Severe
    } else if roll < severe_chance + storm_chance {
        WeatherCondition::Storm
    } else if roll < severe_chance + storm_chance + overcast_chance {
        WeatherCondition::Overcast
    } else {
        WeatherCondition::Clear
    };

    let wind_speed = match condition {
        WeatherCondition::Clear => 0.0,
        WeatherCondition::Overcast => config::WIND_SPEED_OVERCAST,
        WeatherCondition::Storm => config::WIND_SPEED_STORM,
        WeatherCondition::Severe => config::WIND_SPEED_SEVERE,
    };

    // Random wind direction (positive = rightward, negative = leftward)
    let direction: f32 = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
    let wind_x = wind_speed * direction;

    WeatherState {
        condition,
        wind_x,
        wind_y: 0.0,
    }
}

/// Radar range multiplier based on weather condition.
pub fn radar_multiplier(condition: WeatherCondition) -> f32 {
    match condition {
        WeatherCondition::Clear => 1.0,
        WeatherCondition::Overcast => 0.85,
        WeatherCondition::Storm => 0.6,
        WeatherCondition::Severe => 0.4,
    }
}

/// Glow visibility based on weather condition. 0.0 means glow is invisible.
pub fn glow_visibility(condition: WeatherCondition) -> f32 {
    match condition {
        WeatherCondition::Clear => 1.0,
        WeatherCondition::Overcast => 0.3,
        WeatherCondition::Storm => 0.0,
        WeatherCondition::Severe => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn weather_clear_before_wave_16() {
        for wave in 1..config::WEATHER_FIRST_WAVE {
            let mut rng = ChaChaRng::seed_from_u64(wave as u64);
            let w = generate_weather(&mut rng, wave);
            assert_eq!(w.condition, WeatherCondition::Clear, "Wave {wave} should be Clear");
            assert_eq!(w.wind_x, 0.0);
        }
    }

    #[test]
    fn weather_deterministic() {
        let mut rng1 = ChaChaRng::seed_from_u64(999);
        let mut rng2 = ChaChaRng::seed_from_u64(999);
        let w1 = generate_weather(&mut rng1, 20);
        let w2 = generate_weather(&mut rng2, 20);
        assert_eq!(w1.condition, w2.condition);
        assert_eq!(w1.wind_x, w2.wind_x);
    }

    #[test]
    fn weather_uses_rng() {
        let mut rng1 = ChaChaRng::seed_from_u64(1);
        let mut rng2 = ChaChaRng::seed_from_u64(2);
        // Generate many weather samples — at least some should differ
        let mut any_different = false;
        for wave in 16..40 {
            let w1 = generate_weather(&mut rng1, wave);
            let w2 = generate_weather(&mut rng2, wave);
            if w1.condition != w2.condition || w1.wind_x != w2.wind_x {
                any_different = true;
                break;
            }
        }
        assert!(any_different, "Different seeds should produce different weather eventually");
    }

    #[test]
    fn radar_multiplier_decreases_with_worse_weather() {
        assert!(radar_multiplier(WeatherCondition::Clear) > radar_multiplier(WeatherCondition::Overcast));
        assert!(radar_multiplier(WeatherCondition::Overcast) > radar_multiplier(WeatherCondition::Storm));
        assert!(radar_multiplier(WeatherCondition::Storm) > radar_multiplier(WeatherCondition::Severe));
    }

    #[test]
    fn glow_invisible_in_storm() {
        assert_eq!(glow_visibility(WeatherCondition::Storm), 0.0);
        assert_eq!(glow_visibility(WeatherCondition::Severe), 0.0);
        assert!(glow_visibility(WeatherCondition::Clear) > 0.0);
    }
}
