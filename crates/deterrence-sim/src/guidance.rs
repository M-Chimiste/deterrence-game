//! Guidance algorithms for missile interceptors.
//!
//! Provides proportional navigation (PN), time-to-intercept estimation,
//! lead predicted intercept point (PIP) calculation, and engagement envelope checks.

use deterrence_core::constants::*;
use deterrence_core::enums::WeaponType;
use deterrence_core::types::{Position, Velocity};

/// Apply True Proportional Navigation (TPN) guidance to compute new missile velocity.
///
/// The acceleration command is proportional to closing velocity and LOS angular rate,
/// applied perpendicular to the line-of-sight. The resulting velocity maintains
/// constant speed (heading change only), clamped by `PN_MAX_TURN_RATE`.
///
/// Falls back to pure pursuit if closing velocity is too low.
pub fn proportional_navigation(
    missile_pos: &Position,
    missile_vel: &Velocity,
    target_pos: &Position,
    target_vel: &Velocity,
    missile_speed: f64,
    dt: f64,
) -> Velocity {
    // LOS vector from missile to target
    let los_x = target_pos.x - missile_pos.x;
    let los_y = target_pos.y - missile_pos.y;
    let los_z = target_pos.z - missile_pos.z;
    let range_sq = los_x * los_x + los_y * los_y + los_z * los_z;
    let range = range_sq.sqrt();

    if range < 1.0 {
        return *missile_vel;
    }

    // LOS unit vector
    let los_ux = los_x / range;
    let los_uy = los_y / range;
    let los_uz = los_z / range;

    // Relative velocity (target w.r.t. missile)
    let vrel_x = target_vel.x - missile_vel.x;
    let vrel_y = target_vel.y - missile_vel.y;
    let vrel_z = target_vel.z - missile_vel.z;

    // Closing velocity (positive when approaching)
    let v_closing = -(vrel_x * los_ux + vrel_y * los_uy + vrel_z * los_uz);

    // If not closing, fall back to pure pursuit
    if v_closing < 10.0 {
        return pure_pursuit(missile_pos, target_pos, missile_speed);
    }

    // LOS angular rate: omega = (LOS × V_rel) / |R|²
    let omega_x = (los_y * vrel_z - los_z * vrel_y) / range_sq;
    let omega_y = (los_z * vrel_x - los_x * vrel_z) / range_sq;
    let omega_z = (los_x * vrel_y - los_y * vrel_x) / range_sq;

    // TPN acceleration: a = N * Vc * (omega × LOS_hat)
    // This gives acceleration perpendicular to LOS in the engagement plane
    let n = PN_NAVIGATION_CONSTANT;
    let acmd_x = omega_y * los_uz - omega_z * los_uy;
    let acmd_y = omega_z * los_ux - omega_x * los_uz;
    let acmd_z = omega_x * los_uy - omega_y * los_ux;

    let accel_x = n * v_closing * acmd_x;
    let accel_y = n * v_closing * acmd_y;
    let accel_z = n * v_closing * acmd_z;

    // Apply acceleration to current velocity
    let new_vx = missile_vel.x + accel_x * dt;
    let new_vy = missile_vel.y + accel_y * dt;
    let new_vz = missile_vel.z + accel_z * dt;

    // Clamp turn rate
    let current_speed = missile_vel.speed();
    if current_speed < 1.0 {
        return pure_pursuit(missile_pos, target_pos, missile_speed);
    }

    let new_speed = (new_vx * new_vx + new_vy * new_vy + new_vz * new_vz).sqrt();
    if new_speed < 1.0 {
        return pure_pursuit(missile_pos, target_pos, missile_speed);
    }

    // Angle between old and new velocity direction
    let dot = (missile_vel.x * new_vx + missile_vel.y * new_vy + missile_vel.z * new_vz)
        / (current_speed * new_speed);
    let angle = dot.clamp(-1.0, 1.0).acos();
    let max_angle = PN_MAX_TURN_RATE * dt;

    if angle > max_angle && angle > 1e-6 {
        // Limit turn: interpolate between old and new direction
        let t = max_angle / angle;
        let lim_vx = missile_vel.x + (new_vx - missile_vel.x) * t;
        let lim_vy = missile_vel.y + (new_vy - missile_vel.y) * t;
        let lim_vz = missile_vel.z + (new_vz - missile_vel.z) * t;
        let lim_speed = (lim_vx * lim_vx + lim_vy * lim_vy + lim_vz * lim_vz).sqrt();
        let s = missile_speed / lim_speed;
        Velocity::new(lim_vx * s, lim_vy * s, lim_vz * s)
    } else {
        // Within turn rate limit — normalize to maintain constant speed
        let s = missile_speed / new_speed;
        Velocity::new(new_vx * s, new_vy * s, new_vz * s)
    }
}

/// Pure pursuit fallback: velocity pointing directly at target at constant speed.
fn pure_pursuit(from: &Position, to: &Position, speed: f64) -> Velocity {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dz = to.z - from.z;
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    if dist > 1.0 {
        Velocity::new(speed * dx / dist, speed * dy / dist, speed * dz / dist)
    } else {
        Velocity::new(0.0, speed, 0.0)
    }
}

