use crate::ecs::components::*;
use crate::ecs::world::World;
use crate::ecs::entity::EntityId;
use crate::engine::config;
use crate::state::wave_state::WaveState;
use rand::Rng;
use rand_chacha::ChaChaRng;

/// Spawn enemy missiles according to the wave schedule.
/// Uses seeded RNG for deterministic waves.
pub fn run(
    world: &mut World,
    wave: &mut WaveState,
    rng: &mut ChaChaRng,
    city_ids: &[EntityId],
) {
    if wave.all_spawned() || city_ids.is_empty() {
        return;
    }

    // Countdown spawn timer
    if wave.spawn_timer > 0 {
        wave.spawn_timer -= 1;
        return;
    }

    // Time to spawn a missile
    wave.spawn_timer = wave.definition.spawn_interval_ticks;
    wave.missiles_spawned += 1;

    // Pick a random alive city to target
    let alive_cities: Vec<&EntityId> = city_ids
        .iter()
        .filter(|&&id| {
            world.healths[id.index as usize]
                .as_ref()
                .is_some_and(|h| h.current > 0.0)
        })
        .collect();

    if alive_cities.is_empty() {
        return;
    }

    let city_id = alive_cities[rng.gen_range(0..alive_cities.len())];
    let city_pos = match world.transforms[city_id.index as usize] {
        Some(t) => t,
        None => return,
    };

    // Random spawn position along top edge
    let spawn_x: f32 = rng.gen_range(100.0..config::WORLD_WIDTH - 100.0);
    let spawn_y: f32 = config::WORLD_HEIGHT;

    // Random flight time (controls arc profile)
    let flight_time: f32 =
        rng.gen_range(wave.definition.flight_time_min..wave.definition.flight_time_max);

    // Calculate initial velocity to arc toward target under gravity (no-drag approximation)
    // y(T) = y0 + vy*T - 0.5*g*T²  →  vy = (y_target - y0)/T + 0.5*g*T
    // x(T) = x0 + vx*T              →  vx = (x_target - x0)/T
    let dx = city_pos.x - spawn_x;
    let dy = city_pos.y - spawn_y;
    let vx = dx / flight_time;
    let vy = dy / flight_time + 0.5 * config::GRAVITY * flight_time;

    // Spawn the missile entity
    let id = world.spawn();
    let idx = id.index as usize;

    world.transforms[idx] = Some(Transform {
        x: spawn_x,
        y: spawn_y,
        rotation: vy.atan2(vx),
    });

    world.velocities[idx] = Some(Velocity { vx, vy });

    world.ballistics[idx] = Some(Ballistic {
        drag_coefficient: config::MISSILE_DRAG_COEFF,
        mass: config::MISSILE_MASS,
        cross_section: config::MISSILE_CROSS_SECTION,
    });

    // Determine if this missile is a MIRV carrier
    let is_mirv = wave.mirv_spawned < wave.definition.mirv_count;
    if is_mirv {
        wave.mirv_spawned += 1;
        let split_altitude = rng.gen_range(config::MIRV_SPLIT_ALTITUDE_MIN..config::MIRV_SPLIT_ALTITUDE_MAX);
        world.mirv_carriers[idx] = Some(MirvCarrier {
            child_count: wave.definition.mirv_child_count,
            split_altitude,
            spread_angle: config::MIRV_SPREAD_ANGLE,
        });
        world.warheads[idx] = Some(Warhead {
            yield_force: 0.0, // carrier itself has no warhead effect
            blast_radius_base: 0.0,
            warhead_type: WarheadType::Mirv,
        });
    } else {
        world.warheads[idx] = Some(Warhead {
            yield_force: config::WARHEAD_YIELD,
            blast_radius_base: config::WARHEAD_BLAST_RADIUS,
            warhead_type: WarheadType::Standard,
        });
    }

    world.markers[idx] = Some(EntityMarker {
        kind: EntityKind::Missile,
    });

    world.reentry_glows[idx] = Some(ReentryGlow {
        intensity: 1.0,
        altitude_threshold: 200.0,
    });
}
