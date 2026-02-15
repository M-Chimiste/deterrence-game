//! Intercept evaluation system — checks missile-target proximity and rolls for kill.

use std::collections::HashMap;

use hecs::World;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use deterrence_core::components::{MissileState, ThreatProfile};
use deterrence_core::constants::*;
use deterrence_core::enums::*;
use deterrence_core::events::AudioEvent;
use deterrence_core::types::Position;

use crate::engagement::{Engagement, ScoreState};

/// Run the intercept system: check proximity, roll Pk, handle results.
pub fn run(
    world: &mut World,
    engagements: &mut HashMap<u32, Engagement>,
    rng: &mut ChaCha8Rng,
    audio_events: &mut Vec<AudioEvent>,
    score: &mut ScoreState,
    despawn_buffer: &mut Vec<hecs::Entity>,
) {
    for eng in engagements.values_mut() {
        if !matches!(eng.phase, EngagementPhase::Launched) {
            continue;
        }

        let interceptor_entity = match eng.interceptor_entity {
            Some(e) => e,
            None => continue,
        };

        // Get interceptor position
        let interceptor_pos = match world.get::<&Position>(interceptor_entity) {
            Ok(p) => *p,
            Err(_) => {
                // Interceptor despawned externally
                eng.phase = EngagementPhase::Aborted;
                eng.interceptor_entity = None;
                continue;
            }
        };

        // Get target position
        let target_pos = match world.get::<&Position>(eng.target_entity) {
            Ok(p) => *p,
            Err(_) => {
                // Target gone — abort engagement, despawn interceptor
                eng.phase = EngagementPhase::Aborted;
                eng.interceptor_entity = None;
                despawn_buffer.push(interceptor_entity);
                continue;
            }
        };

        let distance = interceptor_pos.range_to(&target_pos);

        // Proximity check
        if distance <= INTERCEPT_LETHAL_RADIUS {
            let hit = rng.gen_bool(eng.pk.clamp(0.0, 1.0));

            if hit {
                eng.result = Some(InterceptResult::Hit);
                eng.phase = EngagementPhase::Complete;
                score.threats_killed += 1;

                // Mark threat as destroyed
                if let Ok(mut profile) = world.get::<&mut ThreatProfile>(eng.target_entity) {
                    profile.phase = ThreatPhase::Destroyed;
                }

                audio_events.push(AudioEvent::Splash {
                    result: InterceptResult::Hit,
                    track_number: eng.target_track_number,
                });
            } else {
                eng.result = Some(InterceptResult::Miss);
                eng.phase = EngagementPhase::Complete;

                audio_events.push(AudioEvent::Splash {
                    result: InterceptResult::Miss,
                    track_number: eng.target_track_number,
                });
            }

            despawn_buffer.push(interceptor_entity);
            eng.interceptor_entity = None;
            continue;
        }

        // Check fuel exhaustion
        if let Ok(mut missile) = world.get::<&mut MissileState>(interceptor_entity) {
            missile.fuel_secs -= DT;
            if missile.fuel_secs <= 0.0 {
                eng.result = Some(InterceptResult::Miss);
                eng.phase = EngagementPhase::Complete;
                eng.interceptor_entity = None;
                despawn_buffer.push(interceptor_entity);

                audio_events.push(AudioEvent::Splash {
                    result: InterceptResult::Miss,
                    track_number: eng.target_track_number,
                });
            }
        }
    }

    // Despawn collected interceptors
    for entity in despawn_buffer.drain(..) {
        let _ = world.despawn(entity);
    }
}
