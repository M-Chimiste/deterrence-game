use deterrence_lib::ecs::components::*;
use deterrence_lib::ecs::world::World;
use deterrence_lib::engine::config;
use deterrence_lib::engine::simulation::Simulation;

/// Helper: spawn a ballistic missile at (x, y) with velocity (vx, vy)
fn spawn_missile(world: &mut World, x: f32, y: f32, vx: f32, vy: f32) -> usize {
    let id = world.spawn();
    let idx = id.index as usize;
    world.transforms[idx] = Some(Transform {
        x,
        y,
        rotation: 0.0,
    });
    world.velocities[idx] = Some(Velocity { vx, vy });
    world.ballistics[idx] = Some(Ballistic {
        drag_coefficient: 0.0, // no drag for pure ballistic tests
        mass: 100.0,
        cross_section: 0.1,
    });
    world.markers[idx] = Some(EntityMarker {
        kind: EntityKind::Missile,
    });
    idx
}

/// Helper: spawn a missile with drag
fn spawn_missile_with_drag(
    world: &mut World,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    cd: f32,
) -> usize {
    let id = world.spawn();
    let idx = id.index as usize;
    world.transforms[idx] = Some(Transform {
        x,
        y,
        rotation: 0.0,
    });
    world.velocities[idx] = Some(Velocity { vx, vy });
    world.ballistics[idx] = Some(Ballistic {
        drag_coefficient: cd,
        mass: 100.0,
        cross_section: 0.5,
    });
    world.markers[idx] = Some(EntityMarker {
        kind: EntityKind::Missile,
    });
    idx
}

/// Helper: spawn an interceptor
fn spawn_interceptor(
    world: &mut World,
    x: f32,
    y: f32,
    target_x: f32,
    target_y: f32,
) -> usize {
    let id = world.spawn();
    let idx = id.index as usize;
    world.transforms[idx] = Some(Transform {
        x,
        y,
        rotation: 0.0,
    });
    world.velocities[idx] = Some(Velocity { vx: 0.0, vy: 0.0 });
    world.ballistics[idx] = Some(Ballistic {
        drag_coefficient: 0.0,
        mass: 50.0,
        cross_section: 0.05,
    });
    world.interceptors[idx] = Some(Interceptor {
        interceptor_type: InterceptorType::Standard,
        thrust: config::INTERCEPTOR_THRUST,
        burn_time: config::INTERCEPTOR_BURN_TIME,
        burn_remaining: config::INTERCEPTOR_BURN_TIME,
        ceiling: config::INTERCEPTOR_CEILING,
        battery_id: 0,
        target_x,
        target_y,
        proximity_fuse_radius: 0.0,
    });
    world.markers[idx] = Some(EntityMarker {
        kind: EntityKind::Interceptor,
    });
    idx
}

#[test]
fn freefall_matches_kinematics() {
    // Drop an object from height with zero horizontal velocity.
    // After N ticks of gravity: vy = -g * N * dt, y = y0 + sum of vy*dt
    let mut sim = Simulation::new();
    let y0 = 500.0;
    let idx = spawn_missile(&mut sim.world, 640.0, y0, 0.0, 0.0);

    let ticks = 60; // 1 second at 60Hz
    for _ in 0..ticks {
        sim.tick();
    }

    let transform = sim.world.transforms[idx].unwrap();
    let vel = sim.world.velocities[idx].unwrap();

    // After 1 second of freefall: vy ≈ -9.81 m/s
    let expected_vy = -config::GRAVITY * (ticks as f32) * config::DT;
    assert!(
        (vel.vy - expected_vy).abs() < 0.1,
        "Expected vy ≈ {expected_vy}, got {}",
        vel.vy
    );

    // y = y0 - 0.5 * g * t^2 ≈ 500 - 4.905 = 495.095
    // Euler integration accumulates differently, so allow some tolerance
    let t = ticks as f32 * config::DT;
    let expected_y_analytic = y0 - 0.5 * config::GRAVITY * t * t;
    assert!(
        (transform.y - expected_y_analytic).abs() < 1.0,
        "Expected y ≈ {expected_y_analytic}, got {}",
        transform.y
    );
}

