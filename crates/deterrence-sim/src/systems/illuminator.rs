//! Illuminator scheduler system — manages channel assignment, release, and time-sharing.
//!
//! 3 illuminator channels provide terminal guidance for semi-active missiles.
//! When more engagements need illumination than channels available, the system
//! time-shares channels and reduces Pk accordingly.

use std::collections::HashMap;

use hecs::World;

use deterrence_core::components::{Illuminator, MissileState};
use deterrence_core::constants::*;
use deterrence_core::enums::{EngagementPhase, IlluminatorStatus, MissilePhase, WeaponType};
use deterrence_core::types::Position;

use crate::engagement::Engagement;

/// Run the illuminator scheduler for one tick.
pub fn run(
    world: &mut World,
    engagements: &mut HashMap<u32, Engagement>,
    illuminator_queue: &mut Vec<u32>,
    _current_tick: u64,
) {
    // Step 1: Release illuminators from completed/aborted engagements
    release_completed(world, engagements);

    // Step 2: Identify engagements needing illumination (interceptor near target)
    identify_candidates(world, engagements, illuminator_queue);

    // Step 3: Assign idle illuminators to queued engagements (lowest TTI first)
    assign_illuminators(world, engagements, illuminator_queue);

    // Step 4: Handle time-sharing if queue still has waiting entries
    update_timesharing(world, engagements, illuminator_queue);
}

/// Release illuminators from engagements that are complete, aborted, or no longer valid.
fn release_completed(world: &mut World, engagements: &mut HashMap<u32, Engagement>) {
    // Collect channels to release
    let mut channels_to_release: Vec<u8> = Vec::new();

    // Release from completed/aborted engagements
    for eng in engagements.values_mut() {
        if let Some(channel) = eng.illuminator_channel {
            let should_release = matches!(
                eng.phase,
                EngagementPhase::Complete | EngagementPhase::Aborted
            ) || eng.interceptor_entity.is_none();

            if should_release {
                channels_to_release.push(channel);
                eng.illuminator_channel = None;
            }
        }
    }

    // Also release orphaned illuminators (assigned to engagements that no longer exist,
    // e.g. cleaned up by fire_control after BDA_DELAY).
    {
        let mut query = world.query::<&Illuminator>();
        for (_, illum) in query.iter() {
            if let Some(eng_id) = illum.assigned_engagement {
                if !engagements.contains_key(&eng_id)
                    && !channels_to_release.contains(&illum.channel_id)
                {
                    channels_to_release.push(illum.channel_id);
                }
            }
        }
    }

    // Reset illuminator components
    if !channels_to_release.is_empty() {
        let mut query = world.query::<&mut Illuminator>();
        for (_, illum) in query.iter() {
            if channels_to_release.contains(&illum.channel_id) {
                illum.status = IlluminatorStatus::Idle;
                illum.assigned_engagement = None;
                illum.dwell_remaining_secs = 0.0;
            }
        }
    }
}

