use crate::ecs::components::*;
use crate::ecs::entity::EntityId;
use crate::ecs::world::World;
use crate::engine::config;
use crate::events::game_events::{DetonationEvent, GameEvent};

pub struct CollisionResult {
    pub events: Vec<GameEvent>,
    pub missiles_destroyed: u32,
    pub interceptors_destroyed: u32,
}

/// Check shockwave proximity against all destructible entities (missiles + interceptors).
/// Two zones:
///   - Destroy zone (dist < radius * DESTROY_RATIO): entity destroyed.
///     Missiles trigger chain reaction shockwaves. Interceptors do not.
///   - Deflect zone (DESTROY_RATIO * radius <= dist < radius): push entity velocity
///     away from shockwave center.
pub fn run(world: &mut World, tick: u64) -> CollisionResult {
    let mut result = CollisionResult {
        events: Vec::new(),
        missiles_destroyed: 0,
        interceptors_destroyed: 0,
    };

    // Gather active shockwave data: (idx, x, y, radius, force)
    let shockwaves: Vec<(usize, f32, f32, f32, f32)> = world
        .alive_entities()
        .iter()
        .filter_map(|&idx| {
            let marker = world.markers[idx].as_ref()?;
            if marker.kind != EntityKind::Shockwave {
                return None;
            }
            let t = world.transforms[idx].as_ref()?;
            let sw = world.shockwaves[idx].as_ref()?;
            Some((idx, t.x, t.y, sw.radius, sw.force))
        })
        .collect();

    // Gather all destructible entities: missiles and interceptors
    // Store: (idx, x, y, kind)
    let targets: Vec<(usize, f32, f32, EntityKind)> = world
        .alive_entities()
        .iter()
        .filter_map(|&idx| {
            let marker = world.markers[idx].as_ref()?;
            if marker.kind != EntityKind::Missile && marker.kind != EntityKind::Interceptor {
                return None;
            }
            let t = world.transforms[idx].as_ref()?;
            Some((idx, t.x, t.y, marker.kind))
        })
        .collect();

    // Determine destroy vs deflect for each target
    let mut to_destroy: Vec<(usize, f32, f32, EntityKind)> = Vec::new();
    let mut to_deflect: Vec<(usize, f32, f32)> = Vec::new(); // (idx, push_x, push_y)

    let destroy_ratio = config::SHOCKWAVE_DESTROY_RATIO;

    for &(_sw_idx, sw_x, sw_y, sw_radius, sw_force) in &shockwaves {
        if sw_radius <= 0.0 {
            continue;
        }
        let destroy_radius = sw_radius * destroy_ratio;

        for &(tgt_idx, tgt_x, tgt_y, kind) in &targets {
            let dx = tgt_x - sw_x;
            let dy = tgt_y - sw_y;
            let dist_sq = dx * dx + dy * dy;
            let dist = dist_sq.sqrt();

            if dist < destroy_radius {
                // Inner destroy zone
                to_destroy.push((tgt_idx, tgt_x, tgt_y, kind));
            } else if dist < sw_radius {
                // Outer deflect zone — push away from shockwave center
                let norm = dist.max(0.01); // prevent div by zero
                let push_x = dx / norm;
                let push_y = dy / norm;
                let force_scale = sw_force * (1.0 - dist / sw_radius)
                    * config::SHOCKWAVE_DEFLECT_FORCE
                    * config::DT;
                to_deflect.push((tgt_idx, push_x * force_scale, push_y * force_scale));
            }
        }
    }

    // Deduplicate destroys (entity in range of multiple shockwaves)
    to_destroy.sort_by_key(|&(idx, _, _, _)| idx);
    to_destroy.dedup_by_key(|entry| entry.0);

    // Aggregate deflection pushes per entity (may be pushed by multiple shockwaves)
    to_deflect.sort_by_key(|&(idx, _, _)| idx);
    let mut aggregated_deflect: Vec<(usize, f32, f32)> = Vec::new();
    for (idx, px, py) in to_deflect {
        if let Some(last) = aggregated_deflect.last_mut()
            && last.0 == idx
        {
            last.1 += px;
            last.2 += py;
            continue;
        }
        aggregated_deflect.push((idx, px, py));
    }

    // Remove entities that are being destroyed from the deflect list
    let destroy_set: Vec<usize> = to_destroy.iter().map(|d| d.0).collect();
    aggregated_deflect.retain(|&(idx, _, _)| !destroy_set.contains(&idx));

    // Apply deflections to entity velocities
    for (tgt_idx, push_x, push_y) in aggregated_deflect {
        if let Some(vel) = &mut world.velocities[tgt_idx] {
            vel.vx += push_x;
            vel.vy += push_y;
        }
    }

    // Destroy entities and spawn chain reaction shockwaves (missiles only)
    let chain_mult = config::CHAIN_REACTION_MULTIPLIER;

    for (tgt_idx, tgt_x, tgt_y, kind) in to_destroy {
        let warhead = world.warheads[tgt_idx];

        // Despawn the entity
        if let Some(generation) = world.allocator.generation_of(tgt_idx as u32) {
            let eid = EntityId::new(tgt_idx as u32, generation);
            world.despawn(eid);
        }

        match kind {
            EntityKind::Missile => {
                result.missiles_destroyed += 1;

                // Chain reaction: missiles trigger new shockwaves
                if let Some(wh) = warhead {
                    let sw_id = world.spawn();
                    let sw_idx = sw_id.index as usize;
                    world.transforms[sw_idx] = Some(Transform {
                        x: tgt_x,
                        y: tgt_y,
                        rotation: 0.0,
                    });
                    world.shockwaves[sw_idx] = Some(Shockwave {
                        radius: 0.0,
                        max_radius: wh.blast_radius_base * chain_mult,
                        force: wh.yield_force * chain_mult,
                        expansion_rate: config::SHOCKWAVE_EXPANSION_RATE,
                        damage_applied: false,
                    });
                    world.markers[sw_idx] = Some(EntityMarker {
                        kind: EntityKind::Shockwave,
                    });
                    world.lifetimes[sw_idx] = Some(Lifetime {
                        remaining_ticks: config::SHOCKWAVE_LIFETIME_TICKS,
                    });

                    result.events.push(GameEvent::Detonation(DetonationEvent {
                        entity_id: tgt_idx as u32,
                        x: tgt_x,
                        y: tgt_y,
                        yield_force: wh.yield_force,
                        tick,
                    }));
                }
            }
            EntityKind::Interceptor => {
                result.interceptors_destroyed += 1;
                // Interceptors do NOT trigger chain reactions — just destroyed
            }
            _ => {}
        }
    }

    result
}
