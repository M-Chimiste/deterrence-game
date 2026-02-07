use crate::ecs::world::World;
use crate::engine::config;

/// Apply thrust to interceptors during their burn phase.
/// Thrust is applied in the direction from launch position toward target.
pub fn run(world: &mut World) {
    for idx in world.alive_entities() {
        let interceptor = match world.interceptors[idx].as_mut() {
            Some(i) if i.burn_remaining > 0.0 => i,
            _ => continue,
        };

        let transform = match world.transforms[idx] {
            Some(t) => t,
            None => continue,
        };

        // Calculate direction toward target
        let dx = interceptor.target_x - transform.x;
        let dy = interceptor.target_y - transform.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 1e-6 {
            interceptor.burn_remaining = 0.0;
            continue;
        }

        let dir_x = dx / dist;
        let dir_y = dy / dist;

        if let Some(ref mut vel) = world.velocities[idx] {
            let thrust_accel = interceptor.thrust * config::DT;
            vel.vx += dir_x * thrust_accel;
            vel.vy += dir_y * thrust_accel;
        }

        interceptor.burn_remaining -= config::DT;
        if interceptor.burn_remaining < 0.0 {
            interceptor.burn_remaining = 0.0;
        }
    }
}
