use crate::ecs::components::EntityKind;
use crate::ecs::world::World;
use crate::engine::config;

/// Apply gravitational acceleration to all ballistic entities.
/// In our coordinate system, positive Y is up, so gravity subtracts from vy.
pub fn run(world: &mut World) {
    for idx in world.alive_entities() {
        // Only apply gravity to entities with velocity and ballistic components
        // Skip shockwaves and static entities (cities, batteries)
        let dominated_by_gravity = match &world.markers[idx] {
            Some(m) => matches!(m.kind, EntityKind::Missile | EntityKind::Interceptor),
            None => false,
        };

        if !dominated_by_gravity {
            continue;
        }

        if let Some(ref mut vel) = world.velocities[idx]
            && world.ballistics[idx].is_some()
        {
            vel.vy -= config::GRAVITY * config::DT;
        }
    }
}
