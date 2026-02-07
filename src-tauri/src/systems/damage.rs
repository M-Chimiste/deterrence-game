use crate::ecs::components::EntityKind;
use crate::ecs::entity::EntityId;
use crate::ecs::world::World;
use crate::engine::config;
use crate::events::game_events::{CityDamagedEvent, GameEvent};

/// Check newly-created ground-level shockwaves against cities.
/// Applies damage once per shockwave using the damage_applied flag.
pub fn run(world: &mut World, city_ids: &[EntityId], tick: u64) -> Vec<GameEvent> {
    let mut events = Vec::new();

    // Gather shockwaves that haven't applied damage yet and are near ground level
    let ground_shockwaves: Vec<(usize, f32, f32, f32, f32)> = world
        .alive_entities()
        .iter()
        .filter_map(|&idx| {
            let marker = world.markers[idx].as_ref()?;
            if marker.kind != EntityKind::Shockwave {
                return None;
            }
            let sw = world.shockwaves[idx].as_ref()?;
            if sw.damage_applied {
                return None;
            }
            let t = world.transforms[idx].as_ref()?;
            // Only ground-level shockwaves can damage cities
            if t.y > config::GROUND_Y + config::GROUND_IMPACT_DAMAGE_RADIUS {
                return None;
            }
            Some((idx, t.x, t.y, sw.max_radius, sw.force))
        })
        .collect();

    // Gather city data
    let cities: Vec<(usize, u32, f32)> = city_ids
        .iter()
        .enumerate()
        .filter_map(|(city_idx, &eid)| {
            if !world.is_alive(eid) {
                return None;
            }
            let idx = eid.index as usize;
            let t = world.transforms[idx].as_ref()?;
            let h = world.healths[idx].as_ref()?;
            if h.current <= 0.0 {
                return None;
            }
            Some((idx, city_idx as u32, t.x))
        })
        .collect();

    // Check each ground shockwave against each city
    for &(sw_idx, sw_x, sw_y, _max_radius, _force) in &ground_shockwaves {
        let damage_radius = config::GROUND_IMPACT_DAMAGE_RADIUS;

        for &(city_world_idx, city_id, city_x) in &cities {
            let dx = city_x - sw_x;
            let dy = config::GROUND_Y - sw_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < damage_radius {
                // Damage falls off linearly with distance
                let falloff = 1.0 - (dist / damage_radius);
                let damage = config::GROUND_IMPACT_BASE_DAMAGE * falloff;

                if let Some(ref mut health) = world.healths[city_world_idx] {
                    health.current = (health.current - damage).max(0.0);
                    events.push(GameEvent::CityDamaged(CityDamagedEvent {
                        city_id,
                        damage,
                        remaining_health: health.current,
                        tick,
                    }));
                }
            }
        }

        // Mark damage as applied
        if let Some(ref mut sw) = world.shockwaves[sw_idx] {
            sw.damage_applied = true;
        }
    }

    events
}
