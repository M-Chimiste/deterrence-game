//! Threat behavior finite state machine.
//!
//! Pure functions that compute phase transitions and velocity adjustments
//! for threat entities based on their archetype, current phase, and situation.
//! No ECS dependency — operates on plain data.

use deterrence_core::constants::*;
use deterrence_core::enums::{ThreatArchetype, ThreatPhase};
use deterrence_core::types::{Position, Velocity};

use crate::profiles::get_profile;

/// Input to the threat FSM for a single entity.
pub struct ThreatContext {
    pub archetype: ThreatArchetype,
    pub phase: ThreatPhase,
    pub position: Position,
    pub velocity: Velocity,
    pub target: Position,
    pub range_to_target: f64,
    pub is_engaged: bool,
    pub elapsed_in_phase_secs: f64,
}

/// Output from the threat FSM.
pub struct ThreatUpdate {
    pub new_phase: ThreatPhase,
    pub new_velocity: Velocity,
    pub phase_changed: bool,
}

/// Evaluate the FSM for one threat. Returns the updated phase and velocity.
pub fn evaluate(ctx: &ThreatContext) -> ThreatUpdate {
    let no_change = ThreatUpdate {
        new_phase: ctx.phase,
        new_velocity: ctx.velocity,
        phase_changed: false,
    };

    // Terminal states — no transitions
    if matches!(ctx.phase, ThreatPhase::Destroyed | ThreatPhase::Impact) {
        return no_change;
    }

    let profile = get_profile(ctx.archetype);

    match ctx.phase {
        ThreatPhase::Cruise => evaluate_cruise(ctx, &profile),
        ThreatPhase::PopUp => evaluate_popup(ctx, &profile),
        ThreatPhase::Terminal => evaluate_terminal(ctx, &profile),
        ThreatPhase::Evasive => evaluate_evasive(ctx),
        ThreatPhase::Destroyed | ThreatPhase::Impact => no_change,
    }
}

fn evaluate_cruise(
    ctx: &ThreatContext,
    profile: &crate::profiles::ThreatBehaviorProfile,
) -> ThreatUpdate {
    // Sea-skimmers: transition to PopUp before Terminal
    if let Some(popup_range) = profile.popup_range {
        if ctx.range_to_target <= popup_range {
            let new_vel = compute_popup_velocity(ctx, profile);
            return ThreatUpdate {
                new_phase: ThreatPhase::PopUp,
                new_velocity: new_vel,
                phase_changed: true,
            };
        }
    }

    // Non-sea-skimmers: go directly to Terminal (if archetype has one)
    if profile.terminal_range > 0.0 && ctx.range_to_target <= profile.terminal_range {
        let new_vel = compute_terminal_velocity(ctx, profile);
        return ThreatUpdate {
            new_phase: ThreatPhase::Terminal,
            new_velocity: new_vel,
            phase_changed: true,
        };
    }

    // SubsonicDrone: stays Cruise until impact
    if ctx.range_to_target <= THREAT_IMPACT_RANGE {
        return ThreatUpdate {
            new_phase: ThreatPhase::Impact,
            new_velocity: Velocity::new(0.0, 0.0, 0.0),
            phase_changed: true,
        };
    }

    ThreatUpdate {
        new_phase: ctx.phase,
        new_velocity: ctx.velocity,
        phase_changed: false,
    }
}

fn evaluate_popup(
    ctx: &ThreatContext,
    profile: &crate::profiles::ThreatBehaviorProfile,
) -> ThreatUpdate {
    // After popup duration, transition to Terminal
    if ctx.elapsed_in_phase_secs >= THREAT_POPUP_DURATION_SECS {
        let new_vel = compute_terminal_velocity(ctx, profile);
        return ThreatUpdate {
            new_phase: ThreatPhase::Terminal,
            new_velocity: new_vel,
            phase_changed: true,
        };
    }

    ThreatUpdate {
        new_phase: ctx.phase,
        new_velocity: ctx.velocity,
        phase_changed: false,
    }
}

