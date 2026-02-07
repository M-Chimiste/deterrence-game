use crate::ecs::components::EntityKind;
use crate::ecs::world::World;
use crate::engine::config;
use crate::state::weather::WeatherState;

/// Apply wind as lateral acceleration to missiles and interceptors.
/// Wind effect scales with altitude — stronger at higher altitudes.
pub fn run(world: &mut World, weather: &WeatherState) {
    if weather.wind_x == 0.0 && weather.wind_y == 0.0 {
        return;
    }

    for idx in world.alive_entities() {
        let marker = match &world.markers[idx] {
            Some(m) => m,
            None => continue,
        };

        // Only affect missiles and interceptors
        if marker.kind != EntityKind::Missile && marker.kind != EntityKind::Interceptor {
            continue;
        }

        let y = match &world.transforms[idx] {
            Some(t) => t.y,
            None => continue,
        };

        let vel = match &mut world.velocities[idx] {
            Some(v) => v,
            None => continue,
        };

        // Wind effect scales with altitude
        let altitude = (y - config::GROUND_Y).max(0.0);
        let altitude_factor = altitude * config::WIND_ALTITUDE_FACTOR;

        vel.vx += weather.wind_x * altitude_factor * config::DT;
        vel.vy += weather.wind_y * altitude_factor * config::DT;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::components::*;
    use crate::state::weather::WeatherCondition;

    fn setup_entity(world: &mut World, kind: EntityKind, x: f32, y: f32, vx: f32, vy: f32) -> usize {
        let id = world.spawn();
        let idx = id.index as usize;
        world.transforms[idx] = Some(Transform { x, y, rotation: 0.0 });
        world.markers[idx] = Some(EntityMarker { kind });
        world.velocities[idx] = Some(Velocity { vx, vy });
        idx
    }

    #[test]
    fn wind_zero_no_effect() {
        let mut world = World::new();
        let idx = setup_entity(&mut world, EntityKind::Missile, 400.0, 300.0, 0.0, -50.0);

        let weather = WeatherState::default(); // Clear, no wind
        run(&mut world, &weather);

        let vel = world.velocities[idx].as_ref().unwrap();
        assert_eq!(vel.vx, 0.0, "Zero wind should not change vx");
    }

    #[test]
    fn wind_applies_lateral_force() {
        let mut world = World::new();
        let idx = setup_entity(&mut world, EntityKind::Missile, 400.0, 300.0, 0.0, -50.0);

        let weather = WeatherState {
            condition: WeatherCondition::Storm,
            wind_x: 15.0,
            wind_y: 0.0,
        };
        run(&mut world, &weather);

        let vel = world.velocities[idx].as_ref().unwrap();
        assert!(vel.vx > 0.0, "Positive wind should increase vx, got {}", vel.vx);
    }

    #[test]
    fn wind_altitude_scaling() {
        let mut world = World::new();
        let low_idx = setup_entity(&mut world, EntityKind::Missile, 400.0, 100.0, 0.0, -50.0);
        let high_idx = setup_entity(&mut world, EntityKind::Missile, 400.0, 500.0, 0.0, -50.0);

        let weather = WeatherState {
            condition: WeatherCondition::Storm,
            wind_x: 15.0,
            wind_y: 0.0,
        };
        run(&mut world, &weather);

        let low_vx = world.velocities[low_idx].as_ref().unwrap().vx;
        let high_vx = world.velocities[high_idx].as_ref().unwrap().vx;
        assert!(
            high_vx > low_vx,
            "Higher altitude ({high_vx}) should get more wind than lower ({low_vx})"
        );
    }

    #[test]
    fn wind_does_not_affect_cities() {
        let mut world = World::new();
        let idx = setup_entity(&mut world, EntityKind::City, 400.0, 50.0, 0.0, 0.0);

        let weather = WeatherState {
            condition: WeatherCondition::Severe,
            wind_x: 30.0,
            wind_y: 0.0,
        };
        run(&mut world, &weather);

        let vel = world.velocities[idx].as_ref().unwrap();
        assert_eq!(vel.vx, 0.0, "Wind should not affect cities");
    }
}
