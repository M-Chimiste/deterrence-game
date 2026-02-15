//! Kinematic integration system.
//!
//! Updates Position from Velocity each tick: position += velocity * dt.
//! Also records position history for trail rendering.

use hecs::World;

use deterrence_core::components::PositionHistory;
use deterrence_core::constants::{DT, HISTORY_DOT_INTERVAL, MAX_HISTORY_DOTS};
use deterrence_core::types::{Position, Velocity};

/// Run kinematic integration for all entities with Position + Velocity.
pub fn run(world: &mut World) {
    for (_entity, (pos, vel)) in world.query_mut::<(&mut Position, &Velocity)>() {
        pos.x += vel.x * DT;
        pos.y += vel.y * DT;
        pos.z += vel.z * DT;
    }
}

/// Record position history for trail rendering.
/// Called after movement; only records a dot every HISTORY_DOT_INTERVAL ticks.
pub fn update_history(world: &mut World, current_tick: u64) {
    if current_tick == 0 || !current_tick.is_multiple_of(HISTORY_DOT_INTERVAL as u64) {
        return;
    }

    for (_entity, (pos, history)) in world.query_mut::<(&Position, &mut PositionHistory)>() {
        history.positions.insert(0, *pos);
        history.positions.truncate(MAX_HISTORY_DOTS);
    }
}
