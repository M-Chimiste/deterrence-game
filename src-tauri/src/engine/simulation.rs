use crate::campaign::economy;
use crate::campaign::territory::RegionId;
use crate::campaign::upgrades::{self, UpgradeAxis};
use crate::campaign::wave_composer;
use crate::ecs::components::*;
use crate::ecs::entity::EntityId;
use crate::ecs::world::World;
use crate::engine::config;
use crate::events::game_events::{GameEvent, WaveCompleteEvent};
use crate::persistence::save_load::SaveData;
use crate::state::weather::{self, WeatherState};
use crate::state::campaign_state::{
    AvailableAction, BatterySlotSnapshot, CampaignSnapshot, CampaignState, CitySnapshotCampaign,
    RegionSnapshot, TechTreeSnapshot, TypeUpgradeSnapshot,
};
use crate::state::game_state::GamePhase;
use crate::state::snapshot::StateSnapshot;
use crate::state::wave_state::WaveState;
use crate::systems;
use crate::systems::input_system::PlayerCommand;
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use std::time::{SystemTime, UNIX_EPOCH};

/// Top-level simulation orchestrator.
/// Owns the ECS World and runs systems in the correct order each tick.
pub struct Simulation {
    pub world: World,
    pub tick: u64,
    pub wave_number: u32,
    pub phase: GamePhase,
    pub rng: ChaChaRng,
    pub seed: u64,
    pub weather: WeatherState,
    pub wave: Option<WaveState>,
    pub city_ids: Vec<EntityId>,
    pub battery_ids: Vec<EntityId>,
    pub input_queue: Vec<PlayerCommand>,
    pending_events: Vec<GameEvent>,
    pub campaign: CampaignState,
}

impl Simulation {
    pub fn new() -> Self {
        Self::new_with_seed(42)
    }

    pub fn new_with_seed(seed: u64) -> Self {
        Self {
            world: World::new(),
            tick: 0,
            wave_number: 0,
            phase: GamePhase::Strategic,
            rng: ChaChaRng::seed_from_u64(seed),
            seed,
            weather: WeatherState::default(),
            wave: None,
            city_ids: Vec::new(),
            battery_ids: Vec::new(),
            input_queue: Vec::new(),
            pending_events: Vec::new(),
            campaign: CampaignState::default(),
        }
    }

    pub fn new_with_campaign(campaign: CampaignState, seed: u64) -> Self {
        Self {
            world: World::new(),
            tick: 0,
            wave_number: 0,
            phase: GamePhase::Strategic,
            rng: ChaChaRng::seed_from_u64(seed),
            seed,
            weather: WeatherState::default(),
            wave: None,
            city_ids: Vec::new(),
            battery_ids: Vec::new(),
            input_queue: Vec::new(),
            pending_events: Vec::new(),
            campaign,
        }
    }

    /// Create a SaveData from current simulation state.
    pub fn to_save_data(&self, slot_name: &str) -> SaveData {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        SaveData {
            campaign: self.campaign.clone(),
            wave_number: self.wave_number,
            seed: self.seed,
            timestamp,
            slot_name: slot_name.to_string(),
        }
    }

    /// Reconstruct a Simulation from saved data.
    pub fn from_save_data(data: SaveData) -> Self {
        // Re-seed RNG offset by wave_number so future waves diverge from earlier saves
        let rng_seed = data.seed.wrapping_add(data.wave_number as u64 * 1000);
        let mut sim = Self {
            world: World::new(),
            tick: 0,
            wave_number: data.wave_number,
            phase: GamePhase::Strategic,
            rng: ChaChaRng::seed_from_u64(rng_seed),
            seed: data.seed,
            weather: WeatherState::default(),
            wave: None,
            city_ids: Vec::new(),
            battery_ids: Vec::new(),
            input_queue: Vec::new(),
            pending_events: Vec::new(),
            campaign: data.campaign,
        };
        sim.setup_world();
        sim
    }

    /// Set up the initial world with cities and batteries from campaign state.
    /// Backward compatible: default campaign = homeland = original 3 cities + 2 batteries.
    pub fn setup_world(&mut self) {
        self.spawn_from_campaign();
    }

    /// Replace the ECS world with a fresh one built from campaign state.
    pub fn rebuild_world(&mut self) {
        self.world = World::new();
        self.city_ids.clear();
        self.battery_ids.clear();
        self.spawn_from_campaign();
    }