/// Estimate time-to-intercept using closing velocity.
///
/// Uses the component of relative velocity along the line-of-sight.
/// Falls back to combined speed if objects aren't closing.
pub fn estimate_tti(pos_a: &Position, vel_a: &Velocity, pos_b: &Position, vel_b: &Velocity) -> f64 {
    let range = pos_a.range_to(pos_b);
    if range < 1.0 {
        return 0.0;
    }

    let dx = pos_b.x - pos_a.x;
    let dy = pos_b.y - pos_a.y;
    let dz = pos_b.z - pos_a.z;
    let los_ux = dx / range;
    let los_uy = dy / range;
    let los_uz = dz / range;

    // Closing velocity: component of A's velocity toward B minus B's away from A
    let v_closing =
        (vel_a.x - vel_b.x) * los_ux + (vel_a.y - vel_b.y) * los_uy + (vel_a.z - vel_b.z) * los_uz;

    if v_closing > 1.0 {
        range / v_closing
    } else {
        // Not closing — use combined speeds as rough estimate
        let speed = vel_a.speed() + vel_b.speed();
        if speed > 1.0 {
            range / speed
        } else {
            f64::MAX
        }
    }
}

/// Calculate lead predicted intercept point using iterative prediction.
///
/// Returns (PIP position, estimated TTI). Uses 2 iterations to refine
/// the prediction accounting for target motion during missile flight.
pub fn calculate_lead_pip(
    target_pos: &Position,
    target_vel: &Velocity,
    own_pos: &Position,
    missile_speed: f64,
) -> (Position, f64) {
    let mut tti = own_pos.range_to(target_pos) / missile_speed;

    for _ in 0..2 {
        let pred = Position::new(
            target_pos.x + target_vel.x * tti,
            target_pos.y + target_vel.y * tti,
            target_pos.z + target_vel.z * tti,
        );
        let range = own_pos.range_to(&pred);
        tti = range / missile_speed;
    }

    let pip = Position::new(
        target_pos.x + target_vel.x * tti,
        target_pos.y + target_vel.y * tti,
        target_pos.z + target_vel.z * tti,
    );

    (pip, tti)
}

/// Check if a target range is within the engagement envelope for a weapon type.
pub fn in_engagement_envelope(range: f64, weapon_type: WeaponType) -> bool {
    let max_range = match weapon_type {
        WeaponType::Standard => STANDARD_MISSILE_MAX_RANGE,
        WeaponType::ExtendedRange => EXTENDED_RANGE_MISSILE_MAX_RANGE,
        WeaponType::PointDefense => POINT_DEFENSE_MISSILE_MAX_RANGE,
    };
    let min_range = 1000.0;
    range >= min_range && range <= max_range
}

