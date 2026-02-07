use serde::{Deserialize, Serialize};

use crate::campaign::economy::CostTable;
use crate::campaign::territory::{BatterySlot, CityDef, Region, RegionId};
use crate::campaign::upgrades::TechTree;
use crate::engine::config;

/// Persistent campaign state that survives across waves.
/// City health and battery ammo are stored here between waves,
/// then projected into the ECS world when a wave starts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignState {
    pub resources: u32,
    pub owned_regions: Vec<RegionId>,
    pub regions: Vec<Region>,
    pub cost_table: CostTable,
    pub total_waves_survived: u32,
    /// Per-city health tracking: (region_id, city_index, current_health)
    pub city_healths: Vec<(RegionId, usize, f32)>,
    /// Per-battery ammo tracking: (region_id, slot_index, current_ammo)
    pub battery_ammo: Vec<(RegionId, usize, u32)>,
    /// Tech tree: unlocked interceptor types and upgrades
    pub tech_tree: TechTree,
}

impl Default for CampaignState {
    fn default() -> Self {
        let regions = crate::campaign::territory::define_regions();

        // Initialize health for homeland cities
        let mut city_healths = Vec::new();
        let homeland = &regions[0];
        for (i, _city) in homeland.cities.iter().enumerate() {
            city_healths.push((RegionId(0), i, config::CITY_MAX_HEALTH));
        }

        // Initialize ammo for homeland batteries (occupied slots)
        let mut battery_ammo = Vec::new();
        for (i, slot) in homeland.battery_slots.iter().enumerate() {
            if slot.occupied {
                battery_ammo.push((RegionId(0), i, config::BATTERY_MAX_AMMO));
            }
        }

        Self {
            resources: 100,
            owned_regions: vec![RegionId(0)],
            regions,
            cost_table: CostTable::default(),
            total_waves_survived: 0,
            city_healths,
            battery_ammo,
            tech_tree: TechTree::default(),
        }
    }
}

impl CampaignState {
    /// Get all city definitions and their health across owned regions.
    pub fn active_cities(&self) -> Vec<(&CityDef, f32)> {
        let mut result = Vec::new();
        for rid in &self.owned_regions {
            let region = self.get_region(*rid).unwrap();
            for (i, city) in region.cities.iter().enumerate() {
                let health = self
                    .city_healths
                    .iter()
                    .find(|(r, ci, _)| *r == *rid && *ci == i)
                    .map(|(_, _, h)| *h)
                    .unwrap_or(config::CITY_MAX_HEALTH);
                result.push((city, health));
            }
        }
        result
    }

    /// Get all occupied battery slots and their ammo across owned regions.
    pub fn active_batteries(&self) -> Vec<(&BatterySlot, u32)> {
        let mut result = Vec::new();
        for rid in &self.owned_regions {
            let region = self.get_region(*rid).unwrap();
            for (i, slot) in region.battery_slots.iter().enumerate() {
                if slot.occupied {
                    let ammo = self
                        .battery_ammo
                        .iter()
                        .find(|(r, si, _)| *r == *rid && *si == i)
                        .map(|(_, _, a)| *a)
                        .unwrap_or(0);
                    result.push((slot, ammo));
                }
            }
        }
        result
    }

    /// Get available (unoccupied) battery slots across owned regions.
    pub fn available_battery_slots(&self) -> Vec<(RegionId, usize, &BatterySlot)> {
        let mut result = Vec::new();
        for rid in &self.owned_regions {
            let region = self.get_region(*rid).unwrap();
            for (i, slot) in region.battery_slots.iter().enumerate() {
                if !slot.occupied {
                    result.push((*rid, i, slot));
                }
            }
        }
        result
    }

    /// Get regions adjacent to owned territory that can be expanded into.
    pub fn expandable_regions(&self) -> Vec<&Region> {
        let mut result = Vec::new();
        for rid in &self.owned_regions {
            let region = self.get_region(*rid).unwrap();
            for adj_id in &region.adjacent {
                if !self.owned_regions.contains(adj_id)
                    && let Some(adj) = self.get_region(*adj_id)
                    && !result.iter().any(|r: &&Region| r.id == *adj_id)
                {
                    result.push(adj);
                }
            }
        }
        result
    }

