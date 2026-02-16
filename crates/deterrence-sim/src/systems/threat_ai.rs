//! Threat AI system — updates threat behavior each tick.
//!
//! Calls the threat FSM from deterrence-threat-ai to compute phase transitions,
//! then updates ECS components accordingly. Applies acceleration smoothing
//! and altitude hold for realistic kinematics.

use hecs::World;

use deterrence_core::components::{Threat, ThreatProfile};
use deterrence_core::constants::DT;
use deterrence_core::enums::ThreatPhase;
use deterrence_core::events::AudioEvent;
use deterrence_core::types::{Position, Velocity};

use deterrence_threat_ai::fsm::{evaluate, ThreatContext};
use deterrence_threat_ai::profiles::get_profile;

/// Run the threat AI system: evaluate FSM for each threat, apply updates.
pub fn run(world: &mut World, current_tick: u64, audio_events: &mut Vec<AudioEvent>) {
    // Collect updates in a buffer to avoid borrow issues with hecs
    let mut updates: Vec<(hecs::Entity, ThreatPhase, Velocity, bool, bool)> = Vec::new();

    {
        let mut query = world.query::<(&Threat, &Position, &Velocity, &ThreatProfile)>();
        for (entity, (_threat, pos, vel, profile)) in query.iter() {
            // Skip terminal states
            if matches!(profile.phase, ThreatPhase::Destroyed | ThreatPhase::Impact) {
                continue;
            }

            let range = pos.horizontal_range_to(&profile.target);
            let elapsed_in_phase =
                (current_tick.saturating_sub(profile.phase_start_tick)) as f64 * DT;

            let ctx = ThreatContext {
                archetype: profile.archetype,
                phase: profile.phase,
                position: *pos,
                velocity: *vel,
                target: profile.target,
                range_to_target: range,
                is_engaged: profile.is_engaged,
                elapsed_in_phase_secs: elapsed_in_phase,
            };

            let update = evaluate(&ctx);

            // Always process: apply altitude hold during cruise even without phase change
            updates.push((
                entity,
                update.new_phase,
                update.new_velocity,
                update.phase_changed,
                update.smooth,
            ));
        }
    }

    // Apply updates
    for (entity, new_phase, target_velocity, phase_changed, smooth) in updates {
        if phase_changed {
            if let Ok(mut profile) = world.get::<&mut ThreatProfile>(entity) {
                profile.phase = new_phase;
                profile.phase_start_tick = current_tick;
            }
        }

        // Apply velocity: either smooth (acceleration-limited) or instant
        if let Ok(mut vel) = world.get::<&mut Velocity>(entity) {
            if smooth {
                // Get profile for acceleration limits
                let archetype = world
                    .get::<&ThreatProfile>(entity)
                    .map(|p| p.archetype)
                    .ok();

                if let Some(archetype) = archetype {
                    let bp = get_profile(archetype);
                    *vel = smooth_velocity(&vel, &target_velocity, &bp);
                } else {
                    *vel = target_velocity;
                }
            } else if phase_changed {
                // Instant velocity change (Impact/Destroyed)
                *vel = target_velocity;
            }
            // If !phase_changed && !smooth, keep current velocity (no-op)
        }

        // Apply altitude hold during cruise for sea-skimmers
        if !phase_changed && new_phase == ThreatPhase::Cruise {
            let archetype = world
                .get::<&ThreatProfile>(entity)
                .map(|p| p.archetype)
                .ok();

            if let Some(archetype) = archetype {
                let bp = get_profile(archetype);
                // Only apply altitude hold if the archetype has a low cruise altitude (sea-skimmer)
                if bp.cruise_altitude < 100.0 {
                    if let (Ok(pos), Ok(mut vel)) = (
                        world.get::<&Position>(entity),
                        world.get::<&mut Velocity>(entity),
                    ) {
                        let altitude_error = bp.cruise_altitude - pos.z;
                        let correction =
                            altitude_error.clamp(-bp.max_descent_rate * DT, bp.max_climb_rate * DT);
                        // Blend z-velocity toward correction
                        vel.z += correction;
                    }
                }
            }
        }

        // Emit VampireImpact audio event when threat reaches target
        if new_phase == ThreatPhase::Impact && phase_changed {
            if let Ok(pos) = world.get::<&Position>(entity) {
                let bearing = Position::default().bearing_to(&pos);
                audio_events.push(AudioEvent::VampireImpact { bearing });
            }
        }
    }
}

/// Smoothly interpolate from current velocity toward target velocity using acceleration limits.
fn smooth_velocity(
    current: &Velocity,
    target: &Velocity,
    profile: &deterrence_threat_ai::profiles::ThreatBehaviorProfile,
) -> Velocity {
    let current_speed = current.speed();
    let target_speed = target.speed();

    // Smooth speed change
    let new_speed = if target_speed > current_speed {
        (current_speed + profile.acceleration * DT).min(target_speed)
    } else {
        (current_speed - profile.deceleration * DT).max(target_speed)
    };

    // Interpolate direction: blend from current heading toward target heading
    // Use the target direction but scale to new_speed
    let target_dir_x = target.x;
    let target_dir_y = target.y;
    let target_dir_z = target.z;
    let target_mag = target_speed;

    if target_mag < 0.01 {
        return Velocity::new(0.0, 0.0, 0.0);
    }

    // Normalize target direction and apply new_speed
    let scale = new_speed / target_mag;
    Velocity::new(
        target_dir_x * scale,
        target_dir_y * scale,
        target_dir_z * scale,
    )
}