    /// Spawn ECS entities from current campaign state.
    fn spawn_from_campaign(&mut self) {
        let owned = self.campaign.owned_regions.clone();
        for rid in &owned {
            let region = self.campaign.get_region(*rid).unwrap().clone();

            for (i, city) in region.cities.iter().enumerate() {
                let health = self
                    .campaign
                    .city_healths
                    .iter()
                    .find(|(r, ci, _)| *r == *rid && *ci == i)
                    .map(|(_, _, h)| *h)
                    .unwrap_or(config::CITY_MAX_HEALTH);

                let id = self.world.spawn();
                let idx = id.index as usize;
                self.world.transforms[idx] = Some(Transform {
                    x: city.x,
                    y: city.y,
                    rotation: 0.0,
                });
                self.world.markers[idx] = Some(EntityMarker {
                    kind: EntityKind::City,
                });
                self.world.healths[idx] = Some(Health {
                    current: health,
                    max: config::CITY_MAX_HEALTH,
                });
                self.city_ids.push(id);
            }

            for (i, slot) in region.battery_slots.iter().enumerate() {
                if !slot.occupied {
                    continue;
                }
                let ammo = self
                    .campaign
                    .battery_ammo
                    .iter()
                    .find(|(r, si, _)| *r == *rid && *si == i)
                    .map(|(_, _, a)| *a)
                    .unwrap_or(config::BATTERY_MAX_AMMO);

                let id = self.world.spawn();
                let idx = id.index as usize;
                self.world.transforms[idx] = Some(Transform {
                    x: slot.x,
                    y: slot.y,
                    rotation: 0.0,
                });
                self.world.markers[idx] = Some(EntityMarker {
                    kind: EntityKind::Battery,
                });
                self.world.battery_states[idx] = Some(BatteryState {
                    ammo,
                    max_ammo: config::BATTERY_MAX_AMMO,
                });
                self.battery_ids.push(id);
            }
        }
    }

    /// Copy city health and battery ammo from ECS back to campaign state.
    pub fn sync_to_campaign(&mut self) {
        let owned = self.campaign.owned_regions.clone();
        let mut city_idx = 0;
        for rid in &owned {
            let region = self.campaign.get_region(*rid).unwrap().clone();
            for i in 0..region.cities.len() {
                if city_idx < self.city_ids.len() {
                    let eid = self.city_ids[city_idx];
                    if self.world.is_alive(eid)
                        && let Some(h) = &self.world.healths[eid.index as usize]
                            && let Some(entry) = self
                                .campaign
                                .city_healths
                                .iter_mut()
                                .find(|(r, ci, _)| *r == *rid && *ci == i)
                            {
                                entry.2 = h.current;
                            }
                }
                city_idx += 1;
            }
        }

        let mut bat_idx = 0;
        for rid in &owned {
            let region = self.campaign.get_region(*rid).unwrap().clone();
            for (i, slot) in region.battery_slots.iter().enumerate() {
                if !slot.occupied {
                    continue;
                }
                if bat_idx < self.battery_ids.len() {
                    let eid = self.battery_ids[bat_idx];
                    if self.world.is_alive(eid)
                        && let Some(bs) = &self.world.battery_states[eid.index as usize]
                            && let Some(entry) = self
                                .campaign
                                .battery_ammo
                                .iter_mut()
                                .find(|(r, si, _)| *r == *rid && *si == i)
                            {
                                entry.2 = bs.ammo;
                            }
                }
                bat_idx += 1;
            }
        }
    }

    /// Calculate and add wave income to resources. Returns the income earned.
    pub fn apply_wave_income(&mut self) -> u32 {
        let city_data: Vec<(u32, f32, f32)> = {
            let mut data = Vec::new();
            for rid in &self.campaign.owned_regions {
                let region = self.campaign.get_region(*rid).unwrap();
                let multiplier = region.resource_multiplier;
                for (i, city) in region.cities.iter().enumerate() {
                    let health = self
                        .campaign
                        .city_healths
                        .iter()
                        .find(|(r, ci, _)| *r == *rid && *ci == i)
                        .map(|(_, _, h)| *h)
                        .unwrap_or(0.0);
                    let health_ratio = health / config::CITY_MAX_HEALTH;
                    data.push((city.population, health_ratio, multiplier));
                }
            }
            data
        };
        let income = economy::calculate_wave_income(&city_data);
        self.campaign.resources += income;
        self.campaign.total_waves_survived += 1;
        income
    }

