use serde::{Deserialize, Serialize};

/// Cost table for strategic actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTable {
    pub place_battery: u32,
    pub restock_battery: u32,
    pub repair_cost_per_hp: u32,
}

impl Default for CostTable {
    fn default() -> Self {
        Self {
            place_battery: 100,
            restock_battery: 15,
            repair_cost_per_hp: 2,
        }
    }
}

/// Calculate resources earned at end of a wave.
/// Each surviving city contributes: (population * health_ratio * region_multiplier) / 10
pub fn calculate_wave_income(
    city_healths: &[(u32, f32, f32)], // (population, health_ratio 0..1, region_multiplier)
) -> u32 {
    let mut total = 0.0_f32;
    for &(population, health_ratio, multiplier) in city_healths {
        total += population as f32 * health_ratio * multiplier;
    }
    (total / 10.0).ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn income_from_full_health_cities() {
        // 3 cities, 500 pop each, full health, 1.0 multiplier
        let cities = vec![
            (500, 1.0_f32, 1.0_f32),
            (500, 1.0, 1.0),
            (500, 1.0, 1.0),
        ];
        let income = calculate_wave_income(&cities);
        // (500 + 500 + 500) / 10 = 150
        assert_eq!(income, 150);
    }

    #[test]
    fn income_scales_with_damage() {
        let cities = vec![(500, 0.5_f32, 1.0_f32)];
        let income = calculate_wave_income(&cities);
        // 500 * 0.5 / 10 = 25
        assert_eq!(income, 25);
    }

    #[test]
    fn income_zero_if_all_dead() {
        let cities = vec![(500, 0.0_f32, 1.0_f32), (500, 0.0, 1.0)];
        let income = calculate_wave_income(&cities);
        assert_eq!(income, 0);
    }

    #[test]
    fn income_scales_with_region_multiplier() {
        let cities = vec![(600, 1.0_f32, 1.5_f32)];
        let income = calculate_wave_income(&cities);
        // 600 * 1.0 * 1.5 / 10 = 90
        assert_eq!(income, 90);
    }
}