    /// Look up a region by ID.
    pub fn get_region(&self, id: RegionId) -> Option<&Region> {
        self.regions.iter().find(|r| r.id == id)
    }

    /// Look up a region mutably by ID.
    pub fn get_region_mut(&mut self, id: RegionId) -> Option<&mut Region> {
        self.regions.iter_mut().find(|r| r.id == id)
    }
}

/// Serializable campaign snapshot for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignSnapshot {
    pub resources: u32,
    pub wave_number: u32,
    pub owned_region_ids: Vec<u32>,
    pub regions: Vec<RegionSnapshot>,
    pub available_actions: Vec<AvailableAction>,
    pub tech_tree: TechTreeSnapshot,
    /// Income from the last completed wave (only set on transition to Strategic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wave_income: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionSnapshot {
    pub id: u32,
    pub name: String,
    pub terrain: String,
    pub owned: bool,
    pub expandable: bool,
    pub cities: Vec<CitySnapshotCampaign>,
    pub battery_slots: Vec<BatterySlotSnapshot>,
    pub map_x: f32,
    pub map_y: f32,
    pub expansion_cost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitySnapshotCampaign {
    pub x: f32,
    pub y: f32,
    pub population: u32,
    pub health: f32,
    pub max_health: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatterySlotSnapshot {
    pub x: f32,
    pub y: f32,
    pub occupied: bool,
    pub ammo: Option<u32>,
    pub max_ammo: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AvailableAction {
    ExpandRegion { region_id: u32, cost: u32 },
    PlaceBattery { region_id: u32, slot_index: u32, cost: u32 },
    RestockAllBatteries { count: u32, cost: u32 },
    RepairCity { region_id: u32, city_index: u32, cost: u32, health_to_restore: f32 },
    UnlockInterceptor { interceptor_type: String, cost: u32, min_wave: u32 },
    UpgradeInterceptor { interceptor_type: String, axis: String, cost: u32, current_level: u32 },
    StartWave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechTreeSnapshot {
    pub unlocked_types: Vec<String>,
    pub upgrades: Vec<TypeUpgradeSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeUpgradeSnapshot {
    pub interceptor_type: String,
    pub thrust_level: u32,
    pub yield_level: u32,
    pub guidance_level: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_campaign_owns_homeland() {
        let cs = CampaignState::default();
        assert_eq!(cs.owned_regions.len(), 1);
        assert_eq!(cs.owned_regions[0], RegionId(0));
    }

    #[test]
    fn default_campaign_has_correct_city_count() {
        let cs = CampaignState::default();
        assert_eq!(cs.active_cities().len(), 3);
    }

    #[test]
    fn default_campaign_has_correct_battery_count() {
        let cs = CampaignState::default();
        assert_eq!(cs.active_batteries().len(), 2);
    }

    #[test]
    fn expandable_regions_are_adjacent_and_unowned() {
        let cs = CampaignState::default();
        let expandable = cs.expandable_regions();
        // Homeland is adjacent to regions 1 and 2
        assert_eq!(expandable.len(), 2);
        let ids: Vec<u32> = expandable.iter().map(|r| r.id.0).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn available_battery_slots_empty_for_homeland() {
        let cs = CampaignState::default();
        // Homeland slots are pre-occupied
        let available = cs.available_battery_slots();
        assert_eq!(available.len(), 0);
    }

    #[test]
    fn city_health_initialized() {
        let cs = CampaignState::default();
        assert_eq!(cs.city_healths.len(), 3);
        for (_, _, health) in &cs.city_healths {
            assert_eq!(*health, config::CITY_MAX_HEALTH);
        }
    }

    #[test]
    fn battery_ammo_initialized() {
        let cs = CampaignState::default();
        assert_eq!(cs.battery_ammo.len(), 2);
        for (_, _, ammo) in &cs.battery_ammo {
            assert_eq!(*ammo, config::BATTERY_MAX_AMMO);
        }
    }
}
