//! Missile kinematics system — handles interceptor flight phase transitions and guidance.
//!
//! Manages Boost → Midcourse → Terminal phase transitions and velocity retargeting
//! so interceptors track moving targets rather than flying to a fixed PIP.

use std::collections::HashMap;

use hecs::World;

use deterrence_core::components::{Interceptor, MissileState};
use deterrence_core::constants::*;
use deterrence_core::enums::{MissilePhase, WeaponType};
use deterrence_core::types::{Position, Velocity};

use crate::engagement::Engagement;

/// Run missile kinematics for one tick: phase transitions and velocity retargeting.
pub fn run(world: &mut World, engagements: &HashMap<u32, Engagement>, current_tick: u64) {
    // Collect updates to apply (avoid borrow conflicts with hecs)
    let mut updates: Vec<(hecs::Entity, MissilePhase, Option<Velocity>)> = Vec::new();

    {
        let mut query = world.query::<(&Interceptor, &MissileState, &Position)>();
        for (entity, (_interceptor, missile, interceptor_pos)) in query.iter() {
            let eng = match engagements.get(&missile.engagement_id) {
                Some(e) => e,
                None => continue,
            };

            match missile.phase {
                MissilePhase::Boost => {
                    // Transition to Midcourse after boost duration
                    let elapsed_secs =
                        (current_tick.saturating_sub(missile.phase_start_tick)) as f64 * DT;
                    if elapsed_secs >= MISSILE_BOOST_DURATION_SECS {
                        updates.push((entity, MissilePhase::Midcourse, None));
                    }
                    // During boost, velocity stays fixed (straight line toward PIP)
                }
                MissilePhase::Midcourse => {
                    // Get target position for guidance
                    let target_pos = match world.get::<&Position>(eng.target_entity) {
                        Ok(p) => *p,
                        Err(_) => continue, // target gone, fire_control will abort
                    };

                    let distance = interceptor_pos.range_to(&target_pos);

                    // Check for terminal transition
                    if distance <= TERMINAL_GUIDANCE_RANGE {
                        // ER missiles have active seekers — no illuminator needed
                        let can_go_terminal = missile.weapon_type == WeaponType::ExtendedRange
                            || eng.illuminator_channel.is_some();

                        if can_go_terminal {
                            // Retarget velocity toward target and transition
                            let new_vel = retarget_velocity(
                                interceptor_pos,
                                &target_pos,
                                missile.weapon_type,
                            );
                            updates.push((entity, MissilePhase::Terminal, Some(new_vel)));
                            continue;
                        }
                    }

                    // Midcourse guidance: retarget velocity toward current target position
                    let new_vel =
                        retarget_velocity(interceptor_pos, &target_pos, missile.weapon_type);
                    updates.push((entity, MissilePhase::Midcourse, Some(new_vel)));
                }
                MissilePhase::Terminal => {
                    // Terminal guidance: continue tracking target
                    let target_pos = match world.get::<&Position>(eng.target_entity) {
                        Ok(p) => *p,
                        Err(_) => continue,
                    };
                    let new_vel =
                        retarget_velocity(interceptor_pos, &target_pos, missile.weapon_type);
                    updates.push((entity, MissilePhase::Terminal, Some(new_vel)));
                }
                MissilePhase::Complete => {}
            }
        }
    }

    // Apply updates
    for (entity, new_phase, new_vel) in updates {
        if let Ok(mut missile) = world.get::<&mut MissileState>(entity) {
            if missile.phase != new_phase {
                missile.phase = new_phase;
                missile.phase_start_tick = current_tick;
            }
        }
        if let Some(vel) = new_vel {
            if let Ok(mut v) = world.get::<&mut Velocity>(entity) {
                *v = vel;
            }
        }
    }
}

/// Calculate velocity vector pointing from interceptor toward target at missile speed.
fn retarget_velocity(from: &Position, to: &Position, weapon_type: WeaponType) -> Velocity {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dz = to.z - from.z;
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let speed = missile_speed(weapon_type);

    if dist > 1.0 {
        Velocity::new(speed * dx / dist, speed * dy / dist, speed * dz / dist)
    } else {
        Velocity::new(0.0, speed, 0.0) // fallback
    }
}

/// Get missile speed for a weapon type.
fn missile_speed(weapon_type: WeaponType) -> f64 {
    match weapon_type {
        WeaponType::Standard => STANDARD_MISSILE_SPEED,
        WeaponType::ExtendedRange => EXTENDED_RANGE_MISSILE_SPEED,
        WeaponType::PointDefense => POINT_DEFENSE_MISSILE_SPEED,
    }
}
