//! Threat AI system — updates threat behavior each tick.
//!
//! Calls the threat FSM from deterrence-threat-ai to compute phase transitions,
//! then updates ECS components accordingly.

use hecs::World;

use deterrence_core::components::{Threat, ThreatProfile};
use deterrence_core::constants::DT;
use deterrence_core::enums::ThreatPhase;
use deterrence_core::events::AudioEvent;
use deterrence_core::types::{Position, Velocity};

use deterrence_threat_ai::fsm::{evaluate, ThreatContext};

/// Run the threat AI system: evaluate FSM for each threat, apply updates.
pub fn run(world: &mut World, current_tick: u64, audio_events: &mut Vec<AudioEvent>) {
    // Collect updates in a buffer to avoid borrow issues with hecs
    let mut updates: Vec<(hecs::Entity, ThreatPhase, Velocity)> = Vec::new();

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
            if update.phase_changed {
                updates.push((entity, update.new_phase, update.new_velocity));
            }
        }
    }

    // Apply updates
    for (entity, new_phase, new_velocity) in updates {
        if let Ok(mut profile) = world.get::<&mut ThreatProfile>(entity) {
            profile.phase = new_phase;
            profile.phase_start_tick = current_tick;
        }
        if let Ok(mut vel) = world.get::<&mut Velocity>(entity) {
            *vel = new_velocity;
        }

        // Emit VampireImpact audio event when threat reaches target
        if new_phase == ThreatPhase::Impact {
            if let Ok(pos) = world.get::<&Position>(entity) {
                let bearing = Position::default().bearing_to(&pos);
                audio_events.push(AudioEvent::VampireImpact { bearing });
            }
        }
    }
}
