use deterrence_lib::campaign::upgrades::UpgradeAxis;
use deterrence_lib::ecs::components::*;
use deterrence_lib::ecs::world::World;
use deterrence_lib::engine::config;
use deterrence_lib::engine::simulation::Simulation;
use deterrence_lib::events::game_events::GameEvent;
use deterrence_lib::systems::input_system::PlayerCommand;

// --- Interceptor Type Tests ---

#[test]
fn standard_profile_matches_existing_constants() {
    let profile = config::interceptor_profile(InterceptorType::Standard);
    assert_eq!(profile.thrust, config::INTERCEPTOR_THRUST);
    assert_eq!(profile.burn_time, config::INTERCEPTOR_BURN_TIME);
    assert_eq!(profile.mass, config::INTERCEPTOR_MASS);
    assert_eq!(profile.blast_radius, config::WARHEAD_BLAST_RADIUS);
}

#[test]
fn sprint_profile_is_faster_and_shorter() {
    let std = config::interceptor_profile(InterceptorType::Standard);
    let sprint = config::interceptor_profile(InterceptorType::Sprint);
    assert!(sprint.thrust > std.thrust, "Sprint should have higher thrust");
    assert!(sprint.burn_time < std.burn_time, "Sprint should have shorter burn");
    assert!(sprint.ceiling < std.ceiling, "Sprint should have lower ceiling");
    assert!(sprint.blast_radius < std.blast_radius, "Sprint should have smaller blast");
}

#[test]
fn exo_profile_is_slower_and_wider() {
    let std = config::interceptor_profile(InterceptorType::Standard);
    let exo = config::interceptor_profile(InterceptorType::Exoatmospheric);
    assert!(exo.thrust < std.thrust, "Exo should have lower thrust");
    assert!(exo.burn_time > std.burn_time, "Exo should have longer burn");
    assert!(exo.ceiling > std.ceiling, "Exo should have higher ceiling");
    assert!(exo.blast_radius > std.blast_radius, "Exo should have wider blast");
}

#[test]
fn launch_with_interceptor_type_uses_correct_profile() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.start_wave();

    sim.push_command(PlayerCommand::LaunchInterceptor {
        battery_id: 0,
        target_x: 400.0,
        target_y: 400.0,
        interceptor_type: InterceptorType::Sprint,
    });
    sim.tick();

    // Find the spawned interceptor
    let interceptor_idx = sim.world.alive_entities().iter().find(|&&idx| {
        sim.world.markers[idx]
            .as_ref()
            .is_some_and(|m| m.kind == EntityKind::Interceptor)
    }).copied();

    assert!(interceptor_idx.is_some(), "Should have spawned interceptor");
    let idx = interceptor_idx.unwrap();
    let interceptor = sim.world.interceptors[idx].as_ref().unwrap();
    assert_eq!(interceptor.interceptor_type, InterceptorType::Sprint);
    assert_eq!(interceptor.thrust, config::SPRINT_THRUST);
    assert_eq!(interceptor.burn_time, config::SPRINT_BURN_TIME);
}

// --- MIRV Split Tests ---

fn spawn_mirv_carrier(world: &mut World, x: f32, y: f32, vy: f32, split_altitude: f32, child_count: u32) -> usize {
    let id = world.spawn();
    let idx = id.index as usize;
    world.transforms[idx] = Some(Transform { x, y, rotation: 0.0 });
    world.velocities[idx] = Some(Velocity { vx: 0.0, vy });
    world.ballistics[idx] = Some(Ballistic {
        drag_coefficient: config::MISSILE_DRAG_COEFF,
        mass: config::MISSILE_MASS,
        cross_section: config::MISSILE_CROSS_SECTION,
    });
    world.warheads[idx] = Some(Warhead {
        yield_force: 0.0,
        blast_radius_base: 0.0,
        warhead_type: WarheadType::Mirv,
    });
    world.markers[idx] = Some(EntityMarker { kind: EntityKind::Missile });
    world.mirv_carriers[idx] = Some(MirvCarrier {
        child_count,
        split_altitude,
        spread_angle: config::MIRV_SPREAD_ANGLE,
    });
    idx
}

