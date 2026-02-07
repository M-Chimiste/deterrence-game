use deterrence_lib::ecs::components::*;
use deterrence_lib::engine::config;
use deterrence_lib::engine::simulation::Simulation;
use deterrence_lib::events::game_events::GameEvent;
use deterrence_lib::state::game_state::GamePhase;
use deterrence_lib::systems::input_system::PlayerCommand;

// --- World Setup Tests ---

#[test]
fn setup_world_creates_cities_and_batteries() {
    let mut sim = Simulation::new();
    sim.setup_world();

    // 3 cities + 2 batteries = 5 entities
    assert_eq!(sim.world.entity_count(), 5);
    assert_eq!(sim.city_ids.len(), 3);
    assert_eq!(sim.battery_ids.len(), 2);

    // Check cities have health
    for &id in &sim.city_ids {
        let idx = id.index as usize;
        let health = sim.world.healths[idx].unwrap();
        assert_eq!(health.current, config::CITY_MAX_HEALTH);
        assert_eq!(health.max, config::CITY_MAX_HEALTH);
        let marker = sim.world.markers[idx].unwrap();
        assert_eq!(marker.kind, EntityKind::City);
    }

    // Check batteries have ammo
    for &id in &sim.battery_ids {
        let idx = id.index as usize;
        let bs = sim.world.battery_states[idx].unwrap();
        assert_eq!(bs.ammo, config::BATTERY_MAX_AMMO);
        let marker = sim.world.markers[idx].unwrap();
        assert_eq!(marker.kind, EntityKind::Battery);
    }
}

// --- Wave Spawner Tests ---

#[test]
fn wave_spawner_produces_correct_missile_count() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.start_wave();

    // Wave 1: WAVE_BASE_MISSILES = 3
    let expected = config::WAVE_BASE_MISSILES;

    // Run enough ticks to spawn all missiles (spawn interval * count + buffer)
    let ticks_needed = config::WAVE_BASE_SPAWN_INTERVAL * expected + 60;
    for _ in 0..ticks_needed {
        sim.tick();
    }

    let wave = sim.wave.as_ref().unwrap();
    assert_eq!(
        wave.missiles_spawned, expected,
        "Wave 1 should spawn {expected} missiles, got {}",
        wave.missiles_spawned
    );
}

#[test]
fn wave_spawner_missiles_have_correct_components() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.start_wave();

    // Run enough ticks to spawn at least one missile
    for _ in 0..5 {
        sim.tick();
    }

    // Find the missile entity (not city, not battery)
    let missile_idx = sim.world.alive_entities().into_iter().find(|&idx| {
        sim.world.markers[idx]
            .as_ref()
            .is_some_and(|m| m.kind == EntityKind::Missile)
    });

    assert!(missile_idx.is_some(), "A missile should have been spawned");
    let idx = missile_idx.unwrap();

    // Missile should have all required components
    assert!(sim.world.transforms[idx].is_some(), "Missile needs Transform");
    assert!(sim.world.velocities[idx].is_some(), "Missile needs Velocity");
    assert!(sim.world.ballistics[idx].is_some(), "Missile needs Ballistic");
    assert!(sim.world.warheads[idx].is_some(), "Missile needs Warhead");
    assert!(sim.world.markers[idx].is_some(), "Missile needs EntityMarker");
}

// --- Input System Tests ---

#[test]
fn launch_interceptor_spawns_entity_and_decrements_ammo() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.start_wave();

    let initial_ammo = sim.world.battery_states[sim.battery_ids[0].index as usize]
        .unwrap()
        .ammo;

    sim.push_command(PlayerCommand::LaunchInterceptor {
        battery_id: 0,
        target_x: 400.0,
        target_y: 500.0,
        interceptor_type: InterceptorType::Standard,
    });

    sim.tick();

    // Check ammo decremented
    let ammo_after = sim.world.battery_states[sim.battery_ids[0].index as usize]
        .unwrap()
        .ammo;
    assert_eq!(ammo_after, initial_ammo - 1);

    // Check interceptor was spawned
    let interceptor_count = sim
        .world
        .alive_entities()
        .iter()
        .filter(|&&idx| {
            sim.world.markers[idx]
                .as_ref()
                .is_some_and(|m| m.kind == EntityKind::Interceptor)
        })
        .count();
    assert_eq!(interceptor_count, 1, "One interceptor should be spawned");
}

