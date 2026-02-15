//! Cleanup system: removes entities that are out of bounds or destroyed.

use hecs::{Entity, World};

use deterrence_core::components::{Interceptor, MissileState, Threat, ThreatProfile};
use deterrence_core::constants::WORLD_RADIUS;
use deterrence_core::enums::{MissilePhase, ThreatPhase};
use deterrence_core::types::Position;

/// Remove entities that are beyond the world boundary or in a terminal/dead state.
/// Uses a pre-allocated buffer to avoid per-tick allocation.
pub fn run(world: &mut World, despawn_buffer: &mut Vec<Entity>) {
    despawn_buffer.clear();

    let radius_sq = WORLD_RADIUS * WORLD_RADIUS;

    // Remove threats that are out of bounds (beyond WORLD_RADIUS from origin).
    for (entity, (pos, _threat)) in world.query_mut::<(&Position, &Threat)>() {
        let range_sq = pos.x * pos.x + pos.y * pos.y;
        if range_sq > radius_sq {
            despawn_buffer.push(entity);
        }
    }

    // Remove threats in Destroyed or Impact phase.
    for (entity, (profile, _threat)) in world.query_mut::<(&ThreatProfile, &Threat)>() {
        if matches!(profile.phase, ThreatPhase::Destroyed | ThreatPhase::Impact) {
            despawn_buffer.push(entity);
        }
    }

    // Remove interceptors that are out of bounds.
    for (entity, (pos, _interceptor)) in world.query_mut::<(&Position, &Interceptor)>() {
        let range_sq = pos.x * pos.x + pos.y * pos.y;
        if range_sq > radius_sq {
            despawn_buffer.push(entity);
        }
    }

    // Remove interceptors with completed missile phase.
    for (entity, (missile, _interceptor)) in world.query_mut::<(&MissileState, &Interceptor)>() {
        if matches!(missile.phase, MissilePhase::Complete) {
            despawn_buffer.push(entity);
        }
    }

    // Despawn collected entities.
    for entity in despawn_buffer.drain(..) {
        let _ = world.despawn(entity);
    }
}