    /// Expand into a new region.
    pub fn expand_region(&mut self, region_id: u32) -> Result<(), String> {
        let target_rid = RegionId(region_id);

        if self.campaign.owned_regions.contains(&target_rid) {
            return Err("Region already owned".into());
        }

        let expandable_ids: Vec<RegionId> = self
            .campaign
            .expandable_regions()
            .iter()
            .map(|r| r.id)
            .collect();
        if !expandable_ids.contains(&target_rid) {
            return Err("Region not adjacent to owned territory".into());
        }

        let cost = self
            .campaign
            .get_region(target_rid)
            .ok_or("Region not found")?
            .expansion_cost;
        if self.campaign.resources < cost {
            return Err(format!(
                "Insufficient resources: have {}, need {}",
                self.campaign.resources, cost
            ));
        }

        self.campaign.resources -= cost;
        self.campaign.owned_regions.push(target_rid);

        let region = self.campaign.get_region(target_rid).unwrap().clone();
        for (i, _) in region.cities.iter().enumerate() {
            self.campaign
                .city_healths
                .push((target_rid, i, config::CITY_MAX_HEALTH));
        }

        self.rebuild_world();
        Ok(())
    }

    /// Place a battery at an available slot.
    pub fn place_battery(&mut self, region_id: u32, slot_index: u32) -> Result<(), String> {
        let rid = RegionId(region_id);

        if !self.campaign.owned_regions.contains(&rid) {
            return Err("Region not owned".into());
        }

        let cost = self.campaign.cost_table.place_battery;
        if self.campaign.resources < cost {
            return Err(format!(
                "Insufficient resources: have {}, need {}",
                self.campaign.resources, cost
            ));
        }

        let region = self
            .campaign
            .get_region_mut(rid)
            .ok_or("Region not found")?;
        let slot = region
            .battery_slots
            .get_mut(slot_index as usize)
            .ok_or("Invalid slot index")?;
        if slot.occupied {
            return Err("Slot already occupied".into());
        }

        slot.occupied = true;
        self.campaign.resources -= cost;
        self.campaign
            .battery_ammo
            .push((rid, slot_index as usize, config::BATTERY_MAX_AMMO));

        self.rebuild_world();
        Ok(())
    }

    /// Restock a battery's ammo. Uses battery_ids index.
    pub fn restock_battery(&mut self, battery_index: u32) -> Result<(), String> {
        let cost = self.campaign.cost_table.restock_battery;
        if self.campaign.resources < cost {
            return Err(format!(
                "Insufficient resources: have {}, need {}",
                self.campaign.resources, cost
            ));
        }

        let bid = *self
            .battery_ids
            .get(battery_index as usize)
            .ok_or("Invalid battery index")?;
        if !self.world.is_alive(bid) {
            return Err("Battery not alive".into());
        }

        let max_ammo = {
            let bs = self.world.battery_states[bid.index as usize]
                .as_ref()
                .ok_or("No battery state")?;
            if bs.ammo >= bs.max_ammo {
                return Err("Battery already full".into());
            }
            bs.max_ammo
        };

        self.world.battery_states[bid.index as usize]
            .as_mut()
            .unwrap()
            .ammo = max_ammo;
        self.campaign.resources -= cost;
        self.sync_battery_ammo_at(battery_index as usize, max_ammo);

        Ok(())
    }

    /// Repair a city to full health. Uses city_ids index.
    pub fn repair_city(&mut self, city_index: u32) -> Result<(), String> {
        let cid = *self
            .city_ids
            .get(city_index as usize)
            .ok_or("Invalid city index")?;
        if !self.world.is_alive(cid) {
            return Err("City not alive".into());
        }

        let (damage, max_health) = {
            let h = self.world.healths[cid.index as usize]
                .as_ref()
                .ok_or("No health component")?;
            (h.max - h.current, h.max)
        };

        if damage <= 0.0 {
            return Err("City already at full health".into());
        }

        let cost = (damage * self.campaign.cost_table.repair_cost_per_hp as f32).ceil() as u32;
        if self.campaign.resources < cost {
            return Err(format!(
                "Insufficient resources: have {}, need {}",
                self.campaign.resources, cost
            ));
        }

        self.campaign.resources -= cost;
        self.world.healths[cid.index as usize]
            .as_mut()
            .unwrap()
            .current = max_health;
        self.sync_city_health_at(city_index as usize, max_health);

        Ok(())
    }

