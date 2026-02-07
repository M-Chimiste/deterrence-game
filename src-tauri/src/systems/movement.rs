use crate::ecs::world::World;
use crate::engine::config;

/// Euler integration: apply velocity to position.
/// Also updates rotation to match velocity direction.
pub fn run(world: &mut World) {
    for idx in world.alive_entities() {
        let vel = match world.velocities[idx] {
            Some(v) => v,
            None => continue,
        };

        if let Some(ref mut transform) = world.transforms[idx] {
            transform.x += vel.vx * config::DT;
            transform.y += vel.vy * config::DT;

            // Update rotation to match velocity direction
            if vel.vx.abs() > 1e-6 || vel.vy.abs() > 1e-6 {
                transform.rotation = vel.vy.atan2(vel.vx);
            }
        }
    }
}