/// Identify engagements that need illumination and add them to the queue.
fn identify_candidates(
    world: &World,
    engagements: &HashMap<u32, Engagement>,
    illuminator_queue: &mut Vec<u32>,
) {
    // Remove stale entries from queue (engagements that no longer need illumination)
    illuminator_queue.retain(|eng_id| {
        engagements.get(eng_id).is_some_and(|eng| {
            // Still needs illumination: has interceptor, not complete/aborted, no channel yet
            eng.illuminator_channel.is_none()
                && eng.interceptor_entity.is_some()
                && !matches!(
                    eng.phase,
                    EngagementPhase::Complete | EngagementPhase::Aborted
                )
                && eng.weapon_type != WeaponType::ExtendedRange
        })
    });

    // Find new candidates: engagements with interceptors near terminal range
    for eng in engagements.values() {
        // Skip if already has illuminator or already in queue
        if eng.illuminator_channel.is_some() {
            continue;
        }
        if illuminator_queue.contains(&eng.id) {
            continue;
        }

        // Skip if not in flight phases
        if !matches!(
            eng.phase,
            EngagementPhase::Launched | EngagementPhase::Midcourse | EngagementPhase::Terminal
        ) {
            continue;
        }

        // ER missiles have active seekers — no illuminator needed
        if eng.weapon_type == WeaponType::ExtendedRange {
            continue;
        }

        let interceptor_entity = match eng.interceptor_entity {
            Some(e) => e,
            None => continue,
        };

        // Check if interceptor is in midcourse and near terminal range
        let missile_in_midcourse = world
            .get::<&MissileState>(interceptor_entity)
            .map(|m| m.phase == MissilePhase::Midcourse)
            .unwrap_or(false);

        if !missile_in_midcourse {
            continue;
        }

        // Check distance to target
        let interceptor_pos = match world.get::<&Position>(interceptor_entity) {
            Ok(p) => *p,
            Err(_) => continue,
        };
        let target_pos = match world.get::<&Position>(eng.target_entity) {
            Ok(p) => *p,
            Err(_) => continue,
        };

        let distance = interceptor_pos.range_to(&target_pos);

        // Request illumination when within 1.5x terminal guidance range
        // (slightly early to ensure illuminator is ready when needed)
        if distance <= TERMINAL_GUIDANCE_RANGE * 1.5 {
            illuminator_queue.push(eng.id);
        }
    }

    // Sort queue by TTI (lowest first — most urgent gets priority)
    illuminator_queue.sort_by(|a, b| {
        let tti_a = engagements
            .get(a)
            .map(|e| e.time_to_intercept)
            .unwrap_or(f64::MAX);
        let tti_b = engagements
            .get(b)
            .map(|e| e.time_to_intercept)
            .unwrap_or(f64::MAX);
        tti_a
            .partial_cmp(&tti_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Assign idle illuminator channels to queued engagements.
fn assign_illuminators(
    world: &mut World,
    engagements: &mut HashMap<u32, Engagement>,
    illuminator_queue: &mut Vec<u32>,
) {
    if illuminator_queue.is_empty() {
        return;
    }

    // Collect idle channel IDs
    let mut idle_channels: Vec<u8> = Vec::new();
    {
        let mut query = world.query::<&Illuminator>();
        for (_, illum) in query.iter() {
            if illum.status == IlluminatorStatus::Idle {
                idle_channels.push(illum.channel_id);
            }
        }
    }
    idle_channels.sort();

    // Assign idle channels to queued engagements
    let mut assigned_pairs: Vec<(u32, u8)> = Vec::new();
    for channel_id in idle_channels {
        if illuminator_queue.is_empty() {
            break;
        }
        let eng_id = illuminator_queue.remove(0);
        assigned_pairs.push((eng_id, channel_id));
    }

    // Apply assignments to engagements
    for (eng_id, channel_id) in &assigned_pairs {
        if let Some(eng) = engagements.get_mut(eng_id) {
            eng.illuminator_channel = Some(*channel_id);
        }
    }

    // Apply assignments to illuminator components
    {
        let mut query = world.query::<&mut Illuminator>();
        for (_, illum) in query.iter() {
            for (eng_id, channel_id) in &assigned_pairs {
                if illum.channel_id == *channel_id {
                    illum.status = IlluminatorStatus::Active;
                    illum.assigned_engagement = Some(*eng_id);
                    illum.dwell_remaining_secs = ILLUMINATOR_DWELL_TIME;
                }
            }
        }
    }
}

/// Handle time-sharing when queue has waiting engagements and all channels are busy.
fn update_timesharing(
    world: &mut World,
    engagements: &mut HashMap<u32, Engagement>,
    illuminator_queue: &mut Vec<u32>,
) {
    if illuminator_queue.is_empty() {
        // No waiting engagements — ensure all active illuminators are not in TimeSharing
        let mut query = world.query::<&mut Illuminator>();
        for (_, illum) in query.iter() {
            if illum.status == IlluminatorStatus::TimeSharing {
                illum.status = IlluminatorStatus::Active;
            }
        }
        return;
    }

    // Queue has waiting engagements — mark active illuminators as TimeSharing
    // and count how many are sharing
    let active_count = {
        let mut count = 0u32;
        let mut query = world.query::<&Illuminator>();
        for (_, illum) in query.iter() {
            if matches!(
                illum.status,
                IlluminatorStatus::Active | IlluminatorStatus::TimeSharing
            ) {
                count += 1;
            }
        }
        count
    };

    if active_count == 0 {
        return;
    }

    // Total engagements needing illumination = active channels + waiting queue
    let total_needing = active_count as usize + illuminator_queue.len();

    // Set all active illuminators to TimeSharing and decrement dwell
    let mut expired_channels: Vec<(u8, u32)> = Vec::new(); // (channel_id, old_eng_id)
    {
        let mut query = world.query::<&mut Illuminator>();
        for (_, illum) in query.iter() {
            if matches!(
                illum.status,
                IlluminatorStatus::Active | IlluminatorStatus::TimeSharing
            ) {
                illum.status = IlluminatorStatus::TimeSharing;
                illum.dwell_remaining_secs -= DT;

                if illum.dwell_remaining_secs <= 0.0 {
                    if let Some(old_eng_id) = illum.assigned_engagement {
                        expired_channels.push((illum.channel_id, old_eng_id));
                    }
                }
            }
        }
    }

    // Rotate expired channels: release old engagement, assign next from queue
    for (channel_id, old_eng_id) in &expired_channels {
        // Release old engagement's illuminator
        if let Some(old_eng) = engagements.get_mut(old_eng_id) {
            old_eng.illuminator_channel = None;
        }
        // Put old engagement back in queue (if still valid)
        if let Some(old_eng) = engagements.get(old_eng_id) {
            if !matches!(
                old_eng.phase,
                EngagementPhase::Complete | EngagementPhase::Aborted
            ) && old_eng.interceptor_entity.is_some()
            {
                illuminator_queue.push(*old_eng_id);
            }
        }

        // Assign next from queue
        if let Some(next_eng_id) = illuminator_queue.first().copied() {
            illuminator_queue.remove(0);
            if let Some(next_eng) = engagements.get_mut(&next_eng_id) {
                next_eng.illuminator_channel = Some(*channel_id);
            }

            // Update illuminator component
            let mut query = world.query::<&mut Illuminator>();
            for (_, illum) in query.iter() {
                if illum.channel_id == *channel_id {
                    illum.assigned_engagement = Some(next_eng_id);
                    // Time-shared dwell is shorter: split among total needing
                    let share_count = total_needing.max(1);
                    illum.dwell_remaining_secs = ILLUMINATOR_DWELL_TIME / share_count as f64;
                }
            }
        }
    }

    // Adjust Pk for time-sharing penalty on all illuminated engagements
    // Pk penalty: effective pk = base_pk / share_count for time-shared engagements
    // (Only adjust the stored pk if currently time-sharing)
    // Note: we don't mutate pk directly — instead intercept.rs will check
    // illuminator status and apply penalty at intercept time.
}

/// Get the number of engagements sharing illuminators (for Pk penalty calculation).
/// Returns the total count of engagements that need illumination (assigned + queued).
pub fn time_share_count(
    engagements: &HashMap<u32, Engagement>,
    illuminator_queue: &[u32],
) -> usize {
    let assigned = engagements
        .values()
        .filter(|e| e.illuminator_channel.is_some())
        .count();
    assigned + illuminator_queue.len()
}