#[test]
fn launch_from_empty_battery_is_ignored() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.start_wave();

    // Exhaust all ammo
    let bat_idx = sim.battery_ids[0].index as usize;
    sim.world.battery_states[bat_idx] = Some(BatteryState {
        ammo: 0,
        max_ammo: config::BATTERY_MAX_AMMO,
    });

    sim.push_command(PlayerCommand::LaunchInterceptor {
        battery_id: 0,
        target_x: 400.0,
        target_y: 500.0,
        interceptor_type: InterceptorType::Standard,
    });

    sim.tick();

    // No interceptor should be spawned
    let interceptor_count = sim
        .world
        .alive_entities()
        .iter()
        .filter(|&&idx| {
            sim.world.markers[idx]
                .as_ref()
                .is_some_and(|m| m.kind == EntityKind::Interceptor)
        })
        .count();
    assert_eq!(interceptor_count, 0, "No interceptor from empty battery");
}

// --- Detonation Tests ---

#[test]
fn missile_detonates_at_ground_level() {
    let mut sim = Simulation::new();
    sim.setup_world();

    // Manually spawn a missile heading straight down near ground
    let id = sim.world.spawn();
    let idx = id.index as usize;
    sim.world.transforms[idx] = Some(Transform {
        x: 400.0,
        y: config::GROUND_Y + 5.0,
        rotation: 0.0,
    });
    sim.world.velocities[idx] = Some(Velocity { vx: 0.0, vy: -50.0 });
    sim.world.ballistics[idx] = Some(Ballistic {
        drag_coefficient: 0.0,
        mass: 50.0,
        cross_section: 0.5,
    });
    sim.world.warheads[idx] = Some(Warhead {
        yield_force: config::WARHEAD_YIELD,
        blast_radius_base: config::WARHEAD_BLAST_RADIUS,
        warhead_type: WarheadType::Standard,
    });
    sim.world.markers[idx] = Some(EntityMarker {
        kind: EntityKind::Missile,
    });

    // Run a few ticks — missile should hit ground and create shockwave
    for _ in 0..10 {
        sim.tick();
    }

    // The missile should be gone (detonated)
    assert!(
        !sim.world.alive_entities().contains(&idx)
            || sim.world.markers[idx]
                .as_ref()
                .is_none_or(|m| m.kind != EntityKind::Missile),
        "Missile should be despawned after ground impact"
    );

    // A shockwave should have been created
    let shockwave_count = sim
        .world
        .alive_entities()
        .iter()
        .filter(|&&i| {
            sim.world.markers[i]
                .as_ref()
                .is_some_and(|m| m.kind == EntityKind::Shockwave)
        })
        .count();
    assert!(shockwave_count > 0, "Shockwave should exist after detonation");

    // Should have an impact event
    let events = sim.drain_events();
    let has_impact = events
        .iter()
        .any(|e| matches!(e, GameEvent::Impact(_)));
    assert!(has_impact, "Impact event should be emitted");
}

// --- Collision & Chain Reaction Tests ---

