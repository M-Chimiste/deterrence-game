//! Fire control system — creates engagements, manages veto clock, launches interceptors.

use std::collections::HashMap;

use hecs::World;
use rand_chacha::ChaCha8Rng;

use deterrence_core::components::*;
use deterrence_core::constants::*;
use deterrence_core::enums::*;
use deterrence_core::events::AudioEvent;
use deterrence_core::types::{Position, Velocity};

use crate::engagement::{Engagement, ScoreState};

/// Run the fire control system for one tick.
#[allow(clippy::too_many_arguments)]
pub fn run(
    world: &mut World,
    engagements: &mut HashMap<u32, Engagement>,
    next_engagement_id: &mut u32,
    next_track_number: &mut u32,
    rng: &mut ChaCha8Rng,
    audio_events: &mut Vec<AudioEvent>,
    score: &mut ScoreState,
    doctrine: DoctrineMode,
    current_tick: u64,
) {
    // Step 1: Mark threat entities as engaged/not-engaged
    update_threat_engagement_status(world, engagements);

    // Step 2: Remove completed/aborted engagements older than BDA_DELAY
    cleanup_completed_engagements(engagements, current_tick);

    // Step 3: Create new engagements for eligible hostile tracks
    if doctrine != DoctrineMode::Manual {
        create_new_engagements(
            world,
            engagements,
            next_engagement_id,
            doctrine,
            current_tick,
        );
    }

    // Step 4: Advance engagement state machines
    let eng_ids: Vec<u32> = engagements.keys().copied().collect();
    for eng_id in eng_ids {
        advance_engagement(
            world,
            engagements,
            eng_id,
            next_track_number,
            rng,
            audio_events,
            score,
            current_tick,
        );
    }
}

/// Update ThreatProfile.is_engaged for all threat entities based on active engagements.
fn update_threat_engagement_status(world: &mut World, engagements: &HashMap<u32, Engagement>) {
    // Collect engaged entity IDs
    let engaged_entities: Vec<hecs::Entity> = engagements
        .values()
        .filter(|e| {
            !matches!(
                e.phase,
                EngagementPhase::Complete | EngagementPhase::Aborted
            )
        })
        .map(|e| e.target_entity)
        .collect();

    for (_entity, (_threat, profile)) in world.query_mut::<(&Threat, &mut ThreatProfile)>() {
        profile.is_engaged = engaged_entities.contains(&_entity);
    }
}

/// Remove engagements that have been Complete/Aborted for longer than BDA_DELAY.
fn cleanup_completed_engagements(engagements: &mut HashMap<u32, Engagement>, current_tick: u64) {
    let bda_ticks = (BDA_DELAY / DT) as u64;
    engagements.retain(|_, eng| {
        if matches!(
            eng.phase,
            EngagementPhase::Complete | EngagementPhase::Aborted
        ) {
            current_tick.saturating_sub(eng.phase_start_tick) < bda_ticks
        } else {
            true
        }
    });
}

