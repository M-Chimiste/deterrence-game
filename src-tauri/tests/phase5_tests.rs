use deterrence_lib::campaign::territory::RegionId;
use deterrence_lib::engine::config;
use deterrence_lib::engine::simulation::Simulation;
use deterrence_lib::state::campaign_state::AvailableAction;
use deterrence_lib::state::game_state::GamePhase;

// --- Campaign Default State ---

#[test]
fn default_simulation_has_campaign_state() {
    let mut sim = Simulation::new();
    sim.setup_world();

    assert_eq!(sim.campaign.resources, 100);
    assert_eq!(sim.campaign.owned_regions.len(), 1);
    assert_eq!(sim.campaign.owned_regions[0], RegionId(0));
    // World should match campaign: 3 cities, 2 batteries
    assert_eq!(sim.city_ids.len(), 3);
    assert_eq!(sim.battery_ids.len(), 2);
}

// --- Economy: Wave Income ---

#[test]
fn wave_income_from_full_health_homeland() {
    let mut sim = Simulation::new();
    sim.setup_world();

    // All 3 cities at full health, population 500 each, multiplier 1.0
    // Income = (500 + 500 + 500) / 10 = 150
    let income = sim.apply_wave_income();
    assert_eq!(income, 150);
    assert_eq!(sim.campaign.resources, 100 + 150); // started with 100
    assert_eq!(sim.campaign.total_waves_survived, 1);
}

#[test]
fn wave_income_scales_with_city_damage() {
    let mut sim = Simulation::new();
    sim.setup_world();

    // Damage first city to half health
    let city0 = sim.city_ids[0];
    sim.world.healths[city0.index as usize].as_mut().unwrap().current = 50.0;
    sim.sync_to_campaign();

    let income = sim.apply_wave_income();
    // City 0: 500 * 0.5 = 250, City 1: 500 * 1.0 = 500, City 2: 500 * 1.0 = 500
    // Total = 1250 / 10 = 125
    assert_eq!(income, 125);
}

// --- Strategic Actions: Expand Region ---

#[test]
fn expand_region_succeeds_with_resources() {
    let mut sim = Simulation::new();
    sim.setup_world();

    // Region 1 costs 150, we have 100. Give extra resources.
    sim.campaign.resources = 200;
    let result = sim.expand_region(1);
    assert!(result.is_ok());
    assert_eq!(sim.campaign.resources, 200 - 150); // 50 remaining
    assert!(sim.campaign.owned_regions.contains(&RegionId(1)));

    // World rebuilt with new cities/batteries
    // Homeland: 3 cities + 2 batteries, Region 1: 1 city + 0 batteries (slots unoccupied)
    assert_eq!(sim.city_ids.len(), 4);
    assert_eq!(sim.battery_ids.len(), 2); // no new batteries (slots not occupied)
}

#[test]
fn expand_region_fails_insufficient_resources() {
    let mut sim = Simulation::new();
    sim.setup_world();

    // Region 1 costs 150, we have 100
    let result = sim.expand_region(1);
    assert!(result.is_err());
    assert_eq!(sim.campaign.owned_regions.len(), 1); // unchanged
}

#[test]
fn expand_region_fails_non_adjacent() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.campaign.resources = 1000;

    // Region 3 is not adjacent to homeland (need region 1 first)
    let result = sim.expand_region(3);
    assert!(result.is_err());
}

#[test]
fn expand_region_fails_already_owned() {
    let mut sim = Simulation::new();
    sim.setup_world();

    let result = sim.expand_region(0); // homeland already owned
    assert!(result.is_err());
}

// --- Strategic Actions: Place Battery ---

#[test]
fn place_battery_succeeds() {
    let mut sim = Simulation::new();
    sim.setup_world();

    // First expand to region 1 which has 2 empty battery slots
    sim.campaign.resources = 500;
    sim.expand_region(1).unwrap();

    let resources_before = sim.campaign.resources;
    let result = sim.place_battery(1, 0); // slot 0 in region 1
    assert!(result.is_ok());
    assert_eq!(sim.campaign.resources, resources_before - 100); // place_battery cost = 100

    // Should now have 3 batteries (2 homeland + 1 new)
    assert_eq!(sim.battery_ids.len(), 3);
}

