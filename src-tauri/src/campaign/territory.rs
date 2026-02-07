use serde::{Deserialize, Serialize};

use crate::engine::config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainType {
    Plains,
    Mountains,
    Coastal,
    Urban,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegionId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityDef {
    pub x: f32,
    pub y: f32,
    pub population: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatterySlot {
    pub x: f32,
    pub y: f32,
    pub occupied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: RegionId,
    pub name: String,
    pub terrain: TerrainType,
    pub cities: Vec<CityDef>,
    pub battery_slots: Vec<BatterySlot>,
    pub adjacent: Vec<RegionId>,
    pub resource_multiplier: f32,
    pub expansion_cost: u32,
    /// Position on the strategic map (for rendering)
    pub map_x: f32,
    pub map_y: f32,
}

/// Define all regions. Positions are within 1280x720 and non-overlapping.
/// Region 0 (Homeland) matches the original hardcoded layout.
pub fn define_regions() -> Vec<Region> {
    vec![
        Region {
            id: RegionId(0),
            name: "Homeland".into(),
            terrain: TerrainType::Plains,
            cities: vec![
                CityDef {
                    x: 320.0,
                    y: config::GROUND_Y,
                    population: 500,
                },
                CityDef {
                    x: 640.0,
                    y: config::GROUND_Y,
                    population: 500,
                },
                CityDef {
                    x: 960.0,
                    y: config::GROUND_Y,
                    population: 500,
                },
            ],
            battery_slots: vec![
                BatterySlot {
                    x: 160.0,
                    y: config::GROUND_Y,
                    occupied: true,
                },
                BatterySlot {
                    x: 1120.0,
                    y: config::GROUND_Y,
                    occupied: true,
                },
            ],
            adjacent: vec![RegionId(1), RegionId(2)],
            resource_multiplier: 1.0,
            expansion_cost: 0,
            map_x: 640.0,
            map_y: 360.0,
        },
        Region {
            id: RegionId(1),
            name: "Western Highlands".into(),
            terrain: TerrainType::Mountains,
            cities: vec![CityDef {
                x: 80.0,
                y: config::GROUND_Y,
                population: 300,
            }],
            battery_slots: vec![
                BatterySlot {
                    x: 40.0,
                    y: config::GROUND_Y,
                    occupied: false,
                },
                BatterySlot {
                    x: 240.0,
                    y: config::GROUND_Y,
                    occupied: false,
                },
            ],
            adjacent: vec![RegionId(0), RegionId(3)],
            resource_multiplier: 0.8,
            expansion_cost: 150,
            map_x: 320.0,
            map_y: 360.0,
        },
        Region {
            id: RegionId(2),
            name: "Eastern Seaboard".into(),
            terrain: TerrainType::Coastal,
            cities: vec![CityDef {
                x: 1200.0,
                y: config::GROUND_Y,
                population: 400,
            }],
            battery_slots: vec![BatterySlot {
                x: 1060.0,
                y: config::GROUND_Y,
                occupied: false,
            }],
            adjacent: vec![RegionId(0), RegionId(4)],
            resource_multiplier: 1.2,
            expansion_cost: 200,
            map_x: 960.0,
            map_y: 360.0,
        },
        Region {
            id: RegionId(3),
            name: "Northern Plains".into(),
            terrain: TerrainType::Plains,
            cities: vec![
                CityDef {
                    x: 420.0,
                    y: config::GROUND_Y,
                    population: 350,
                },
                CityDef {
                    x: 540.0,
                    y: config::GROUND_Y,
                    population: 350,
                },
            ],
            battery_slots: vec![BatterySlot {
                x: 480.0,
                y: config::GROUND_Y,
                occupied: false,
            }],
            adjacent: vec![RegionId(1), RegionId(4)],
            resource_multiplier: 1.0,
            expansion_cost: 250,
            map_x: 480.0,
            map_y: 200.0,
        },
        Region {
            id: RegionId(4),
            name: "Industrial Core".into(),
            terrain: TerrainType::Urban,
            cities: vec![CityDef {
                x: 800.0,
                y: config::GROUND_Y,
                population: 600,
            }],
            battery_slots: vec![
                BatterySlot {
                    x: 720.0,
                    y: config::GROUND_Y,
                    occupied: false,
                },
                BatterySlot {
                    x: 880.0,
                    y: config::GROUND_Y,
                    occupied: false,
                },
            ],
            adjacent: vec![RegionId(2), RegionId(3)],
            resource_multiplier: 1.5,
            expansion_cost: 300,
            map_x: 800.0,
            map_y: 200.0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn define_regions_has_five_regions() {
        let regions = define_regions();
        assert_eq!(regions.len(), 5);
    }

    #[test]
    fn homeland_has_three_cities_two_batteries() {
        let regions = define_regions();
        let homeland = &regions[0];
        assert_eq!(homeland.id, RegionId(0));
        assert_eq!(homeland.cities.len(), 3);
        assert_eq!(homeland.battery_slots.len(), 2);
        assert!(homeland.battery_slots[0].occupied);
        assert!(homeland.battery_slots[1].occupied);
    }

    #[test]
    fn region_adjacency_is_symmetric() {
        let regions = define_regions();
        for region in &regions {
            for adj_id in &region.adjacent {
                let adj = regions.iter().find(|r| r.id == *adj_id).unwrap();
                assert!(
                    adj.adjacent.contains(&region.id),
                    "Region {} lists {} as adjacent, but not vice versa",
                    region.id.0,
                    adj_id.0
                );
            }
        }
    }

    #[test]
    fn no_city_position_overlaps_across_all_regions() {
        let regions = define_regions();
        let mut positions: Vec<(f32, f32)> = Vec::new();
        for region in &regions {
            for city in &region.cities {
                for &(px, py) in &positions {
                    let dist = ((city.x - px).powi(2) + (city.y - py).powi(2)).sqrt();
                    assert!(
                        dist > 30.0,
                        "Cities overlap at ({}, {}) and ({}, {})",
                        city.x,
                        city.y,
                        px,
                        py
                    );
                }
                positions.push((city.x, city.y));
            }
        }
    }
}