/// Get missile speed for a given weapon type.
pub fn missile_speed_for_type(weapon_type: WeaponType) -> f64 {
    match weapon_type {
        WeaponType::Standard => STANDARD_MISSILE_SPEED,
        WeaponType::ExtendedRange => EXTENDED_RANGE_MISSILE_SPEED,
        WeaponType::PointDefense => POINT_DEFENSE_MISSILE_SPEED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pn_intercepts_head_on_target() {
        // Missile heading north, target heading south (head-on)
        let mut m_pos = Position::new(0.0, 0.0, 100.0);
        let mut m_vel = Velocity::new(0.0, STANDARD_MISSILE_SPEED, 50.0);
        let mut t_pos = Position::new(0.0, 80_000.0, 5_000.0);
        let t_vel = Velocity::new(0.0, -SEA_SKIMMER_SPEED, 0.0);
        let speed = STANDARD_MISSILE_SPEED;
        let dt = DT;

        let mut min_range = f64::MAX;

        for _ in 0..5000 {
            let range = m_pos.range_to(&t_pos);
            if range < min_range {
                min_range = range;
            }
            if range < 200.0 {
                break;
            }

            m_vel = proportional_navigation(&m_pos, &m_vel, &t_pos, &t_vel, speed, dt);

            m_pos.x += m_vel.x * dt;
            m_pos.y += m_vel.y * dt;
            m_pos.z += m_vel.z * dt;
            t_pos.x += t_vel.x * dt;
            t_pos.y += t_vel.y * dt;
            t_pos.z += t_vel.z * dt;
        }

        assert!(
            min_range < 200.0,
            "PN should converge on head-on target, min range: {min_range:.1}m"
        );
    }

    #[test]
    fn test_pn_intercepts_crossing_target() {
        // Missile heading north, target crossing east-to-west
        let mut m_pos = Position::new(0.0, 0.0, 100.0);
        let mut m_vel = Velocity::new(0.0, STANDARD_MISSILE_SPEED, 0.0);
        let mut t_pos = Position::new(30_000.0, 60_000.0, 5_000.0);
        let t_vel = Velocity::new(-290.0, 0.0, 0.0);
        let speed = STANDARD_MISSILE_SPEED;
        let dt = DT;

        let mut min_range = f64::MAX;

        for _ in 0..5000 {
            let range = m_pos.range_to(&t_pos);
            if range < min_range {
                min_range = range;
            }
            if range < 200.0 {
                break;
            }

            m_vel = proportional_navigation(&m_pos, &m_vel, &t_pos, &t_vel, speed, dt);

            m_pos.x += m_vel.x * dt;
            m_pos.y += m_vel.y * dt;
            m_pos.z += m_vel.z * dt;
            t_pos.x += t_vel.x * dt;
            t_pos.y += t_vel.y * dt;
            t_pos.z += t_vel.z * dt;
        }

        assert!(
            min_range < 200.0,
            "PN should converge on crossing target, min range: {min_range:.1}m"
        );
    }

    #[test]
    fn test_pn_intercepts_maneuvering_target() {
        // Missile heading north, target weaving (sinusoidal x-velocity)
        let mut m_pos = Position::new(0.0, 0.0, 100.0);
        let mut m_vel = Velocity::new(0.0, STANDARD_MISSILE_SPEED, 50.0);
        let mut t_pos = Position::new(10_000.0, 70_000.0, 5_000.0);
        let speed = STANDARD_MISSILE_SPEED;
        let dt = DT;

        let mut min_range = f64::MAX;

        for step in 0..5000 {
            let range = m_pos.range_to(&t_pos);
            if range < min_range {
                min_range = range;
            }
            if range < 200.0 {
                break;
            }

            // Target weaves: heading south with sinusoidal east-west component
            let time = step as f64 * dt;
            let t_vel = Velocity::new(
                200.0 * (time * 0.5).sin(), // weave at ~0.08 Hz
                -SEA_SKIMMER_SPEED,
                0.0,
            );

            m_vel = proportional_navigation(&m_pos, &m_vel, &t_pos, &t_vel, speed, dt);

            m_pos.x += m_vel.x * dt;
            m_pos.y += m_vel.y * dt;
            m_pos.z += m_vel.z * dt;
            t_pos.x += t_vel.x * dt;
            t_pos.y += t_vel.y * dt;
            t_pos.z += t_vel.z * dt;
        }

        assert!(
            min_range < 300.0,
            "PN should converge on maneuvering target, min range: {min_range:.1}m"
        );
    }

    #[test]
    fn test_tti_estimate_accuracy() {
        // Head-on: missile at origin heading north, target 80km north heading south
        let m_pos = Position::new(0.0, 0.0, 0.0);
        let m_vel = Velocity::new(0.0, STANDARD_MISSILE_SPEED, 0.0);
        let t_pos = Position::new(0.0, 80_000.0, 0.0);
        let t_vel = Velocity::new(0.0, -SEA_SKIMMER_SPEED, 0.0);

        let tti = estimate_tti(&m_pos, &m_vel, &t_pos, &t_vel);

        // Actual TTI = range / (missile_speed + target_speed) = 80000 / 1490 ≈ 53.7s
        let expected = 80_000.0 / (STANDARD_MISSILE_SPEED + SEA_SKIMMER_SPEED);
        let error = (tti - expected).abs() / expected;
        assert!(
            error < 0.05,
            "TTI estimate should be within 5% of actual: expected {expected:.1}s, got {tti:.1}s"
        );
    }

    #[test]
    fn test_engagement_envelope_check() {
        // Standard missile: max range 167km
        assert!(in_engagement_envelope(100_000.0, WeaponType::Standard));
        assert!(!in_engagement_envelope(200_000.0, WeaponType::Standard));
        assert!(!in_engagement_envelope(500.0, WeaponType::Standard)); // too close

        // ER missile: max range 250km
        assert!(in_engagement_envelope(200_000.0, WeaponType::ExtendedRange));
        assert!(!in_engagement_envelope(
            300_000.0,
            WeaponType::ExtendedRange
        ));

        // Point defense: max range 30km
        assert!(in_engagement_envelope(15_000.0, WeaponType::PointDefense));
        assert!(!in_engagement_envelope(35_000.0, WeaponType::PointDefense));
    }

    #[test]
    fn test_lead_pip_ahead_of_target() {
        // Target at 50km north heading east at 290 m/s
        let t_pos = Position::new(0.0, 50_000.0, 5_000.0);
        let t_vel = Velocity::new(290.0, 0.0, 0.0);
        let own_pos = Position::new(0.0, 0.0, 0.0);

        let (pip, tti) = calculate_lead_pip(&t_pos, &t_vel, &own_pos, STANDARD_MISSILE_SPEED);

        // PIP should be east of target's current position (leading it)
        assert!(
            pip.x > t_pos.x,
            "PIP should be east of target: pip.x={:.0}, target.x={:.0}",
            pip.x,
            t_pos.x
        );
        assert!(tti > 0.0, "TTI should be positive");
        // PIP should be roughly at target_pos + target_vel * tti
        let expected_x = t_pos.x + t_vel.x * tti;
        assert!(
            (pip.x - expected_x).abs() < 100.0,
            "PIP x should match predicted position"
        );
    }
}