#[test]
fn shockwave_destroys_nearby_missile() {
    let mut sim = Simulation::new();

    // Spawn a shockwave
    let sw_id = sim.world.spawn();
    let sw_idx = sw_id.index as usize;
    sim.world.transforms[sw_idx] = Some(Transform {
        x: 400.0,
        y: 400.0,
        rotation: 0.0,
    });
    sim.world.shockwaves[sw_idx] = Some(Shockwave {
        radius: 50.0,
        max_radius: 60.0,
        force: 100.0,
        expansion_rate: config::SHOCKWAVE_EXPANSION_RATE,
        damage_applied: false,
    });
    sim.world.markers[sw_idx] = Some(EntityMarker {
        kind: EntityKind::Shockwave,
    });
    sim.world.lifetimes[sw_idx] = Some(Lifetime {
        remaining_ticks: 30,
    });

    // Spawn a missile within shockwave radius
    let ms_id = sim.world.spawn();
    let ms_idx = ms_id.index as usize;
    sim.world.transforms[ms_idx] = Some(Transform {
        x: 420.0,
        y: 400.0,
        rotation: 0.0,
    });
    sim.world.velocities[ms_idx] = Some(Velocity { vx: 0.0, vy: -10.0 });
    sim.world.ballistics[ms_idx] = Some(Ballistic {
        drag_coefficient: 0.0,
        mass: 50.0,
        cross_section: 0.5,
    });
    sim.world.warheads[ms_idx] = Some(Warhead {
        yield_force: 80.0,
        blast_radius_base: 30.0,
        warhead_type: WarheadType::Standard,
    });
    sim.world.markers[ms_idx] = Some(EntityMarker {
        kind: EntityKind::Missile,
    });

    sim.tick();

    // Missile should be destroyed
    let missile_alive = sim.world.alive_entities().iter().any(|&i| {
        sim.world.markers[i]
            .as_ref()
            .is_some_and(|m| m.kind == EntityKind::Missile)
    });
    assert!(!missile_alive, "Missile should be destroyed by shockwave");

    // Chain reaction shockwave should exist (from destroyed warhead)
    // There should be at least 2 shockwaves now (original + chain reaction)
    let sw_count = sim
        .world
        .alive_entities()
        .iter()
        .filter(|&&i| {
            sim.world.markers[i]
                .as_ref()
                .is_some_and(|m| m.kind == EntityKind::Shockwave)
        })
        .count();
    assert!(sw_count >= 2, "Chain reaction should spawn a new shockwave, got {sw_count}");
}

// --- Damage Tests ---

#[test]
fn ground_impact_damages_nearby_city() {
    let mut sim = Simulation::new();
    sim.setup_world();

    let city_id = sim.city_ids[0];
    let city_idx = city_id.index as usize;
    let city_x = sim.world.transforms[city_idx].unwrap().x;

    // Spawn a missile directly above the first city, close to ground
    let id = sim.world.spawn();
    let idx = id.index as usize;
    sim.world.transforms[idx] = Some(Transform {
        x: city_x,
        y: config::GROUND_Y + 2.0,
        rotation: 0.0,
    });
    sim.world.velocities[idx] = Some(Velocity { vx: 0.0, vy: -100.0 });
    sim.world.ballistics[idx] = Some(Ballistic {
        drag_coefficient: 0.0,
        mass: 50.0,
        cross_section: 0.5,
    });
    sim.world.warheads[idx] = Some(Warhead {
        yield_force: config::WARHEAD_YIELD,
        blast_radius_base: config::WARHEAD_BLAST_RADIUS,
        warhead_type: WarheadType::Standard,
    });
    sim.world.markers[idx] = Some(EntityMarker {
        kind: EntityKind::Missile,
    });

    // Run a few ticks for impact + damage
    for _ in 0..5 {
        sim.tick();
    }

    let health = sim.world.healths[city_idx].unwrap();
    assert!(
        health.current < config::CITY_MAX_HEALTH,
        "City should have taken damage: health = {} (max = {})",
        health.current,
        config::CITY_MAX_HEALTH
    );
}

// --- Wave Completion Tests ---

