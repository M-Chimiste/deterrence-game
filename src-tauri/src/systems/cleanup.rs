use crate::ecs::entity::EntityId;
use crate::ecs::world::World;
use crate::engine::config;

/// Remove entities that are expired (lifetime) or out of bounds.
pub fn run(world: &mut World) {
    let mut to_despawn: Vec<EntityId> = Vec::new();

    for idx in world.alive_entities() {
        let mut should_despawn = false;

        // Check lifetime expiry
        if let Some(ref mut lifetime) = world.lifetimes[idx] {
            if lifetime.remaining_ticks == 0 {
                should_despawn = true;
            } else {
                lifetime.remaining_ticks -= 1;
            }
        }

        // Check out of bounds
        if let Some(ref transform) = world.transforms[idx] {
            let margin = config::OOB_MARGIN;
            if transform.x < -margin
                || transform.x > config::WORLD_WIDTH + margin
                || transform.y < -margin
                || transform.y > config::WORLD_HEIGHT + margin
            {
                should_despawn = true;
            }
        }

        if should_despawn {
            // Reconstruct the EntityId from index
            // We need to look up the generation from the allocator
            if let Some(generation) = world.allocator.generation_of(idx as u32) {
                to_despawn.push(EntityId::new(idx as u32, generation));
            }
        }
    }

    for id in to_despawn {
        world.despawn(id);
    }
}