    /// Unlock a new interceptor type.
    pub fn unlock_interceptor(&mut self, itype: InterceptorType) -> Result<(), String> {
        let cost = self.campaign.tech_tree.unlock(itype, self.wave_number, self.campaign.resources)?;
        self.campaign.resources -= cost;
        Ok(())
    }

    /// Upgrade an interceptor type on a given axis.
    pub fn upgrade_interceptor(&mut self, itype: InterceptorType, axis: UpgradeAxis) -> Result<(), String> {
        let cost = self.campaign.tech_tree.apply_upgrade(itype, axis, self.campaign.resources)?;
        self.campaign.resources -= cost;
        Ok(())
    }

    /// Build a campaign snapshot for the frontend.
    pub fn build_campaign_snapshot(&self) -> CampaignSnapshot {
        let expandable_ids: Vec<u32> = self
            .campaign
            .expandable_regions()
            .iter()
            .map(|r| r.id.0)
            .collect();

        let regions: Vec<RegionSnapshot> = self
            .campaign
            .regions
            .iter()
            .map(|region| {
                let owned = self.campaign.owned_regions.contains(&region.id);
                let expandable = expandable_ids.contains(&region.id.0);

                let cities: Vec<CitySnapshotCampaign> = region
                    .cities
                    .iter()
                    .enumerate()
                    .map(|(i, city)| {
                        let health = if owned {
                            self.campaign
                                .city_healths
                                .iter()
                                .find(|(r, ci, _)| *r == region.id && *ci == i)
                                .map(|(_, _, h)| *h)
                                .unwrap_or(config::CITY_MAX_HEALTH)
                        } else {
                            config::CITY_MAX_HEALTH
                        };
                        CitySnapshotCampaign {
                            x: city.x,
                            y: city.y,
                            population: city.population,
                            health,
                            max_health: config::CITY_MAX_HEALTH,
                        }
                    })
                    .collect();

                let battery_slots: Vec<BatterySlotSnapshot> = region
                    .battery_slots
                    .iter()
                    .enumerate()
                    .map(|(i, slot)| {
                        let (ammo, max_ammo) = if slot.occupied {
                            let a = self
                                .campaign
                                .battery_ammo
                                .iter()
                                .find(|(r, si, _)| *r == region.id && *si == i)
                                .map(|(_, _, a)| *a)
                                .unwrap_or(0);
                            (Some(a), Some(config::BATTERY_MAX_AMMO))
                        } else {
                            (None, None)
                        };
                        BatterySlotSnapshot {
                            x: slot.x,
                            y: slot.y,
                            occupied: slot.occupied,
                            ammo,
                            max_ammo,
                        }
                    })
                    .collect();

                RegionSnapshot {
                    id: region.id.0,
                    name: region.name.clone(),
                    terrain: format!("{:?}", region.terrain),
                    owned,
                    expandable,
                    cities,
                    battery_slots,
                    map_x: region.map_x,
                    map_y: region.map_y,
                    expansion_cost: region.expansion_cost,
                }
            })
            .collect();

        let mut available_actions = Vec::new();

        for r in self.campaign.expandable_regions() {
            if self.campaign.resources >= r.expansion_cost {
                available_actions.push(AvailableAction::ExpandRegion {
                    region_id: r.id.0,
                    cost: r.expansion_cost,
                });
            }
        }

        for (rid, si, _) in self.campaign.available_battery_slots() {
            if self.campaign.resources >= self.campaign.cost_table.place_battery {
                available_actions.push(AvailableAction::PlaceBattery {
                    region_id: rid.0,
                    slot_index: si as u32,
                    cost: self.campaign.cost_table.place_battery,
                });
            }
        }

        for (i, &bid) in self.battery_ids.iter().enumerate() {
            if self.world.is_alive(bid)
                && let Some(bs) = &self.world.battery_states[bid.index as usize]
                    && bs.ammo < bs.max_ammo
                        && self.campaign.resources >= self.campaign.cost_table.restock_battery
                    {
                        let (rid, si) = self.battery_index_to_region(i);
                        available_actions.push(AvailableAction::RestockBattery {
                            region_id: rid.0,
                            slot_index: si as u32,
                            cost: self.campaign.cost_table.restock_battery,
                        });
                    }
        }

        for (i, &cid) in self.city_ids.iter().enumerate() {
            if self.world.is_alive(cid)
                && let Some(h) = &self.world.healths[cid.index as usize] {
                    let damage = h.max - h.current;
                    if damage > 0.0 {
                        let cost = (damage * self.campaign.cost_table.repair_cost_per_hp as f32)
                            .ceil() as u32;
                        if self.campaign.resources >= cost {
                            let (rid, ci) = self.city_index_to_region(i);
                            available_actions.push(AvailableAction::RepairCity {
                                region_id: rid.0,
                                city_index: ci as u32,
                                cost,
                                health_to_restore: damage,
                            });
                        }
                    }
                }
        }

        // Tech tree unlock actions
        for itype in &[InterceptorType::Sprint, InterceptorType::Exoatmospheric, InterceptorType::AreaDenial] {
            if self.campaign.tech_tree.can_unlock(*itype, self.wave_number, self.campaign.resources) {
                let (min_wave, cost) = upgrades::unlock_gate(*itype);
                available_actions.push(AvailableAction::UnlockInterceptor {
                    interceptor_type: itype.as_str().to_string(),
                    cost,
                    min_wave,
                });
            }
        }

        // Tech tree upgrade actions
        for itype in &self.campaign.tech_tree.unlocked_types.clone() {
            let up = self.campaign.tech_tree.upgrades.get(itype).cloned().unwrap_or_default();
            for axis in &[UpgradeAxis::Thrust, UpgradeAxis::Yield, UpgradeAxis::Guidance] {
                let level = up.level_for(*axis);
                if let Some(cost) = upgrades::upgrade_cost(*axis, level)
                    && self.campaign.resources >= cost
                {
                    available_actions.push(AvailableAction::UpgradeInterceptor {
                        interceptor_type: itype.as_str().to_string(),
                        axis: axis.as_str().to_string(),
                        cost,
                        current_level: level,
                    });
                }
            }
        }

        available_actions.push(AvailableAction::StartWave);

        // Build tech tree snapshot
        let tech_tree = TechTreeSnapshot {
            unlocked_types: self.campaign.tech_tree.unlocked_types
                .iter()
                .map(|t| t.as_str().to_string())
                .collect(),
            upgrades: self.campaign.tech_tree.unlocked_types
                .iter()
                .map(|t| {
                    let u = self.campaign.tech_tree.upgrades.get(t).cloned().unwrap_or_default();
                    TypeUpgradeSnapshot {
                        interceptor_type: t.as_str().to_string(),
                        thrust_level: u.thrust_level,
                        yield_level: u.yield_level,
                        guidance_level: u.guidance_level,
                    }
                })
                .collect(),
        };

        CampaignSnapshot {
            resources: self.campaign.resources,
            wave_number: self.wave_number,
            owned_region_ids: self.campaign.owned_regions.iter().map(|r| r.0).collect(),
            regions,
            available_actions,
            tech_tree,
            wave_income: None,
        }
    }

