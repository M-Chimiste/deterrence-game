use serde::{Deserialize, Serialize};

use crate::engine::config::{self, InterceptorProfile};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcPrediction {
    pub points: Vec<(f32, f32)>,
    pub time_to_target: f32,
    pub reaches_target: bool,
}

/// Predict the trajectory of an interceptor launched from (start_x, start_y)
/// aiming at (target_x, target_y). Pure physics computation — no ECS access.
///
/// Replicates the exact physics from thrust.rs, gravity.rs, drag.rs, wind.rs,
/// and detonation.rs to produce an accurate predicted path.
pub fn predict_arc(
    start_x: f32,
    start_y: f32,
    target_x: f32,
    target_y: f32,
    profile: &InterceptorProfile,
    wind_x: f32,
) -> ArcPrediction {
    let dx = target_x - start_x;
    let dy = target_y - start_y;
    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
    let dir_x = dx / dist;
    let dir_y = dy / dist;

    // Initial velocity — matches input_system.rs
    let mut x = start_x;
    let mut y = start_y;
    let mut vx = dir_x * 10.0;
    let mut vy = dir_y * 10.0;
    let mut burn_remaining = profile.burn_time;

    let mut points = Vec::with_capacity(120);
    points.push((x, y));

    let max_steps = (10.0 / config::DT) as usize; // 10 seconds max
    let mut time = 0.0_f32;
    let mut reached_target = false;

    for _ in 0..max_steps {
        // Thrust — matches thrust.rs
        if burn_remaining > 0.0 {
            let tdx = target_x - x;
            let tdy = target_y - y;
            let tdist = (tdx * tdx + tdy * tdy).sqrt();
            if tdist > 1e-6 {
                let tdir_x = tdx / tdist;
                let tdir_y = tdy / tdist;
                let thrust_accel = profile.thrust * config::DT;
                vx += tdir_x * thrust_accel;
                vy += tdir_y * thrust_accel;
            } else {
                burn_remaining = 0.0;
            }
            burn_remaining -= config::DT;
            if burn_remaining < 0.0 {
                burn_remaining = 0.0;
            }
        }

        // Gravity — matches gravity.rs
        vy -= config::GRAVITY * config::DT;

        // Drag — matches drag.rs
        let speed_sq = vx * vx + vy * vy;
        let speed = speed_sq.sqrt();
        if speed > 1e-6 {
            let h = (y - config::GROUND_Y).max(0.0);
            let rho = config::AIR_DENSITY_SEA_LEVEL * (-h / config::ATMOSPHERE_SCALE_HEIGHT).exp();
            let drag_accel = 0.5 * rho * speed_sq * profile.drag_coeff
                * profile.cross_section
                / profile.mass;
            let drag_factor = (drag_accel * config::DT / speed).min(0.99);
            vx -= vx * drag_factor;
            vy -= vy * drag_factor;
        }

        // Wind — matches wind.rs
        if wind_x != 0.0 {
            let altitude = (y - config::GROUND_Y).max(0.0);
            let altitude_factor = altitude * config::WIND_ALTITUDE_FACTOR;
            vx += wind_x * altitude_factor * config::DT;
        }

        // Movement — matches movement.rs
        x += vx * config::DT;
        y += vy * config::DT;
        time += config::DT;

        points.push((x, y));

        // Target proximity check — matches detonation.rs
        let prox_dx = x - target_x;
        let prox_dy = y - target_y;
        let prox_dist_sq = prox_dx * prox_dx + prox_dy * prox_dy;
        if prox_dist_sq
            < config::INTERCEPTOR_DETONATION_PROXIMITY * config::INTERCEPTOR_DETONATION_PROXIMITY
        {
            reached_target = true;
            break;
        }

        // Overshoot check — matches detonation.rs
        if burn_remaining <= 0.0 {
            let to_target_x = target_x - x;
            let to_target_y = target_y - y;
            let dot = vx * to_target_x + vy * to_target_y;
            if dot < 0.0 {
                reached_target = true;
                break;
            }
        }

        // OOB / ground check
        if y <= config::GROUND_Y
            || !(-config::OOB_MARGIN..=config::WORLD_WIDTH + config::OOB_MARGIN).contains(&x)
            || y > config::WORLD_HEIGHT + config::OOB_MARGIN
        {
            break;
        }
    }

    ArcPrediction {
        points,
        time_to_target: time,
        reaches_target: reached_target,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::components::InterceptorType;

    fn standard_profile() -> InterceptorProfile {
        config::interceptor_profile(InterceptorType::Standard)
    }

    #[test]
    fn arc_reaches_straight_up() {
        let pred = predict_arc(160.0, config::GROUND_Y, 160.0, 400.0, &standard_profile(), 0.0);
        assert!(pred.reaches_target, "Should reach target directly above");
        assert!(pred.time_to_target > 0.5 && pred.time_to_target < 5.0);
    }

    #[test]
    fn arc_reaches_diagonal_target() {
        let pred = predict_arc(160.0, config::GROUND_Y, 640.0, 400.0, &standard_profile(), 0.0);
        assert!(pred.reaches_target, "Should reach diagonal target");
        assert!(pred.time_to_target > 0.5 && pred.time_to_target < 8.0);
    }

    #[test]
    fn arc_unreachable_far_target() {
        // Target extremely far away — interceptor should run out of energy
        let pred = predict_arc(160.0, config::GROUND_Y, 10000.0, 10000.0, &standard_profile(), 0.0);
        assert!(!pred.reaches_target, "Should not reach extremely far target");
    }

    #[test]
    fn arc_starts_at_battery_position() {
        let pred = predict_arc(160.0, config::GROUND_Y, 640.0, 400.0, &standard_profile(), 0.0);
        assert!(!pred.points.is_empty());
        let (px, py) = pred.points[0];
        assert!((px - 160.0).abs() < 0.01);
        assert!((py - config::GROUND_Y).abs() < 0.01);
    }

    #[test]
    fn arc_has_reasonable_point_count() {
        let pred = predict_arc(160.0, config::GROUND_Y, 640.0, 400.0, &standard_profile(), 0.0);
        assert!(
            pred.points.len() > 10,
            "Should have enough points for rendering: got {}",
            pred.points.len()
        );
    }

    #[test]
    fn arc_from_right_battery() {
        let pred = predict_arc(1120.0, config::GROUND_Y, 640.0, 400.0, &standard_profile(), 0.0);
        assert!(pred.reaches_target, "Right battery should reach center target");
        let (px, _) = pred.points[0];
        assert!((px - 1120.0).abs() < 0.01);
    }

    #[test]
    fn arc_sprint_faster_than_standard() {
        let std_pred = predict_arc(160.0, config::GROUND_Y, 400.0, 200.0, &standard_profile(), 0.0);
        let sprint_profile = config::interceptor_profile(InterceptorType::Sprint);
        let sprint_pred = predict_arc(160.0, config::GROUND_Y, 400.0, 200.0, &sprint_profile, 0.0);
        assert!(std_pred.reaches_target);
        assert!(sprint_pred.reaches_target);
        assert!(
            sprint_pred.time_to_target < std_pred.time_to_target,
            "Sprint ({:.2}s) should reach nearby target faster than Standard ({:.2}s)",
            sprint_pred.time_to_target,
            std_pred.time_to_target
        );
    }

    #[test]
    fn arc_sprint_vs_standard_different_trajectories() {
        let std_pred = predict_arc(160.0, config::GROUND_Y, 640.0, 400.0, &standard_profile(), 0.0);
        let sprint_profile = config::interceptor_profile(InterceptorType::Sprint);
        let sprint_pred = predict_arc(160.0, config::GROUND_Y, 640.0, 400.0, &sprint_profile, 0.0);
        // Different point counts indicate different trajectories
        assert_ne!(
            std_pred.points.len(),
            sprint_pred.points.len(),
            "Sprint and Standard should produce different trajectories"
        );
    }

    #[test]
    fn arc_prediction_with_wind() {
        let no_wind = predict_arc(640.0, config::GROUND_Y, 640.0, 400.0, &standard_profile(), 0.0);
        let with_wind = predict_arc(640.0, config::GROUND_Y, 640.0, 400.0, &standard_profile(), 15.0);
        // Wind should shift the final x position
        let no_wind_last = no_wind.points.last().unwrap().0;
        let wind_last = with_wind.points.last().unwrap().0;
        assert!(
            (wind_last - no_wind_last).abs() > 1.0,
            "Wind should curve the arc: no_wind_x={no_wind_last:.1}, wind_x={wind_last:.1}"
        );
    }
}
