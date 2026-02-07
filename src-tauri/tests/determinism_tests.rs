use deterrence_lib::ecs::components::*;
use deterrence_lib::engine::config;
use deterrence_lib::engine::simulation::Simulation;

fn setup_scenario(sim: &mut Simulation) {
    // Spawn several missiles with different trajectories
    let missiles = [
        (100.0, 700.0, 50.0, -30.0, 0.1),
        (300.0, 650.0, -20.0, -50.0, 0.2),
        (640.0, 720.0, 10.0, -80.0, 0.05),
        (900.0, 680.0, -40.0, -20.0, 0.15),
    ];

    for (x, y, vx, vy, cd) in missiles {
        let id = sim.world.spawn();
        let idx = id.index as usize;
        sim.world.transforms[idx] = Some(Transform {
            x,
            y,
            rotation: 0.0,
        });
        sim.world.velocities[idx] = Some(Velocity { vx, vy });
        sim.world.ballistics[idx] = Some(Ballistic {
            drag_coefficient: cd,
            mass: 100.0,
            cross_section: 0.3,
        });
        sim.world.markers[idx] = Some(EntityMarker {
            kind: EntityKind::Missile,
        });
    }

    // Spawn an interceptor
    let id = sim.world.spawn();
    let idx = id.index as usize;
    sim.world.transforms[idx] = Some(Transform {
        x: 500.0,
        y: config::GROUND_Y,
        rotation: 0.0,
    });
    sim.world.velocities[idx] = Some(Velocity { vx: 0.0, vy: 0.0 });
    sim.world.ballistics[idx] = Some(Ballistic {
        drag_coefficient: 0.05,
        mass: 50.0,
        cross_section: 0.05,
    });
    sim.world.interceptors[idx] = Some(Interceptor {
        interceptor_type: InterceptorType::Standard,
        thrust: config::INTERCEPTOR_THRUST,
        burn_time: config::INTERCEPTOR_BURN_TIME,
        burn_remaining: config::INTERCEPTOR_BURN_TIME,
        ceiling: config::INTERCEPTOR_CEILING,
        battery_id: 0,
        target_x: 300.0,
        target_y: 500.0,
    });
    sim.world.markers[idx] = Some(EntityMarker {
        kind: EntityKind::Interceptor,
    });
}

fn run_scenario(ticks: u64) -> String {
    let mut sim = Simulation::new();
    setup_scenario(&mut sim);

    let mut last_snapshot = sim.tick();
    for _ in 1..ticks {
        last_snapshot = sim.tick();
    }

    // Serialize the final snapshot to JSON for comparison
    serde_json::to_string(&last_snapshot).unwrap()
}

#[test]
fn identical_conditions_produce_identical_snapshots() {
    let run1 = run_scenario(120);
    let run2 = run_scenario(120);

    assert_eq!(
        run1, run2,
        "Two identical simulation runs must produce byte-identical snapshots"
    );
}

#[test]
fn determinism_over_longer_run() {
    let run1 = run_scenario(300);
    let run2 = run_scenario(300);

    assert_eq!(
        run1, run2,
        "Determinism must hold over 300 ticks (5 seconds)"
    );
}

#[test]
fn determinism_with_different_tick_counts_diverges() {
    let run_120 = run_scenario(120);
    let run_121 = run_scenario(121);

    assert_ne!(
        run_120, run_121,
        "Different tick counts should produce different snapshots"
    );
}