#[test]
fn mirv_splits_at_altitude() {
    let mut world = World::new();
    spawn_mirv_carrier(&mut world, 640.0, 350.0, -50.0, 400.0, 3);

    // Carrier is at 350 (below split_altitude=400) and descending (vy=-50) → should split
    let result = deterrence_lib::systems::mirv_split::run(&mut world, 0);

    assert_eq!(result.splits, 1, "Should have split");
    // No MIRV carriers should remain (carrier despawned, children are standard)
    let mirv_count = world.alive_entities().iter().filter(|&&idx| {
        world.mirv_carriers[idx].is_some()
    }).count();
    assert_eq!(mirv_count, 0, "Carrier should be despawned (no MIRV carriers remain)");
}

#[test]
fn mirv_produces_correct_child_count() {
    let mut world = World::new();
    spawn_mirv_carrier(&mut world, 640.0, 350.0, -50.0, 400.0, 4);

    let initial_count = world.entity_count();
    deterrence_lib::systems::mirv_split::run(&mut world, 0);

    // Carrier despawned (-1) + 4 children spawned (+4) = net +3
    assert_eq!(world.entity_count(), initial_count - 1 + 4);

    // All children should be missiles
    let missile_count = world.alive_entities().iter().filter(|&&idx| {
        world.markers[idx].as_ref().is_some_and(|m| m.kind == EntityKind::Missile)
    }).count();
    assert_eq!(missile_count, 4);
}

#[test]
fn mirv_children_have_warheads() {
    let mut world = World::new();
    spawn_mirv_carrier(&mut world, 640.0, 350.0, -50.0, 400.0, 3);
    deterrence_lib::systems::mirv_split::run(&mut world, 0);

    for &idx in &world.alive_entities() {
        if world.markers[idx].as_ref().is_some_and(|m| m.kind == EntityKind::Missile) {
            assert!(world.warheads[idx].is_some(), "Child should have warhead");
            let wh = world.warheads[idx].unwrap();
            assert_eq!(wh.warhead_type, WarheadType::Standard, "Children should be standard");
            assert!(wh.yield_force > 0.0, "Children should have yield");
            assert!(world.mirv_carriers[idx].is_none(), "Children should NOT be MIRV carriers");
        }
    }
}

#[test]
fn mirv_children_have_spread_velocities() {
    let mut world = World::new();
    spawn_mirv_carrier(&mut world, 640.0, 350.0, -50.0, 400.0, 3);
    deterrence_lib::systems::mirv_split::run(&mut world, 0);

    let velocities: Vec<(f32, f32)> = world.alive_entities().iter().filter_map(|&idx| {
        if world.markers[idx].as_ref().is_some_and(|m| m.kind == EntityKind::Missile) {
            world.velocities[idx].as_ref().map(|v| (v.vx, v.vy))
        } else {
            None
        }
    }).collect();

    assert_eq!(velocities.len(), 3);
    // Children should not all have the same velocity (spread)
    assert!(
        velocities[0] != velocities[1] || velocities[1] != velocities[2],
        "Children should have divergent velocities: {:?}", velocities
    );
}

#[test]
fn mirv_no_split_while_ascending() {
    let mut world = World::new();
    // vy > 0 means ascending
    spawn_mirv_carrier(&mut world, 640.0, 350.0, 50.0, 400.0, 3);

    let result = deterrence_lib::systems::mirv_split::run(&mut world, 0);
    assert_eq!(result.splits, 0, "Should not split while ascending");
    assert_eq!(world.entity_count(), 1, "Carrier should still exist");
}

#[test]
fn mirv_no_split_above_altitude() {
    let mut world = World::new();
    // y=500 is above split_altitude=400, descending
    spawn_mirv_carrier(&mut world, 640.0, 500.0, -50.0, 400.0, 3);

    let result = deterrence_lib::systems::mirv_split::run(&mut world, 0);
    assert_eq!(result.splits, 0, "Should not split above split altitude");
}

#[test]
fn mirv_split_emits_event() {
    let mut world = World::new();
    spawn_mirv_carrier(&mut world, 640.0, 350.0, -50.0, 400.0, 3);

    let result = deterrence_lib::systems::mirv_split::run(&mut world, 42);

    assert_eq!(result.events.len(), 1);
    match &result.events[0] {
        GameEvent::MirvSplit(e) => {
            assert!((e.x - 640.0).abs() < 0.01);
            assert!((e.y - 350.0).abs() < 0.01);
            assert_eq!(e.child_count, 3);
            assert_eq!(e.tick, 42);
        }
        _ => panic!("Expected MirvSplit event"),
    }
}