/// Scan for hostile tracks and create engagements for eligible ones.
fn create_new_engagements(
    world: &mut World,
    engagements: &mut HashMap<u32, Engagement>,
    next_engagement_id: &mut u32,
    doctrine: DoctrineMode,
    current_tick: u64,
) {
    if engagements.len() >= MAX_SIMULTANEOUS_ENGAGEMENTS {
        return;
    }

    // Collect already-engaged track numbers
    let engaged_tracks: Vec<u32> = engagements
        .values()
        .filter(|e| {
            !matches!(
                e.phase,
                EngagementPhase::Complete | EngagementPhase::Aborted
            )
        })
        .map(|e| e.target_track_number)
        .collect();

    // Find own ship position
    let own_pos = {
        let mut query = world.query::<(&OwnShip, &Position)>();
        query
            .iter()
            .next()
            .map(|(_, (_, pos))| *pos)
            .unwrap_or_default()
    };

    // Collect candidates: entities with TrackInfo + Threat marker
    let mut candidates: Vec<(
        hecs::Entity,
        u32,
        Classification,
        f64,
        Position,
        Velocity,
        f64,
    )> = Vec::new();
    {
        let mut query = world.query::<(
            &Threat,
            &TrackInfo,
            &Position,
            &Velocity,
            &RadarCrossSection,
        )>();
        for (entity, (_threat, track, pos, vel, rcs)) in query.iter() {
            // Already engaged?
            if engaged_tracks.contains(&track.track_number) {
                continue;
            }

            // Classification eligible?
            let eligible = match doctrine {
                DoctrineMode::AutoSpecial => track.classification == Classification::Hostile,
                DoctrineMode::AutoComposite => {
                    track.classification == Classification::Hostile
                        || track.classification == Classification::Suspect
                }
                DoctrineMode::Manual => false,
            };
            if !eligible {
                continue;
            }

            // Track quality sufficient?
            if track.quality < TRACK_FIRM_QUALITY {
                continue;
            }

            let range = own_pos.range_to(pos);
            candidates.push((
                entity,
                track.track_number,
                track.classification,
                track.quality,
                *pos,
                *vel,
                rcs.base_rcs_m2,
            ));
            let _ = range; // used below when processing candidates
        }
    }

    // Create engagements for each candidate
    for (entity, track_number, _class, quality, pos, vel, rcs) in candidates {
        if engagements.len() >= MAX_SIMULTANEOUS_ENGAGEMENTS {
            break;
        }

        let range = own_pos.range_to(&pos);

        // Select weapon and find available cell
        let (weapon_type, cell_idx) = match select_weapon_and_cell(world, range) {
            Some(result) => result,
            None => continue, // no weapons available
        };

        // Mark cell as Assigned
        assign_vls_cell(world, cell_idx, weapon_type);

        // Calculate PIP and Pk
        let missile_speed = missile_speed_for_weapon(weapon_type);
        let (pip, tti) = calculate_pip(&pos, &vel, &own_pos, missile_speed);
        let pk = calculate_pk(weapon_type, range, quality, rcs);

        let eng_id = *next_engagement_id;
        *next_engagement_id += 1;

        engagements.insert(
            eng_id,
            Engagement {
                id: eng_id,
                target_entity: entity,
                target_track_number: track_number,
                phase: EngagementPhase::SolutionCalc,
                weapon_type,
                pk,
                assigned_cell: Some(cell_idx),
                phase_start_tick: current_tick,
                veto_remaining_secs: 0.0,
                veto_total_secs: VETO_CLOCK_DURATION,
                warned_3s: false,
                warned_1s: false,
                illuminator_channel: None,
                interceptor_entity: None,
                time_to_intercept: tti,
                result: None,
                pip,
            },
        );
    }
}

/// Advance one engagement's state machine.
#[allow(clippy::too_many_arguments)]
fn advance_engagement(
    world: &mut World,
    engagements: &mut HashMap<u32, Engagement>,
    eng_id: u32,
    next_track_number: &mut u32,
    _rng: &mut ChaCha8Rng,
    audio_events: &mut Vec<AudioEvent>,
    score: &mut ScoreState,
    current_tick: u64,
) {
    let eng = match engagements.get_mut(&eng_id) {
        Some(e) => e,
        None => return,
    };

    match eng.phase {
        EngagementPhase::SolutionCalc => {
            let elapsed = (current_tick.saturating_sub(eng.phase_start_tick)) as f64 * DT;
            if elapsed >= SOLUTION_CALC_TIME {
                eng.phase = EngagementPhase::Ready;
                eng.phase_start_tick = current_tick;
                eng.veto_remaining_secs = VETO_CLOCK_DURATION;
                audio_events.push(AudioEvent::VetoClockStart {
                    engagement_id: eng.id,
                    duration_secs: VETO_CLOCK_DURATION,
                });
            }
        }
        EngagementPhase::Ready => {
            eng.veto_remaining_secs -= DT;

            // Warnings
            if eng.veto_remaining_secs <= VETO_WARNING_THRESHOLD_1 && !eng.warned_3s {
                audio_events.push(AudioEvent::VetoClockWarning {
                    engagement_id: eng.id,
                    remaining_secs: eng.veto_remaining_secs,
                });
                eng.warned_3s = true;
            }
            if eng.veto_remaining_secs <= VETO_WARNING_THRESHOLD_2 && !eng.warned_1s {
                audio_events.push(AudioEvent::VetoClockWarning {
                    engagement_id: eng.id,
                    remaining_secs: eng.veto_remaining_secs,
                });
                eng.warned_1s = true;
            }

            // Expiry → launch
            if eng.veto_remaining_secs <= 0.0 {
                launch_interceptor(
                    world,
                    eng,
                    next_track_number,
                    audio_events,
                    score,
                    current_tick,
                );
            }
        }
        EngagementPhase::Launched | EngagementPhase::Midcourse | EngagementPhase::Terminal => {
            // Sync engagement phase from interceptor's MissileState
            if let Some(interceptor_e) = eng.interceptor_entity {
                if let Ok(missile) = world.get::<&MissileState>(interceptor_e) {
                    let synced_phase = match missile.phase {
                        MissilePhase::Midcourse => EngagementPhase::Midcourse,
                        MissilePhase::Terminal => EngagementPhase::Terminal,
                        _ => eng.phase,
                    };
                    if synced_phase != eng.phase {
                        eng.phase = synced_phase;
                        eng.phase_start_tick = current_tick;
                    }
                }
            }

            // Update time-to-intercept estimate
            if let (Some(interceptor_e), Ok(target_pos)) = (
                eng.interceptor_entity,
                world.get::<&Position>(eng.target_entity),
            ) {
                if let Ok(interceptor_pos) = world.get::<&Position>(interceptor_e) {
                    let dist = interceptor_pos.range_to(&target_pos);
                    let speed = missile_speed_for_weapon(eng.weapon_type);
                    eng.time_to_intercept = dist / speed;
                }
            }

            // Check if target has been despawned or destroyed
            if world.get::<&Position>(eng.target_entity).is_err() {
                eng.phase = EngagementPhase::Aborted;
                eng.phase_start_tick = current_tick;
                // Despawn interceptor if it exists
                if let Some(interceptor_e) = eng.interceptor_entity.take() {
                    let _ = world.despawn(interceptor_e);
                }
            }
        }
        _ => {}
    }
}