#[test]
fn horizontal_projectile_arcs() {
    // Launch at 45 degrees with moderate speed to stay within world bounds
    let mut sim = Simulation::new();
    let speed = 50.0;
    let angle = std::f32::consts::FRAC_PI_4; // 45°
    let vx = speed * angle.cos();
    let vy = speed * angle.sin();
    let x0 = 400.0;
    let idx = spawn_missile(&mut sim.world, x0, config::GROUND_Y, vx, vy);

    // Track the arc
    let mut max_height = config::GROUND_Y;
    let mut peak_tick = 0;
    let mut last_x = x0;

    // time to peak ≈ vy/g = 35.4/9.81 ≈ 3.6s = 216 ticks, full flight ≈ 432 ticks
    for tick in 0..500 {
        sim.tick();
        if let Some(t) = sim.world.transforms[idx] {
            if t.y > max_height {
                max_height = t.y;
                peak_tick = tick;
            }
            last_x = t.x;
            if t.y < config::GROUND_Y && tick > 10 {
                break;
            }
        } else {
            break; // entity was cleaned up
        }
    }

    assert!(max_height > config::GROUND_Y + 10.0, "Missile should arc upward, max_height={max_height}");
    assert!(peak_tick > 0, "Peak should not be at start");
    assert!(
        last_x > x0 + 50.0,
        "Missile should travel horizontally: got {last_x}"
    );
}

#[test]
fn drag_reduces_speed() {
    // An object moving fast through low altitude (dense air) should lose speed to drag
    let mut sim = Simulation::new();
    // Start at low altitude with high horizontal speed — drag should slow it down
    let idx = spawn_missile_with_drag(
        &mut sim.world,
        400.0,
        config::GROUND_Y + 100.0, // low altitude = dense air
        200.0, // fast horizontal
        0.0,
        0.5, // significant drag
    );

    let initial_vx = sim.world.velocities[idx].unwrap().vx;

    // Run 30 ticks (0.5 seconds)
    for _ in 0..30 {
        sim.tick();
    }

    if let Some(vel) = sim.world.velocities[idx] {
        assert!(
            vel.vx.abs() < initial_vx.abs(),
            "Drag should reduce horizontal speed: initial={initial_vx}, current={}",
            vel.vx
        );
    }
}

#[test]
fn drag_stronger_at_low_altitude() {
    // Same missile at two different altitudes — low altitude should experience more drag
    let mut sim_low = Simulation::new();
    let idx_low = spawn_missile_with_drag(
        &mut sim_low.world,
        400.0,
        config::GROUND_Y + 50.0,
        100.0,
        0.0,
        0.3,
    );

    let mut sim_high = Simulation::new();
    let idx_high = spawn_missile_with_drag(
        &mut sim_high.world,
        400.0,
        config::GROUND_Y + 400.0,
        100.0,
        0.0,
        0.3,
    );

    // Run 10 ticks
    for _ in 0..10 {
        sim_low.tick();
        sim_high.tick();
    }

    let vx_low = sim_low.world.velocities[idx_low].unwrap().vx;
    let vx_high = sim_high.world.velocities[idx_high].unwrap().vx;

    assert!(
        vx_low < vx_high,
        "Low altitude should have more drag: vx_low={vx_low}, vx_high={vx_high}"
    );
}

#[test]
fn thrust_accelerates_interceptor() {
    let mut sim = Simulation::new();
    let idx = spawn_interceptor(&mut sim.world, 400.0, config::GROUND_Y, 400.0, 500.0);

    // Run a few ticks during burn phase
    for _ in 0..30 {
        sim.tick();
    }

    let vel = sim.world.velocities[idx].unwrap();
    let transform = sim.world.transforms[idx].unwrap();

    // Interceptor should have gained velocity toward target
    let speed = (vel.vx * vel.vx + vel.vy * vel.vy).sqrt();
    assert!(speed > 10.0, "Interceptor should have accelerated: speed = {speed}");

    // Should have moved upward toward the target
    assert!(
        transform.y > config::GROUND_Y,
        "Interceptor should move toward target (upward)"
    );
}