#[test]
fn place_battery_fails_slot_occupied() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.campaign.resources = 500;
    sim.expand_region(1).unwrap();
    sim.place_battery(1, 0).unwrap();

    // Try to place at same slot again
    let result = sim.place_battery(1, 0);
    assert!(result.is_err());
}

#[test]
fn place_battery_fails_region_not_owned() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.campaign.resources = 500;

    let result = sim.place_battery(1, 0); // region 1 not owned
    assert!(result.is_err());
}

// --- Strategic Actions: Restock Battery ---

#[test]
fn restock_battery_succeeds() {
    let mut sim = Simulation::new();
    sim.setup_world();

    // Deplete battery ammo
    let bat0 = sim.battery_ids[0];
    sim.world.battery_states[bat0.index as usize].as_mut().unwrap().ammo = 0;

    let resources_before = sim.campaign.resources;
    let result = sim.restock_battery(0);
    assert!(result.is_ok());
    assert_eq!(sim.campaign.resources, resources_before - 30); // restock cost = 30

    let ammo = sim.world.battery_states[bat0.index as usize].unwrap().ammo;
    assert_eq!(ammo, config::BATTERY_MAX_AMMO);
}

#[test]
fn restock_battery_fails_already_full() {
    let mut sim = Simulation::new();
    sim.setup_world();

    let result = sim.restock_battery(0);
    assert!(result.is_err());
}

#[test]
fn restock_battery_fails_insufficient_resources() {
    let mut sim = Simulation::new();
    sim.setup_world();

    let bat0 = sim.battery_ids[0];
    sim.world.battery_states[bat0.index as usize].as_mut().unwrap().ammo = 0;
    sim.campaign.resources = 0;

    let result = sim.restock_battery(0);
    assert!(result.is_err());
}

// --- Strategic Actions: Repair City ---

#[test]
fn repair_city_succeeds() {
    let mut sim = Simulation::new();
    sim.setup_world();

    // Damage city to 50 health
    let city0 = sim.city_ids[0];
    sim.world.healths[city0.index as usize].as_mut().unwrap().current = 50.0;

    let resources_before = sim.campaign.resources;
    let result = sim.repair_city(0);
    assert!(result.is_ok());

    // Cost: 50 damage * 2 per hp = 100
    assert_eq!(sim.campaign.resources, resources_before - 100);

    let health = sim.world.healths[city0.index as usize].unwrap().current;
    assert_eq!(health, config::CITY_MAX_HEALTH);
}

#[test]
fn repair_city_fails_already_full() {
    let mut sim = Simulation::new();
    sim.setup_world();

    let result = sim.repair_city(0);
    assert!(result.is_err());
}

// --- Campaign Snapshot ---

#[test]
fn campaign_snapshot_has_correct_structure() {
    let mut sim = Simulation::new();
    sim.setup_world();

    let snapshot = sim.build_campaign_snapshot();
    assert_eq!(snapshot.resources, 100);
    assert_eq!(snapshot.wave_number, 0);
    assert_eq!(snapshot.owned_region_ids, vec![0]);
    assert_eq!(snapshot.regions.len(), 5);

    // Homeland region
    let homeland = &snapshot.regions[0];
    assert!(homeland.owned);
    assert!(!homeland.expandable);
    assert_eq!(homeland.cities.len(), 3);
    assert_eq!(homeland.battery_slots.len(), 2);

    // Region 1 should be expandable
    let region1 = &snapshot.regions[1];
    assert!(!region1.owned);
    assert!(region1.expandable);

    // Available actions should include StartWave and expandable regions (if affordable)
    let has_start_wave = snapshot.available_actions.iter().any(|a| matches!(a, AvailableAction::StartWave));
    assert!(has_start_wave);
}

// --- World Rebuild Preserves Campaign State ---