/// Launch an interceptor: expend VLS cell, spawn entity, update engagement.
fn launch_interceptor(
    world: &mut World,
    eng: &mut Engagement,
    next_track_number: &mut u32,
    audio_events: &mut Vec<AudioEvent>,
    score: &mut ScoreState,
    current_tick: u64,
) {
    // Mark VLS cell as Expended
    if let Some(cell_idx) = eng.assigned_cell {
        expend_vls_cell(world, cell_idx);
    }

    // Get own ship position
    let own_pos = {
        let mut query = world.query::<(&OwnShip, &Position)>();
        query
            .iter()
            .next()
            .map(|(_, (_, pos))| *pos)
            .unwrap_or_default()
    };

    // Update PIP from current target position
    if let Ok(target_pos) = world.get::<&Position>(eng.target_entity) {
        if let Ok(target_vel) = world.get::<&Velocity>(eng.target_entity) {
            let speed = missile_speed_for_weapon(eng.weapon_type);
            let (pip, tti) = calculate_pip(&target_pos, &target_vel, &own_pos, speed);
            eng.pip = pip;
            eng.time_to_intercept = tti;
        }
    }

    // Calculate velocity toward PIP
    let dx = eng.pip.x - own_pos.x;
    let dy = eng.pip.y - own_pos.y;
    let dz = eng.pip.z - own_pos.z;
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let speed = missile_speed_for_weapon(eng.weapon_type);
    let velocity = if dist > 1.0 {
        Velocity::new(speed * dx / dist, speed * dy / dist, speed * dz / dist)
    } else {
        Velocity::new(0.0, speed, 0.0) // fallback: straight north
    };

    // Assign track number for interceptor
    let track_number = *next_track_number;
    *next_track_number += 1;

    // Fuel based on weapon type
    let fuel_secs = match eng.weapon_type {
        WeaponType::Standard => STANDARD_MISSILE_MAX_RANGE / STANDARD_MISSILE_SPEED,
        WeaponType::ExtendedRange => {
            EXTENDED_RANGE_MISSILE_MAX_RANGE / EXTENDED_RANGE_MISSILE_SPEED
        }
        WeaponType::PointDefense => POINT_DEFENSE_MISSILE_MAX_RANGE / POINT_DEFENSE_MISSILE_SPEED,
    };

    // Spawn interceptor entity
    let interceptor = world.spawn((
        Interceptor,
        own_pos,
        velocity,
        MissileState {
            phase: MissilePhase::Boost,
            target_track: Some(eng.target_track_number),
            engagement_id: eng.id,
            fuel_secs,
            weapon_type: eng.weapon_type,
            phase_start_tick: current_tick,
        },
        TrackInfo {
            track_number,
            quality: 1.0,
            classification: Classification::Friend,
            iff_status: IffStatus::FriendlyResponse,
            hooked: false,
            hits: 0,
            misses: 0,
        },
        PositionHistory::default(),
        RadarCrossSection { base_rcs_m2: 0.5 },
    ));

    eng.interceptor_entity = Some(interceptor);
    eng.phase = EngagementPhase::Launched;
    eng.phase_start_tick = current_tick;

    score.interceptors_fired += 1;

    audio_events.push(AudioEvent::BirdAway {
        weapon_type: eng.weapon_type,
    });
}

