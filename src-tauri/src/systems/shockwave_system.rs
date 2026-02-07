use crate::ecs::world::World;
use crate::engine::config;

/// Expand active shockwaves each tick. Cleanup handles despawn via lifetime.
pub fn run(world: &mut World) {
    for idx in world.alive_entities() {
        if let Some(ref mut sw) = world.shockwaves[idx]
            && sw.radius < sw.max_radius
        {
            sw.radius += sw.expansion_rate * config::DT;
            if sw.radius > sw.max_radius {
                sw.radius = sw.max_radius;
            }
        }
    }
}