    /// Begin the next wave using wave composer.
    pub fn start_wave(&mut self) {
        self.wave_number += 1;
        self.weather = weather::generate_weather(&mut self.rng, self.wave_number);
        let def = wave_composer::compose_wave(
            self.wave_number,
            self.campaign.owned_regions.len() as u32,
            &self.weather,
        );
        self.wave = Some(WaveState::new(def));
        self.phase = GamePhase::WaveActive;
    }

    /// Queue a player command for processing next tick.
    pub fn push_command(&mut self, cmd: PlayerCommand) {
        self.input_queue.push(cmd);
    }

    /// Drain all pending game events.
    pub fn drain_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Build a snapshot without advancing the simulation.
    pub fn build_snapshot(&self) -> StateSnapshot {
        let phase_str = format!("{:?}", self.phase);
        let mut snapshot = systems::state_snapshot::build(&self.world, self.tick, self.wave_number, &phase_str);
        snapshot.weather = Some(self.weather.condition.as_str().to_string());
        snapshot.wind_x = Some(self.weather.wind_x);
        snapshot
    }

    /// Advance the simulation by one fixed timestep.
    pub fn tick(&mut self) -> StateSnapshot {
        let launched = systems::input_system::run(
            &mut self.world,
            &mut self.input_queue,
            &self.battery_ids,
            &self.campaign.tech_tree,
        );
        if let Some(ref mut wave) = self.wave {
            wave.interceptors_launched += launched;
        }

        if let Some(ref mut wave) = self.wave {
            systems::wave_spawner::run(
                &mut self.world,
                wave,
                &mut self.rng,
                &self.city_ids,
            );
        }

        systems::thrust::run(&mut self.world);
        systems::gravity::run(&mut self.world);
        systems::drag::run(&mut self.world);
        systems::wind::run(&mut self.world, &self.weather);
        systems::movement::run(&mut self.world);

        let mirv_result = systems::mirv_split::run(&mut self.world, self.tick);
        self.pending_events.extend(mirv_result.events);

        let collision_result = systems::collision::run(&mut self.world, self.tick);
        self.pending_events.extend(collision_result.events);
        if let Some(ref mut wave) = self.wave {
            wave.missiles_destroyed += collision_result.missiles_destroyed;
        }

        let detonation_result = systems::detonation::run(&mut self.world, self.tick);
        self.pending_events.extend(detonation_result.events);
        if let Some(ref mut wave) = self.wave {
            wave.missiles_impacted += detonation_result.missiles_impacted;
        }

        systems::shockwave_system::run(&mut self.world);

        let damage_events = systems::damage::run(&mut self.world, &self.city_ids, self.tick);
        self.pending_events.extend(damage_events);

        systems::detection::run(&mut self.world, &self.battery_ids, &self.weather);

        systems::cleanup::run(&mut self.world);

        self.check_wave_complete();

        self.tick += 1;
        self.build_snapshot()
    }