/// Select weapon type based on range and find an available VLS cell.
fn select_weapon_and_cell(world: &World, range: f64) -> Option<(WeaponType, usize)> {
    let preferred = if range > 100_000.0 {
        WeaponType::ExtendedRange
    } else if range > 20_000.0 {
        WeaponType::Standard
    } else {
        WeaponType::PointDefense
    };

    // Fallback order
    let order = match preferred {
        WeaponType::ExtendedRange => [
            WeaponType::ExtendedRange,
            WeaponType::Standard,
            WeaponType::PointDefense,
        ],
        WeaponType::Standard => [
            WeaponType::Standard,
            WeaponType::ExtendedRange,
            WeaponType::PointDefense,
        ],
        WeaponType::PointDefense => [
            WeaponType::PointDefense,
            WeaponType::Standard,
            WeaponType::ExtendedRange,
        ],
    };

    let mut query = world.query::<(&OwnShip, &LauncherSystem)>();
    let launcher = query.iter().next().map(|(_, (_, l))| l)?;

    for weapon in &order {
        for (idx, cell) in launcher.cells.iter().enumerate() {
            if let CellStatus::Ready(w) = cell {
                if w == weapon {
                    return Some((*weapon, idx));
                }
            }
        }
    }

    None
}

/// Mark a VLS cell as Assigned.
fn assign_vls_cell(world: &mut World, cell_idx: usize, weapon_type: WeaponType) {
    for (_entity, (_own, launcher)) in world.query_mut::<(&OwnShip, &mut LauncherSystem)>() {
        if cell_idx < launcher.cells.len() {
            launcher.cells[cell_idx] = CellStatus::Assigned(weapon_type);
        }
    }
}

/// Mark a VLS cell as Expended.
fn expend_vls_cell(world: &mut World, cell_idx: usize) {
    for (_entity, (_own, launcher)) in world.query_mut::<(&OwnShip, &mut LauncherSystem)>() {
        if cell_idx < launcher.cells.len() {
            launcher.cells[cell_idx] = CellStatus::Expended;
        }
    }
}

/// Release a VLS cell back to Ready (on veto/abort).
pub fn release_vls_cell(world: &mut World, cell_idx: usize, weapon_type: WeaponType) {
    for (_entity, (_own, launcher)) in world.query_mut::<(&OwnShip, &mut LauncherSystem)>() {
        if cell_idx < launcher.cells.len() {
            launcher.cells[cell_idx] = CellStatus::Ready(weapon_type);
        }
    }
}

/// Get missile speed for a weapon type.
fn missile_speed_for_weapon(weapon_type: WeaponType) -> f64 {
    match weapon_type {
        WeaponType::Standard => STANDARD_MISSILE_SPEED,
        WeaponType::ExtendedRange => EXTENDED_RANGE_MISSILE_SPEED,
        WeaponType::PointDefense => POINT_DEFENSE_MISSILE_SPEED,
    }
}

/// Calculate predicted intercept point and time-to-intercept.
fn calculate_pip(
    target_pos: &Position,
    target_vel: &Velocity,
    own_pos: &Position,
    missile_speed: f64,
) -> (Position, f64) {
    let range = own_pos.range_to(target_pos);
    let target_speed = target_vel.speed();
    // Closing speed estimate: missile speed + component of target heading toward us
    let closing_speed = missile_speed + target_speed * 0.5;
    let tti = if closing_speed > 0.0 {
        range / closing_speed
    } else {
        range / missile_speed
    };

    let pip = Position::new(
        target_pos.x + target_vel.x * tti,
        target_pos.y + target_vel.y * tti,
        target_pos.z + target_vel.z * tti,
    );
    (pip, tti)
}

/// Calculate probability of kill.
pub fn calculate_pk(weapon_type: WeaponType, range: f64, quality: f64, rcs: f64) -> f64 {
    let base = match weapon_type {
        WeaponType::Standard => PK_STANDARD_BASE,
        WeaponType::ExtendedRange => PK_EXTENDED_RANGE_BASE,
        WeaponType::PointDefense => PK_POINT_DEFENSE_BASE,
    };
    let max_range = match weapon_type {
        WeaponType::Standard => STANDARD_MISSILE_MAX_RANGE,
        WeaponType::ExtendedRange => EXTENDED_RANGE_MISSILE_MAX_RANGE,
        WeaponType::PointDefense => POINT_DEFENSE_MISSILE_MAX_RANGE,
    };
    let range_factor = 1.0 - (range / max_range).min(1.0) * 0.3;
    let quality_factor = quality;
    let rcs_factor = (rcs / 1.0_f64).sqrt().clamp(0.5, 1.5);
    (base * range_factor * quality_factor / rcs_factor).clamp(0.1, 0.95)
}
