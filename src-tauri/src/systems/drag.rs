use crate::ecs::components::EntityKind;
use crate::ecs::world::World;
use crate::engine::config;

/// Compute air density at a given altitude using exponential falloff.
/// density = sea_level_density * exp(-altitude / scale_height)
fn air_density(altitude: f32) -> f32 {
    let h = (altitude - config::GROUND_Y).max(0.0);
    config::AIR_DENSITY_SEA_LEVEL * (-h / config::ATMOSPHERE_SCALE_HEIGHT).exp()
}

/// Apply altitude-dependent atmospheric drag to ballistic entities.
/// Drag force: F = 0.5 * rho * v^2 * Cd * A
/// Drag acceleration: a = F / m = 0.5 * rho * v^2 * Cd * A / m
pub fn run(world: &mut World) {
    for idx in world.alive_entities() {
        let dominated_by_drag = match &world.markers[idx] {
            Some(m) => matches!(m.kind, EntityKind::Missile | EntityKind::Interceptor),
            None => false,
        };

        if !dominated_by_drag {
            continue;
        }

        let (cd, mass, cross_section) = match world.ballistics[idx] {
            Some(b) => (b.drag_coefficient, b.mass, b.cross_section),
            None => continue,
        };

        let altitude = match world.transforms[idx] {
            Some(t) => t.y,
            None => continue,
        };

        if let Some(ref mut vel) = world.velocities[idx] {
            let speed_sq = vel.vx * vel.vx + vel.vy * vel.vy;
            let speed = speed_sq.sqrt();
            if speed < 1e-6 {
                continue;
            }

            let rho = air_density(altitude);
            let drag_accel = 0.5 * rho * speed_sq * cd * cross_section / mass;
            let drag_factor = (drag_accel * config::DT / speed).min(0.99);

            vel.vx -= vel.vx * drag_factor;
            vel.vy -= vel.vy * drag_factor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_decreases_with_altitude() {
        let d_low = air_density(config::GROUND_Y + 10.0);
        let d_high = air_density(config::GROUND_Y + 500.0);
        assert!(d_low > d_high);
    }

    #[test]
    fn density_at_ground_is_sea_level() {
        let d = air_density(config::GROUND_Y);
        assert!((d - config::AIR_DENSITY_SEA_LEVEL).abs() < 0.01);
    }
}