// --- Chain Reaction / Dual-Zone Collision Tests ---

fn spawn_shockwave(world: &mut World, x: f32, y: f32, radius: f32, max_radius: f32, force: f32) -> usize {
    let id = world.spawn();
    let idx = id.index as usize;
    world.transforms[idx] = Some(Transform { x, y, rotation: 0.0 });
    world.shockwaves[idx] = Some(Shockwave {
        radius,
        max_radius,
        force,
        expansion_rate: config::SHOCKWAVE_EXPANSION_RATE,
        damage_applied: false,
    });
    world.markers[idx] = Some(EntityMarker { kind: EntityKind::Shockwave });
    world.lifetimes[idx] = Some(Lifetime { remaining_ticks: config::SHOCKWAVE_LIFETIME_TICKS });
    idx
}

fn spawn_missile(world: &mut World, x: f32, y: f32, vx: f32, vy: f32) -> usize {
    let id = world.spawn();
    let idx = id.index as usize;
    world.transforms[idx] = Some(Transform { x, y, rotation: 0.0 });
    world.velocities[idx] = Some(Velocity { vx, vy });
    world.warheads[idx] = Some(Warhead {
        yield_force: config::WARHEAD_YIELD,
        blast_radius_base: config::WARHEAD_BLAST_RADIUS,
        warhead_type: WarheadType::Standard,
    });
    world.markers[idx] = Some(EntityMarker { kind: EntityKind::Missile });
    idx
}

fn spawn_interceptor_entity(world: &mut World, x: f32, y: f32, vx: f32, vy: f32) -> usize {
    let id = world.spawn();
    let idx = id.index as usize;
    world.transforms[idx] = Some(Transform { x, y, rotation: 0.0 });
    world.velocities[idx] = Some(Velocity { vx, vy });
    world.markers[idx] = Some(EntityMarker { kind: EntityKind::Interceptor });
    world.interceptors[idx] = Some(Interceptor {
        interceptor_type: InterceptorType::Standard,
        thrust: config::INTERCEPTOR_THRUST,
        burn_time: config::INTERCEPTOR_BURN_TIME,
        burn_remaining: 0.0,
        ceiling: config::INTERCEPTOR_CEILING,
        battery_id: 0,
        target_x: x,
        target_y: y,
        proximity_fuse_radius: 0.0,
    });
    world.warheads[idx] = Some(Warhead {
        yield_force: config::WARHEAD_YIELD,
        blast_radius_base: config::WARHEAD_BLAST_RADIUS,
        warhead_type: WarheadType::Standard,
    });
    idx
}

#[test]
fn shockwave_destroys_interceptor_in_inner_zone() {
    let mut world = World::new();
    // Shockwave at (400, 400) with radius 50
    spawn_shockwave(&mut world, 400.0, 400.0, 50.0, 60.0, 100.0);
    // Interceptor at (410, 400) — distance=10, well inside destroy radius (50*0.7=35)
    let intc_idx = spawn_interceptor_entity(&mut world, 410.0, 400.0, 0.0, 0.0);

    let result = deterrence_lib::systems::collision::run(&mut world, 0);

    assert_eq!(result.interceptors_destroyed, 1, "Interceptor in inner zone should be destroyed");
    assert!(!world.alive_entities().contains(&intc_idx), "Interceptor should be despawned");
}

#[test]
fn destroyed_interceptor_does_not_chain_react() {
    let mut world = World::new();
    spawn_shockwave(&mut world, 400.0, 400.0, 50.0, 60.0, 100.0);
    spawn_interceptor_entity(&mut world, 410.0, 400.0, 0.0, 0.0);

    let result = deterrence_lib::systems::collision::run(&mut world, 0);

    // No detonation event should be emitted for the interceptor
    assert!(result.events.is_empty(), "Destroyed interceptor should NOT emit chain events");
    assert_eq!(result.interceptors_destroyed, 1);
    assert_eq!(result.missiles_destroyed, 0);

    // Only the original shockwave should remain (no chain shockwave spawned)
    let sw_count = world.alive_entities().iter().filter(|&&idx| {
        world.markers[idx].as_ref().is_some_and(|m| m.kind == EntityKind::Shockwave)
    }).count();
    assert_eq!(sw_count, 1, "Only original shockwave should remain, no chain reaction from interceptor");
}

