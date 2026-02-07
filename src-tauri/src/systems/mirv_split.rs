use crate::ecs::components::*;
use crate::ecs::entity::EntityId;
use crate::ecs::world::World;
use crate::engine::config;
use crate::events::game_events::{GameEvent, MirvSplitEvent};

pub struct MirvSplitResult {
    pub events: Vec<GameEvent>,
    pub splits: u32,
}

/// Check MIRV carriers for split conditions: descending below split altitude.
/// Spawn child warheads in a fan pattern and despawn the carrier.
pub fn run(world: &mut World, tick: u64) -> MirvSplitResult {
    let mut result = MirvSplitResult {
        events: Vec::new(),
        splits: 0,
    };

    // Gather carriers that should split
    let mut to_split: Vec<(usize, f32, f32, f32, f32, MirvCarrier)> = Vec::new();

    for idx in world.alive_entities() {
        let marker = match &world.markers[idx] {
            Some(m) => m,
            None => continue,
        };
        if marker.kind != EntityKind::Missile {
            continue;
        }
        let carrier = match world.mirv_carriers[idx] {
            Some(c) => c,
            None => continue,
        };
        let transform = match &world.transforms[idx] {
            Some(t) => *t,
            None => continue,
        };
        let velocity = match &world.velocities[idx] {
            Some(v) => *v,
            None => continue,
        };

        // Split when descending below split altitude
        if transform.y <= carrier.split_altitude && velocity.vy < 0.0 {
            to_split.push((idx, transform.x, transform.y, velocity.vx, velocity.vy, carrier));
        }
    }

    // Process splits
    for (carrier_idx, x, y, vx, vy, carrier) in to_split {
        // Despawn the carrier
        if let Some(generation) = world.allocator.generation_of(carrier_idx as u32) {
            let eid = EntityId::new(carrier_idx as u32, generation);
            world.despawn(eid);
        }

        // Calculate base direction from carrier velocity
        let speed = (vx * vx + vy * vy).sqrt().max(1.0);
        let base_angle = vy.atan2(vx);

        // Spawn child warheads in a fan pattern
        let child_count = carrier.child_count.max(1);
        let half_spread = carrier.spread_angle / 2.0;
        for i in 0..child_count {
            let angle_offset = if child_count > 1 {
                -half_spread + carrier.spread_angle * (i as f32 / (child_count - 1) as f32)
            } else {
                0.0
            };
            let child_angle = base_angle + angle_offset;
            let child_vx = child_angle.cos() * speed;
            let child_vy = child_angle.sin() * speed;

            let child_id = world.spawn();
            let cidx = child_id.index as usize;

            world.transforms[cidx] = Some(Transform {
                x,
                y,
                rotation: child_angle,
            });
            world.velocities[cidx] = Some(Velocity {
                vx: child_vx,
                vy: child_vy,
            });
            world.ballistics[cidx] = Some(Ballistic {
                drag_coefficient: config::MISSILE_DRAG_COEFF,
                mass: config::MISSILE_MASS,
                cross_section: config::MISSILE_CROSS_SECTION,
            });
            world.warheads[cidx] = Some(Warhead {
                yield_force: config::MIRV_CHILD_YIELD,
                blast_radius_base: config::MIRV_CHILD_BLAST_RADIUS,
                warhead_type: WarheadType::Standard,
            });
            world.markers[cidx] = Some(EntityMarker {
                kind: EntityKind::Missile,
            });
            world.reentry_glows[cidx] = Some(ReentryGlow {
                intensity: 1.0,
                altitude_threshold: 200.0,
            });
        }

        result.events.push(GameEvent::MirvSplit(MirvSplitEvent {
            carrier_id: carrier_idx as u32,
            x,
            y,
            child_count,
            tick,
        }));
        result.splits += 1;
    }

    result
}