fn evaluate_terminal(
    ctx: &ThreatContext,
    profile: &crate::profiles::ThreatBehaviorProfile,
) -> ThreatUpdate {
    // Impact check
    if ctx.range_to_target <= THREAT_IMPACT_RANGE {
        return ThreatUpdate {
            new_phase: ThreatPhase::Impact,
            new_velocity: Velocity::new(0.0, 0.0, 0.0),
            phase_changed: true,
        };
    }

    // SeaSkimmerMk2 goes evasive when engaged
    if profile.can_evade && ctx.is_engaged {
        let new_vel = compute_evasive_velocity(ctx);
        return ThreatUpdate {
            new_phase: ThreatPhase::Evasive,
            new_velocity: new_vel,
            phase_changed: true,
        };
    }

    ThreatUpdate {
        new_phase: ctx.phase,
        new_velocity: ctx.velocity,
        phase_changed: false,
    }
}

fn evaluate_evasive(ctx: &ThreatContext) -> ThreatUpdate {
    // Still heading toward target, but with jink — impact check
    if ctx.range_to_target <= THREAT_IMPACT_RANGE {
        return ThreatUpdate {
            new_phase: ThreatPhase::Impact,
            new_velocity: Velocity::new(0.0, 0.0, 0.0),
            phase_changed: true,
        };
    }

    ThreatUpdate {
        new_phase: ctx.phase,
        new_velocity: ctx.velocity,
        phase_changed: false,
    }
}

/// Compute velocity for pop-up maneuver: maintain horizontal heading, add climb.
fn compute_popup_velocity(
    ctx: &ThreatContext,
    profile: &crate::profiles::ThreatBehaviorProfile,
) -> Velocity {
    let h_speed = ctx.velocity.horizontal_speed();
    let heading = ctx.velocity.heading();
    // Climb rate to reach popup altitude in popup duration
    let climb_rate = profile.popup_altitude / THREAT_POPUP_DURATION_SECS;
    Velocity::new(h_speed * heading.sin(), h_speed * heading.cos(), climb_rate)
}

/// Compute velocity for terminal phase: speed up, head toward target, dive if needed.
fn compute_terminal_velocity(
    ctx: &ThreatContext,
    profile: &crate::profiles::ThreatBehaviorProfile,
) -> Velocity {
    let dx = ctx.target.x - ctx.position.x;
    let dy = ctx.target.y - ctx.position.y;
    let horiz_dist = (dx * dx + dy * dy).sqrt();

    let new_speed = profile.cruise_speed * profile.terminal_speed_factor;

    if horiz_dist < 1.0 {
        // Directly above target — dive straight down
        return Velocity::new(0.0, 0.0, -new_speed);
    }

    let heading_to_target = dx.atan2(dy);

    // Vertical component: dive back to sea level (or steep dive for ballistic)
    let vz = if profile.terminal_dive {
        // Ballistic: steep dive proportional to altitude
        let dive_angle = (ctx.position.z / horiz_dist).atan();
        -new_speed * dive_angle.sin()
    } else if ctx.position.z > profile.cruise_altitude + 10.0 {
        // Sea-skimmer returning from popup — descend
        -ctx.position.z / 3.0 // descend over ~3 seconds
    } else {
        0.0
    };

    // Horizontal speed component
    let h_speed = (new_speed * new_speed - vz * vz).max(0.0).sqrt();

    Velocity::new(
        h_speed * heading_to_target.sin(),
        h_speed * heading_to_target.cos(),
        vz,
    )
}

/// Compute evasive velocity: maintain heading toward target with lateral jink.
fn compute_evasive_velocity(ctx: &ThreatContext) -> Velocity {
    let dx = ctx.target.x - ctx.position.x;
    let dy = ctx.target.y - ctx.position.y;
    let horiz_dist = (dx * dx + dy * dy).sqrt();

    if horiz_dist < 1.0 {
        return ctx.velocity;
    }

    let heading_to_target = dx.atan2(dy);
    let speed = ctx.velocity.speed();

    // Lateral jink: perpendicular to heading, oscillating based on elapsed time
    let jink_angle = (ctx.elapsed_in_phase_secs * 2.0 * std::f64::consts::PI).sin() * 0.3;
    let effective_heading = heading_to_target + jink_angle;

    Velocity::new(
        speed * effective_heading.sin(),
        speed * effective_heading.cos(),
        0.0,
    )
}