#[test]
fn missile_in_edge_zone_gets_deflected() {
    let mut world = World::new();
    // Shockwave at (400, 400) with radius 50, force 100
    // Destroy zone inner radius: 50*0.7 = 35
    spawn_shockwave(&mut world, 400.0, 400.0, 50.0, 60.0, 100.0);
    // Missile at (440, 400) — distance=40, in deflect zone (35 < 40 < 50)
    let ms_idx = spawn_missile(&mut world, 440.0, 400.0, 0.0, -50.0);

    let vx_before = world.velocities[ms_idx].as_ref().unwrap().vx;

    let result = deterrence_lib::systems::collision::run(&mut world, 0);

    assert_eq!(result.missiles_destroyed, 0, "Missile in deflect zone should NOT be destroyed");
    assert!(world.alive_entities().contains(&ms_idx), "Missile should still be alive");

    // Velocity should have been nudged in the +x direction (away from shockwave center)
    let vx_after = world.velocities[ms_idx].as_ref().unwrap().vx;
    assert!(vx_after > vx_before, "Missile should be pushed away (vx increased): before={}, after={}", vx_before, vx_after);
}

#[test]
fn deflection_pushes_entity_away_from_center() {
    let mut world = World::new();
    // Shockwave at center
    spawn_shockwave(&mut world, 400.0, 400.0, 50.0, 60.0, 100.0);
    // Missile above the shockwave center, in deflect zone
    let ms_idx = spawn_missile(&mut world, 400.0, 440.0, 0.0, -50.0);

    deterrence_lib::systems::collision::run(&mut world, 0);

    // Should be pushed in +y direction (away from shockwave center below it)
    let vy_after = world.velocities[ms_idx].as_ref().unwrap().vy;
    assert!(vy_after > -50.0, "Missile should have vy pushed upward (away from center): {}", vy_after);
}

#[test]
fn deflection_is_deterministic() {
    // Run deflection twice with same setup, verify same result
    let run = || {
        let mut world = World::new();
        spawn_shockwave(&mut world, 400.0, 400.0, 50.0, 60.0, 100.0);
        let ms_idx = spawn_missile(&mut world, 440.0, 400.0, 10.0, -30.0);
        deterrence_lib::systems::collision::run(&mut world, 0);
        let vel = world.velocities[ms_idx].as_ref().unwrap();
        (vel.vx, vel.vy)
    };
    let (vx1, vy1) = run();
    let (vx2, vy2) = run();
    assert_eq!(vx1, vx2, "Deflection vx should be deterministic");
    assert_eq!(vy1, vy2, "Deflection vy should be deterministic");
}

#[test]
fn multi_step_cascade_over_ticks() {
    // Place a small shockwave that destroys missile A (nearby), which chain-reacts
    // into missile B (outside original range but inside chain range).
    let mut world = World::new();

    // Small shockwave: radius 15, destroy zone = 15*0.7 = 10.5
    spawn_shockwave(&mut world, 400.0, 400.0, 15.0, 20.0, 100.0);
    // Missile A: distance 2 from center → inside destroy zone
    spawn_missile(&mut world, 402.0, 400.0, 0.0, -50.0);
    // Missile B: distance 20 from center → outside radius 15 entirely
    // But distance 18 from missile A → will be in chain reaction destroy zone
    // (chain max_radius = WARHEAD_BLAST_RADIUS * 0.7 = 28, destroy zone = 28*0.7 ≈ 19.6)
    spawn_missile(&mut world, 420.0, 400.0, 0.0, -50.0);

    // First pass: original shockwave destroys missile A, spawning chain shockwave at (402, 400)
    let result1 = deterrence_lib::systems::collision::run(&mut world, 0);
    assert_eq!(result1.missiles_destroyed, 1, "First pass should destroy only missile A");

    // Chain shockwave starts at radius 0 — expand it over multiple ticks
    let mut total_destroyed = result1.missiles_destroyed;
    for tick in 1..30 {
        deterrence_lib::systems::shockwave_system::run(&mut world);
        let r = deterrence_lib::systems::collision::run(&mut world, tick);
        total_destroyed += r.missiles_destroyed;
        if total_destroyed >= 2 { break; }
    }

    assert_eq!(total_destroyed, 2, "Chain reaction should eventually destroy missile B");
}

// --- Area Denial Tests ---

