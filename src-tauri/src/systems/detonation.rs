use crate::ecs::components::*;
use crate::ecs::entity::EntityId;
use crate::ecs::world::World;
use crate::engine::config;
use crate::events::game_events::{DetonationEvent, GameEvent, ImpactEvent};

pub struct DetonationResult {
    pub events: Vec<GameEvent>,
    pub missiles_impacted: u32,
}

/// Check for interceptor target arrival and missile ground impact.
/// Creates shockwave entities at detonation points, despawns detonated entities.
pub fn run(world: &mut World, tick: u64) -> DetonationResult {
    let mut result = DetonationResult {
        events: Vec::new(),
        missiles_impacted: 0,
    };

    let mut to_detonate: Vec<(usize, f32, f32, f32, f32, bool, bool)> = Vec::new();
    // (entity_idx, det_x, det_y, yield_force, blast_radius, is_ground_impact, is_area_denial)

    for idx in world.alive_entities() {
        let marker = match &world.markers[idx] {
            Some(m) => m,
            None => continue,
        };
        let transform = match &world.transforms[idx] {
            Some(t) => *t,
            None => continue,
        };

        match marker.kind {
            EntityKind::Interceptor => {
                let interceptor = match &world.interceptors[idx] {
                    Some(i) => i,
                    None => continue,
                };

                let dx = transform.x - interceptor.target_x;
                let dy = transform.y - interceptor.target_y;
                let dist_sq = dx * dx + dy * dy;
                let proximity = config::INTERCEPTOR_DETONATION_PROXIMITY;

                let mut should_detonate = dist_sq < proximity * proximity;

                // Proximity fuse: auto-detonate when near any enemy missile
                if !should_detonate && interceptor.proximity_fuse_radius > 0.0 {
                    let fuse_sq = interceptor.proximity_fuse_radius * interceptor.proximity_fuse_radius;
                    for &midx in world.alive_entities().iter() {
                        if let Some(m) = &world.markers[midx]
                            && m.kind == EntityKind::Missile
                            && let Some(mt) = &world.transforms[midx]
                        {
                            let mx = transform.x - mt.x;
                            let my = transform.y - mt.y;
                            if mx * mx + my * my < fuse_sq {
                                should_detonate = true;
                                break;
                            }
                        }
                    }
                }

                // If post-burn, check if moving away from target (overshoot)
                if !should_detonate
                    && interceptor.burn_remaining <= 0.0
                    && let Some(vel) = &world.velocities[idx]
                {
                    let to_target_x = interceptor.target_x - transform.x;
                    let to_target_y = interceptor.target_y - transform.y;
                    let dot = vel.vx * to_target_x + vel.vy * to_target_y;
                    if dot < 0.0 {
                        should_detonate = true;
                    }
                }

                if should_detonate {
                    let warhead = world.warheads[idx].unwrap_or(Warhead {
                        yield_force: config::WARHEAD_YIELD,
                        blast_radius_base: config::WARHEAD_BLAST_RADIUS,
                        warhead_type: WarheadType::Standard,
                    });
                    let is_area_denial = interceptor.interceptor_type
                        == InterceptorType::AreaDenial;
                    to_detonate.push((
                        idx,
                        transform.x,
                        transform.y,
                        warhead.yield_force,
                        warhead.blast_radius_base,
                        false,
                        is_area_denial,
                    ));
                }
            }
            EntityKind::Missile => {
                // Missile hits ground
                if transform.y <= config::GROUND_Y {
                    let warhead = world.warheads[idx].unwrap_or(Warhead {
                        yield_force: config::WARHEAD_YIELD,
                        blast_radius_base: config::WARHEAD_BLAST_RADIUS,
                        warhead_type: WarheadType::Standard,
                    });
                    to_detonate.push((
                        idx,
                        transform.x,
                        config::GROUND_Y,
                        warhead.yield_force,
                        warhead.blast_radius_base,
                        true,
                        false,
                    ));
                }
            }
            _ => {}
        }
    }

    // Process detonations: despawn entity, spawn shockwave, emit event
    for (idx, det_x, det_y, yield_force, blast_radius, is_ground_impact, is_area_denial) in
        to_detonate
    {
        // Despawn the detonated entity
        if let Some(generation) = world.allocator.generation_of(idx as u32) {
            let eid = EntityId::new(idx as u32, generation);
            world.despawn(eid);
        }

        // Spawn shockwave entity
        let sw_id = world.spawn();
        let sw_idx = sw_id.index as usize;
        world.transforms[sw_idx] = Some(Transform {
            x: det_x,
            y: det_y,
            rotation: 0.0,
        });
        let (expansion_rate, lifetime_ticks) = if is_area_denial {
            (config::AREA_DENIAL_EXPANSION_RATE, config::AREA_DENIAL_LINGER_TICKS)
        } else {
            (config::SHOCKWAVE_EXPANSION_RATE, config::SHOCKWAVE_LIFETIME_TICKS)
        };
        world.shockwaves[sw_idx] = Some(Shockwave {
            radius: 0.0,
            max_radius: blast_radius,
            force: yield_force,
            expansion_rate,
            damage_applied: false,
        });
        world.markers[sw_idx] = Some(EntityMarker {
            kind: EntityKind::Shockwave,
        });
        world.lifetimes[sw_idx] = Some(Lifetime {
            remaining_ticks: lifetime_ticks,
        });

        // Emit event
        if is_ground_impact {
            result.missiles_impacted += 1;
            result.events.push(GameEvent::Impact(ImpactEvent {
                entity_id: idx as u32,
                x: det_x,
                y: det_y,
                tick,
            }));
        } else {
            result.events.push(GameEvent::Detonation(DetonationEvent {
                entity_id: idx as u32,
                x: det_x,
                y: det_y,
                yield_force,
                tick,
            }));
        }
    }

    result
}