    fn check_wave_complete(&mut self) {
        let wave = match &self.wave {
            Some(w) => w,
            None => return,
        };

        if !wave.all_spawned() {
            return;
        }

        let missiles_alive = self.world.alive_entities().iter().any(|&idx| {
            self.world.markers[idx]
                .as_ref()
                .is_some_and(|m| m.kind == EntityKind::Missile)
        });
        if missiles_alive {
            return;
        }

        let shockwaves_alive = self.world.alive_entities().iter().any(|&idx| {
            self.world.markers[idx]
                .as_ref()
                .is_some_and(|m| m.kind == EntityKind::Shockwave)
        });
        if shockwaves_alive {
            return;
        }

        let cities_remaining = self
            .city_ids
            .iter()
            .filter(|&&id| {
                self.world.is_alive(id)
                    && self.world.healths[id.index as usize]
                        .as_ref()
                        .is_some_and(|h| h.current > 0.0)
            })
            .count() as u32;

        let wave = self.wave.as_ref().unwrap();
        self.pending_events
            .push(GameEvent::WaveComplete(WaveCompleteEvent {
                wave_number: self.wave_number,
                missiles_destroyed: wave.missiles_destroyed,
                missiles_impacted: wave.missiles_impacted,
                interceptors_launched: wave.interceptors_launched,
                cities_remaining,
                tick: self.tick,
            }));

        self.phase = GamePhase::WaveResult;
        self.wave = None;
    }

    fn battery_index_to_region(&self, battery_idx: usize) -> (RegionId, usize) {
        let mut idx = 0;
        for rid in &self.campaign.owned_regions {
            let region = self.campaign.get_region(*rid).unwrap();
            for (i, slot) in region.battery_slots.iter().enumerate() {
                if slot.occupied {
                    if idx == battery_idx {
                        return (*rid, i);
                    }
                    idx += 1;
                }
            }
        }
        (RegionId(0), 0)
    }

    fn city_index_to_region(&self, city_idx: usize) -> (RegionId, usize) {
        let mut idx = 0;
        for rid in &self.campaign.owned_regions {
            let region = self.campaign.get_region(*rid).unwrap();
            for i in 0..region.cities.len() {
                if idx == city_idx {
                    return (*rid, i);
                }
                idx += 1;
            }
        }
        (RegionId(0), 0)
    }

    fn sync_battery_ammo_at(&mut self, battery_idx: usize, ammo: u32) {
        let (rid, si) = self.battery_index_to_region(battery_idx);
        if let Some(entry) = self
            .campaign
            .battery_ammo
            .iter_mut()
            .find(|(r, s, _)| *r == rid && *s == si)
        {
            entry.2 = ammo;
        }
    }

    fn sync_city_health_at(&mut self, city_idx: usize, health: f32) {
        let (rid, ci) = self.city_index_to_region(city_idx);
        if let Some(entry) = self
            .campaign
            .city_healths
            .iter_mut()
            .find(|(r, c, _)| *r == rid && *c == ci)
        {
            entry.2 = health;
        }
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}