#[test]
fn rebuild_world_preserves_campaign_state() {
    let mut sim = Simulation::new();
    sim.setup_world();

    // Damage a city
    let city0 = sim.city_ids[0];
    sim.world.healths[city0.index as usize].as_mut().unwrap().current = 60.0;

    // Deplete battery ammo
    let bat0 = sim.battery_ids[0];
    sim.world.battery_states[bat0.index as usize].as_mut().unwrap().ammo = 3;

    // Sync to campaign, then rebuild
    sim.sync_to_campaign();
    sim.rebuild_world();

    // City health should be preserved
    let city0 = sim.city_ids[0];
    let health = sim.world.healths[city0.index as usize].unwrap().current;
    assert!((health - 60.0).abs() < 0.01, "City health should be preserved: got {health}");

    // Battery ammo should be preserved
    let bat0 = sim.battery_ids[0];
    let ammo = sim.world.battery_states[bat0.index as usize].unwrap().ammo;
    assert_eq!(ammo, 3, "Battery ammo should be preserved");
}

// --- Full Campaign Cycle ---

#[test]
fn full_cycle_strategic_to_wave_to_strategic() {
    let mut sim = Simulation::new();
    sim.setup_world();
    assert_eq!(sim.phase, GamePhase::Strategic);

    // Start wave
    sim.start_wave();
    assert_eq!(sim.phase, GamePhase::WaveActive);
    assert_eq!(sim.wave_number, 1);

    // Run wave to completion
    for _ in 0..1200 {
        if sim.phase != GamePhase::WaveActive {
            break;
        }
        sim.tick();
    }
    assert_eq!(sim.phase, GamePhase::WaveResult);

    // Transition to strategic
    sim.sync_to_campaign();
    let income = sim.apply_wave_income();
    assert!(income > 0, "Should earn income from surviving cities");
    sim.phase = GamePhase::Strategic;
    sim.rebuild_world();

    assert_eq!(sim.phase, GamePhase::Strategic);
    assert_eq!(sim.campaign.total_waves_survived, 1);
    assert!(sim.campaign.resources > 100, "Resources should increase from wave income");
}

// --- Wave Composer Integration ---

#[test]
fn wave_composer_scales_with_territory() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.campaign.resources = 1000;

    // Start wave 1 with 1 region
    sim.start_wave();
    let wave1 = sim.wave.as_ref().unwrap();
    let missiles_wave1 = wave1.definition.missile_count;

    // Reset for comparison
    let mut sim2 = Simulation::new();
    sim2.setup_world();
    sim2.campaign.resources = 1000;
    sim2.expand_region(1).unwrap();

    // Start wave 1 with 2 regions
    sim2.start_wave();
    let wave1_2regions = sim2.wave.as_ref().unwrap();
    let missiles_wave1_2regions = wave1_2regions.definition.missile_count;

    assert!(
        missiles_wave1_2regions >= missiles_wave1,
        "More territory should mean equal or more missiles: {} vs {}",
        missiles_wave1_2regions, missiles_wave1
    );
}

// --- Backward Compatibility ---

#[test]
fn default_campaign_matches_original_layout() {
    let mut sim = Simulation::new();
    sim.setup_world();

    // Verify city positions match config::CITY_POSITIONS
    for (i, &(expected_x, expected_y)) in config::CITY_POSITIONS.iter().enumerate() {
        let city = sim.city_ids[i];
        let t = sim.world.transforms[city.index as usize].unwrap();
        assert!((t.x - expected_x).abs() < 0.01, "City {i} x mismatch");
        assert!((t.y - expected_y).abs() < 0.01, "City {i} y mismatch");
    }

    // Verify battery positions match config::BATTERY_POSITIONS
    for (i, &(expected_x, expected_y)) in config::BATTERY_POSITIONS.iter().enumerate() {
        let bat = sim.battery_ids[i];
        let t = sim.world.transforms[bat.index as usize].unwrap();
        assert!((t.x - expected_x).abs() < 0.01, "Battery {i} x mismatch");
        assert!((t.y - expected_y).abs() < 0.01, "Battery {i} y mismatch");
    }
}