#[test]
fn interceptor_goes_ballistic_after_burn() {
    let mut sim = Simulation::new();
    // Target very far away so detonation system doesn't trigger during test
    let idx = spawn_interceptor(&mut sim.world, 400.0, config::GROUND_Y, 400.0, 50000.0);

    // Burn time is 2 seconds = 120 ticks
    // Run just past burn time (125 ticks), keeping entity in bounds
    for _ in 0..125 {
        sim.tick();
    }

    let interceptor = sim.world.interceptors[idx].unwrap();
    assert!(
        interceptor.burn_remaining <= 0.0,
        "Burn should be exhausted: remaining = {}",
        interceptor.burn_remaining
    );

    // Record velocity, tick once more, check gravity is pulling it down
    let vel_before = sim.world.velocities[idx].unwrap();
    sim.tick();
    let vel_after = sim.world.velocities[idx].unwrap();

    // vy should decrease (gravity pulling down) since no more thrust
    assert!(
        vel_after.vy < vel_before.vy + 0.01,
        "After burn, gravity should pull interceptor down"
    );
}

#[test]
fn cleanup_removes_oob_entities() {
    let mut sim = Simulation::new();
    // Spawn entity way out of bounds
    let id = sim.world.spawn();
    let idx = id.index as usize;
    sim.world.transforms[idx] = Some(Transform {
        x: -500.0,
        y: -500.0,
        rotation: 0.0,
    });
    sim.world.markers[idx] = Some(EntityMarker {
        kind: EntityKind::Missile,
    });

    assert_eq!(sim.world.entity_count(), 1);
    sim.tick();
    assert_eq!(
        sim.world.entity_count(),
        0,
        "OOB entity should be cleaned up"
    );
}

#[test]
fn cleanup_removes_expired_entities() {
    let mut sim = Simulation::new();
    let id = sim.world.spawn();
    let idx = id.index as usize;
    sim.world.transforms[idx] = Some(Transform {
        x: 400.0,
        y: 400.0,
        rotation: 0.0,
    });
    sim.world.lifetimes[idx] = Some(Lifetime {
        remaining_ticks: 3,
    });
    sim.world.markers[idx] = Some(EntityMarker {
        kind: EntityKind::Shockwave,
    });

    assert_eq!(sim.world.entity_count(), 1);
    sim.tick(); // remaining: 2
    assert_eq!(sim.world.entity_count(), 1);
    sim.tick(); // remaining: 1
    assert_eq!(sim.world.entity_count(), 1);
    sim.tick(); // remaining: 0 → cleaned up
    assert_eq!(sim.world.entity_count(), 1); // 0 ticks: despawn happens next tick
    sim.tick(); // despawned
    assert_eq!(
        sim.world.entity_count(),
        0,
        "Expired entity should be removed"
    );
}

#[test]
fn state_snapshot_contains_entities() {
    let mut sim = Simulation::new();
    sim.setup_world(); // creates batteries for radar detection
    let base_entities = sim.world.entity_count(); // cities + batteries

    // Spawn missiles within radar range of batteries (500 unit base range)
    spawn_missile(&mut sim.world, 200.0, 300.0, 10.0, 50.0);
    spawn_missile(&mut sim.world, 300.0, 200.0, -5.0, 30.0);

    let snapshot = sim.tick();
    assert_eq!(
        snapshot.entities.len(),
        base_entities + 2,
        "Snapshot should contain base entities plus 2 missiles"
    );
    assert_eq!(snapshot.tick, 1);
}

#[test]
fn projectile_45_degree_range() {
    // Classic physics: range = v^2 * sin(2*theta) / g
    // At 45°: range = v^2 / g
    // With v=100, g=9.81: range ≈ 1019.4 meters
    // We use no drag to test pure ballistics
    let mut sim = Simulation::new();
    let speed = 100.0;
    let angle = std::f32::consts::FRAC_PI_4;
    let vx = speed * angle.cos();
    let vy = speed * angle.sin();
    let x0 = 100.0;
    let idx = spawn_missile(&mut sim.world, x0, config::GROUND_Y, vx, vy);

    // Run until it falls back to ground
    for tick in 0..2000 {
        sim.tick();
        let y = sim.world.transforms[idx].unwrap().y;
        if y <= config::GROUND_Y && tick > 10 {
            break;
        }
    }

    let final_x = sim.world.transforms[idx].unwrap().x;
    let range = final_x - x0;
    let expected_range = speed * speed / config::GRAVITY;

    // Euler integration has some error, allow ~5% tolerance
    let tolerance = expected_range * 0.05;
    assert!(
        (range - expected_range).abs() < tolerance,
        "Expected range ≈ {expected_range}, got {range} (tolerance: {tolerance})"
    );
}