#[test]
fn area_denial_shockwave_lingers() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.start_wave();

    // Launch an area denial interceptor at a nearby target
    sim.push_command(PlayerCommand::LaunchInterceptor {
        battery_id: 0,
        target_x: 200.0,
        target_y: 150.0,
        interceptor_type: InterceptorType::AreaDenial,
    });

    // Run until the interceptor detonates
    let mut shockwave_found = false;
    for _ in 0..600 {
        sim.tick();
        // Look for a shockwave
        for &idx in &sim.world.alive_entities() {
            if sim.world.markers[idx].as_ref().is_some_and(|m| m.kind == EntityKind::Shockwave)
                && let Some(lt) = &sim.world.lifetimes[idx]
                && lt.remaining_ticks > config::SHOCKWAVE_LIFETIME_TICKS
            {
                shockwave_found = true;
                // Check that it has area denial properties
                assert!(
                    lt.remaining_ticks > 100,
                    "Area denial shockwave should have long lifetime: {}",
                    lt.remaining_ticks
                );
                break;
            }
        }
        if shockwave_found { break; }
    }
    assert!(shockwave_found, "Should find an area denial shockwave with extended lifetime");
}

// --- Tech Tree & Upgrade Tests ---

#[test]
fn launch_uses_upgraded_stats() {
    let mut sim = Simulation::new();
    sim.setup_world();

    // Upgrade Standard thrust once (+15%)
    sim.campaign.tech_tree.apply_upgrade(InterceptorType::Standard, UpgradeAxis::Thrust, 999).unwrap();

    sim.start_wave();
    sim.push_command(PlayerCommand::LaunchInterceptor {
        battery_id: 0,
        target_x: 400.0,
        target_y: 400.0,
        interceptor_type: InterceptorType::Standard,
    });
    sim.tick();

    let interceptor_idx = sim.world.alive_entities().iter().find(|&&idx| {
        sim.world.markers[idx].as_ref().is_some_and(|m| m.kind == EntityKind::Interceptor)
    }).copied();

    assert!(interceptor_idx.is_some(), "Should have spawned interceptor");
    let idx = interceptor_idx.unwrap();
    let interceptor = sim.world.interceptors[idx].as_ref().unwrap();

    let base_thrust = config::INTERCEPTOR_THRUST;
    let expected_thrust = base_thrust * 1.15;
    assert!(
        (interceptor.thrust - expected_thrust).abs() < 0.1,
        "Interceptor should have upgraded thrust: expected {}, got {}",
        expected_thrust, interceptor.thrust
    );
}

#[test]
fn unlock_interceptor_via_simulation() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.wave_number = 8;
    sim.campaign.resources = 300;

    assert!(sim.unlock_interceptor(InterceptorType::Sprint).is_ok());
    assert!(sim.campaign.tech_tree.is_unlocked(InterceptorType::Sprint));
    assert_eq!(sim.campaign.resources, 100); // 300 - 200
}

#[test]
fn unlock_fails_before_wave_gate() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.wave_number = 7;
    sim.campaign.resources = 1000;

    assert!(sim.unlock_interceptor(InterceptorType::Sprint).is_err());
    assert!(!sim.campaign.tech_tree.is_unlocked(InterceptorType::Sprint));
}

#[test]
fn campaign_snapshot_includes_tech_tree() {
    let mut sim = Simulation::new();
    sim.setup_world();

    let snap = sim.build_campaign_snapshot();
    assert_eq!(snap.tech_tree.unlocked_types.len(), 1);
    assert_eq!(snap.tech_tree.unlocked_types[0], "Standard");
    assert_eq!(snap.tech_tree.upgrades.len(), 1);
    assert_eq!(snap.tech_tree.upgrades[0].thrust_level, 0);
}

#[test]
fn campaign_snapshot_shows_unlock_actions_when_eligible() {
    let mut sim = Simulation::new();
    sim.setup_world();
    sim.wave_number = 8;
    sim.campaign.resources = 500;

    let snap = sim.build_campaign_snapshot();
    let has_unlock = snap.available_actions.iter().any(|a| {
        matches!(a, deterrence_lib::state::campaign_state::AvailableAction::UnlockInterceptor { interceptor_type, .. } if interceptor_type == "Sprint")
    });
    assert!(has_unlock, "Should show Sprint unlock action at wave 8 with sufficient resources");
}
