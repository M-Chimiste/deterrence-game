use crate::ecs::components::EntityKind;
use crate::ecs::world::World;
use crate::state::snapshot::{EntityExtra, EntitySnapshot, EntityType, StateSnapshot};

/// Build a serializable StateSnapshot from the current world state.
pub fn build(world: &World, tick: u64, wave_number: u32, phase: &str) -> StateSnapshot {
    let mut entities = Vec::new();

    for idx in world.alive_entities() {
        let marker = match &world.markers[idx] {
            Some(m) => m,
            None => continue,
        };

        let transform = match &world.transforms[idx] {
            Some(t) => t,
            None => continue,
        };

        let (vx, vy) = world.velocities[idx]
            .as_ref()
            .map(|v| (v.vx, v.vy))
            .unwrap_or((0.0, 0.0));

        let entity_type = match marker.kind {
            EntityKind::Missile => EntityType::Missile,
            EntityKind::Interceptor => EntityType::Interceptor,
            EntityKind::Shockwave => EntityType::Shockwave,
            EntityKind::City => EntityType::City,
            EntityKind::Battery => EntityType::Battery,
        };

        let extra = match marker.kind {
            EntityKind::Shockwave => world.shockwaves[idx].as_ref().map(|s| EntityExtra::Shockwave {
                radius: s.radius,
                max_radius: s.max_radius,
            }),
            EntityKind::City => world.healths[idx].as_ref().map(|h| EntityExtra::City {
                health: h.current,
                max_health: h.max,
            }),
            EntityKind::Battery => {
                world.battery_states[idx].as_ref().map(|b| EntityExtra::Battery {
                    ammo: b.ammo,
                    max_ammo: b.max_ammo,
                })
            }
            EntityKind::Interceptor => {
                world.interceptors[idx].as_ref().map(|i| EntityExtra::Interceptor {
                    burn_remaining: i.burn_remaining,
                    burn_time: i.burn_time,
                    interceptor_type: i.interceptor_type.as_str().to_string(),
                })
            }
            EntityKind::Missile => {
                // Always include all missiles — no radar gating
                let is_mirv = world.mirv_carriers[idx].is_some();
                Some(EntityExtra::Missile {
                    is_mirv,
                    detected_by_radar: true,
                    detected_by_glow: false,
                })
            }
        };

        entities.push(EntitySnapshot {
            id: idx as u32,
            entity_type,
            x: transform.x,
            y: transform.y,
            rotation: transform.rotation,
            vx,
            vy,
            extra,
        });
    }

    StateSnapshot {
        tick,
        wave_number,
        phase: phase.to_string(),
        entities,
        weather: None,
        wind_x: None,
    }
}
