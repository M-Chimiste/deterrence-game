use crate::ecs::components::{Detected, EntityKind};
use crate::ecs::entity::EntityId;
use crate::ecs::world::World;
use crate::engine::config;
use crate::state::weather::{self, WeatherState};

/// Detection system: determines which missiles are visible to the player.
///
/// - **Radar**: missiles within RADAR_BASE_RANGE * weather_multiplier of any battery are radar-detected
/// - **Glow**: missiles with ReentryGlow below altitude_threshold in clear/overcast weather are glow-detected
/// - Cities, batteries, interceptors, and shockwaves are always detected
pub fn run(world: &mut World, battery_ids: &[EntityId], weather: &WeatherState) {
    let radar_range = config::RADAR_BASE_RANGE * weather::radar_multiplier(weather.condition);
    let radar_range_sq = radar_range * radar_range;
    let glow_vis = weather::glow_visibility(weather.condition);

    // Collect battery positions for distance checks
    let battery_positions: Vec<(f32, f32)> = battery_ids
        .iter()
        .filter_map(|&bid| {
            if world.is_alive(bid) {
                world.transforms[bid.index as usize].map(|t| (t.x, t.y))
            } else {
                None
            }
        })
        .collect();

    for idx in world.alive_entities() {
        let marker = match &world.markers[idx] {
            Some(m) => m,
            None => continue,
        };

        match marker.kind {
            // Cities, batteries, interceptors, shockwaves always detected
            EntityKind::City | EntityKind::Battery | EntityKind::Interceptor | EntityKind::Shockwave => {
                world.detected[idx] = Some(Detected {
                    by_radar: true,
                    by_glow: false,
                });
            }
            EntityKind::Missile => {
                let transform = match &world.transforms[idx] {
                    Some(t) => t,
                    None => continue,
                };

                // Radar check: distance to any battery within effective range
                let by_radar = battery_positions.iter().any(|&(bx, by)| {
                    let dx = transform.x - bx;
                    let dy = transform.y - by;
                    dx * dx + dy * dy <= radar_range_sq
                });

                // Glow check: has ReentryGlow, below altitude threshold, weather permits
                let by_glow = glow_vis > 0.0
                    && world.reentry_glows[idx]
                        .as_ref()
                        .is_some_and(|g| transform.y < g.altitude_threshold);

                if by_radar || by_glow {
                    world.detected[idx] = Some(Detected { by_radar, by_glow });
                } else {
                    world.detected[idx] = None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::components::*;
    use crate::state::weather::{WeatherCondition, WeatherState};

    fn clear_weather() -> WeatherState {
        WeatherState {
            condition: WeatherCondition::Clear,
            wind_x: 0.0,
            wind_y: 0.0,
        }
    }

    fn spawn_battery(world: &mut World, x: f32, y: f32) -> EntityId {
        let id = world.spawn();
        let idx = id.index as usize;
        world.transforms[idx] = Some(Transform { x, y, rotation: 0.0 });
        world.markers[idx] = Some(EntityMarker { kind: EntityKind::Battery });
        world.battery_states[idx] = Some(BatteryState { ammo: 10, max_ammo: 10 });
        id
    }

    fn spawn_missile(world: &mut World, x: f32, y: f32) -> EntityId {
        let id = world.spawn();
        let idx = id.index as usize;
        world.transforms[idx] = Some(Transform { x, y, rotation: 0.0 });
        world.velocities[idx] = Some(Velocity { vx: 0.0, vy: -50.0 });
        world.markers[idx] = Some(EntityMarker { kind: EntityKind::Missile });
        world.warheads[idx] = Some(Warhead {
            yield_force: 100.0,
            blast_radius_base: 40.0,
            warhead_type: WarheadType::Standard,
        });
        id
    }

    fn spawn_missile_with_glow(world: &mut World, x: f32, y: f32, altitude_threshold: f32) -> EntityId {
        let id = spawn_missile(world, x, y);
        let idx = id.index as usize;
        world.reentry_glows[idx] = Some(ReentryGlow {
            intensity: 1.0,
            altitude_threshold,
        });
        id
    }

    #[test]
    fn missile_within_radar_range_detected() {
        let mut world = World::new();
        let bat = spawn_battery(&mut world, 160.0, 50.0);
        // Missile at 300 units from battery (within 500 base range)
        let missile = spawn_missile(&mut world, 460.0, 50.0);

        run(&mut world, &[bat], &clear_weather());

        let det = world.detected[missile.index as usize].as_ref().unwrap();
        assert!(det.by_radar);
    }

    #[test]
    fn missile_outside_radar_range_not_detected() {
        let mut world = World::new();
        let bat = spawn_battery(&mut world, 160.0, 50.0);
        // Missile at 600 units from battery (beyond 500 base range)
        let missile = spawn_missile(&mut world, 760.0, 50.0);

        run(&mut world, &[bat], &clear_weather());

        assert!(world.detected[missile.index as usize].is_none());
    }

    #[test]
    fn glow_below_altitude_threshold_detected() {
        let mut world = World::new();
        let bat = spawn_battery(&mut world, 160.0, 50.0);
        // Missile far from battery but with glow below threshold
        let missile = spawn_missile_with_glow(&mut world, 900.0, 200.0, 300.0);

        run(&mut world, &[bat], &clear_weather());

        let det = world.detected[missile.index as usize].as_ref().unwrap();
        assert!(!det.by_radar); // too far for radar
        assert!(det.by_glow);
    }

    #[test]
    fn glow_above_altitude_threshold_not_detected() {
        let mut world = World::new();
        let bat = spawn_battery(&mut world, 160.0, 50.0);
        // Missile far from battery, above glow threshold
        let missile = spawn_missile_with_glow(&mut world, 900.0, 400.0, 300.0);

        run(&mut world, &[bat], &clear_weather());

        assert!(world.detected[missile.index as usize].is_none());
    }

    #[test]
    fn weather_degrades_radar_range() {
        let mut world = World::new();
        let bat = spawn_battery(&mut world, 160.0, 50.0);
        // Missile at 400 units — within clear range (500) but outside storm range (500*0.6=300)
        let missile = spawn_missile(&mut world, 560.0, 50.0);

        let storm = WeatherState {
            condition: WeatherCondition::Storm,
            wind_x: 10.0,
            wind_y: 0.0,
        };
        run(&mut world, &[bat], &storm);

        assert!(world.detected[missile.index as usize].is_none());
    }

    #[test]
    fn severe_blocks_glow_detection() {
        let mut world = World::new();
        let bat = spawn_battery(&mut world, 160.0, 50.0);
        // Missile with glow, below threshold, but Severe weather blocks glow
        let missile = spawn_missile_with_glow(&mut world, 900.0, 200.0, 300.0);

        let severe = WeatherState {
            condition: WeatherCondition::Severe,
            wind_x: 20.0,
            wind_y: 0.0,
        };
        run(&mut world, &[bat], &severe);

        assert!(world.detected[missile.index as usize].is_none());
    }

    #[test]
    fn cities_always_detected() {
        let mut world = World::new();
        let bat = spawn_battery(&mut world, 160.0, 50.0);
        let city = world.spawn();
        let idx = city.index as usize;
        world.transforms[idx] = Some(Transform { x: 640.0, y: 50.0, rotation: 0.0 });
        world.markers[idx] = Some(EntityMarker { kind: EntityKind::City });
        world.healths[idx] = Some(Health { current: 100.0, max: 100.0 });

        run(&mut world, &[bat], &clear_weather());

        assert!(world.detected[idx].is_some());
    }

    #[test]
    fn interceptors_always_detected() {
        let mut world = World::new();
        let bat = spawn_battery(&mut world, 160.0, 50.0);
        let interceptor = world.spawn();
        let idx = interceptor.index as usize;
        world.transforms[idx] = Some(Transform { x: 800.0, y: 400.0, rotation: 0.0 });
        world.markers[idx] = Some(EntityMarker { kind: EntityKind::Interceptor });
        world.velocities[idx] = Some(Velocity { vx: 0.0, vy: 100.0 });

        run(&mut world, &[bat], &clear_weather());

        assert!(world.detected[idx].is_some());
    }

    #[test]
    fn multiple_batteries_extend_coverage() {
        let mut world = World::new();
        let bat1 = spawn_battery(&mut world, 160.0, 50.0);
        let bat2 = spawn_battery(&mut world, 1120.0, 50.0);
        // Missile near bat2 but far from bat1
        let missile = spawn_missile(&mut world, 900.0, 50.0);

        run(&mut world, &[bat1, bat2], &clear_weather());

        let det = world.detected[missile.index as usize].as_ref().unwrap();
        assert!(det.by_radar);
    }

    #[test]
    fn undetected_missile_has_none() {
        let mut world = World::new();
        let bat = spawn_battery(&mut world, 160.0, 50.0);
        // Missile very far from battery, no glow
        let missile = spawn_missile(&mut world, 1200.0, 600.0);

        run(&mut world, &[bat], &clear_weather());

        assert!(world.detected[missile.index as usize].is_none());
    }
}