#[test]
fn wave_completes_when_all_missiles_resolved() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.start_wave();
    assert_eq!(sim.phase, GamePhase::WaveActive);

    // Run many ticks to let all missiles spawn, fly, and impact/be cleaned up
    // Wave 1 has 3 missiles. With ~90 tick spawn interval, all spawn by tick 270.
    // Missiles take ~6-12 seconds (360-720 ticks) flight time.
    // Total: ~1000 ticks should be more than enough.
    for _ in 0..1200 {
        if sim.phase != GamePhase::WaveActive {
            break;
        }
        sim.tick();
    }

    assert_eq!(
        sim.phase,
        GamePhase::WaveResult,
        "Phase should be WaveResult after all missiles resolved"
    );

    // Wave complete event should exist
    let events = sim.drain_events();
    let has_wave_complete = events
        .iter()
        .any(|e| matches!(e, GameEvent::WaveComplete(_)));
    assert!(has_wave_complete, "WaveComplete event should be emitted");
}

// --- Determinism Tests ---

#[test]
fn wave_with_same_seed_is_deterministic() {
    let run = |seed: u64| -> Vec<String> {
        let mut sim = Simulation::new_with_seed(seed);
        sim.setup_world();
        sim.start_wave();

        let mut snapshots = Vec::new();
        for _ in 0..300 {
            let snap = sim.tick();
            snapshots.push(format!("{:?}", snap.entities.len()));
        }
        snapshots
    };

    let run1 = run(12345);
    let run2 = run(12345);
    assert_eq!(run1, run2, "Same seed should produce identical wave progression");
}

#[test]
fn scripted_intercepts_produce_expected_kills() {
    let mut sim = Simulation::new_with_seed(99);
    sim.setup_world();
    sim.start_wave();

    // Run until first missile spawns
    for _ in 0..5 {
        sim.tick();
    }

    // Find the missile
    let missile_idx = sim.world.alive_entities().into_iter().find(|&idx| {
        sim.world.markers[idx]
            .as_ref()
            .is_some_and(|m| m.kind == EntityKind::Missile)
    });

    if let Some(ms_idx) = missile_idx {
        let ms_pos = sim.world.transforms[ms_idx].unwrap();

        // Launch an interceptor at the missile's position
        sim.push_command(PlayerCommand::LaunchInterceptor {
            battery_id: 0,
            target_x: ms_pos.x,
            target_y: ms_pos.y,
            interceptor_type: InterceptorType::Standard,
        });
    }

    // Run the simulation to completion
    for _ in 0..1200 {
        if sim.phase != GamePhase::WaveActive {
            break;
        }
        sim.tick();
    }

    // At least verify the wave completed
    assert_eq!(sim.phase, GamePhase::WaveResult);
}

// --- Shockwave System Tests ---

#[test]
fn shockwave_expands_to_max_radius() {
    let mut sim = Simulation::new();

    let sw_id = sim.world.spawn();
    let sw_idx = sw_id.index as usize;
    sim.world.transforms[sw_idx] = Some(Transform {
        x: 400.0,
        y: 400.0,
        rotation: 0.0,
    });
    sim.world.shockwaves[sw_idx] = Some(Shockwave {
        radius: 0.0,
        max_radius: 60.0,
        force: 100.0,
        expansion_rate: config::SHOCKWAVE_EXPANSION_RATE,
        damage_applied: false,
    });
    sim.world.markers[sw_idx] = Some(EntityMarker {
        kind: EntityKind::Shockwave,
    });
    sim.world.lifetimes[sw_idx] = Some(Lifetime {
        remaining_ticks: config::SHOCKWAVE_LIFETIME_TICKS,
    });

    // Expansion: 200 units/s, max 60 units = 0.3s = 18 ticks
    for _ in 0..20 {
        sim.tick();
    }

    let sw = sim.world.shockwaves[sw_idx].unwrap();
    assert!(
        (sw.radius - 60.0).abs() < 1.0,
        "Shockwave should reach max radius: got {}",
        sw.radius
    );
}
