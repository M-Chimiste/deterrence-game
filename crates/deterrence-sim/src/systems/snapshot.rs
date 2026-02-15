//! Snapshot system: queries the ECS world and builds a complete GameStateSnapshot.
//!
//! This system is read-only — it never modifies the world.

use std::collections::HashMap;

use hecs::World;

use deterrence_core::components::*;
use deterrence_core::enums::*;
use deterrence_core::events::AudioEvent;
use deterrence_core::state::*;
use deterrence_core::types::{Position, SimTime, Velocity};

use crate::engagement::{Engagement, ScoreState};

/// Build a complete GameStateSnapshot from the current world state.
pub fn build_snapshot(
    world: &World,
    time: &SimTime,
    phase: GamePhase,
    doctrine: DoctrineMode,
    audio_events: Vec<AudioEvent>,
    engagements: &HashMap<u32, Engagement>,
    score: &ScoreState,
) -> GameStateSnapshot {
    let own_ship_pos = find_own_ship_position(world);

    GameStateSnapshot {
        time: *time,
        phase,
        doctrine,
        tracks: build_tracks(world, &own_ship_pos),
        engagements: build_engagements(engagements),
        own_ship: build_own_ship(&own_ship_pos),
        radar: build_radar(world),
        vls: build_vls(world),
        illuminators: build_illuminators(world),
        alerts: Vec::new(),
        audio_events,
        score: ScoreView {
            threats_killed: score.threats_killed,
            threats_total: score.threats_total,
            interceptors_fired: score.interceptors_fired,
            assets_protected: score.threats_impacted == 0,
        },
    }
}

/// Find own ship position (used by multiple builders).
fn find_own_ship_position(world: &World) -> Position {
    world
        .query::<(&OwnShip, &Position)>()
        .iter()
        .next()
        .map(|(_, (_, pos))| *pos)
        .unwrap_or_default()
}

/// Build TrackView list from all entities with TrackInfo.
fn build_tracks(world: &World, own_ship_pos: &Position) -> Vec<TrackView> {
    let mut tracks: Vec<TrackView> = world
        .query::<(&Position, &Velocity, &TrackInfo, &PositionHistory)>()
        .iter()
        .map(|(_, (pos, vel, track_info, history))| TrackView {
            track_number: track_info.track_number,
            position: *pos,
            bearing: own_ship_pos.bearing_to(pos),
            range: own_ship_pos.range_to(pos),
            altitude: pos.z,
            speed: vel.speed(),
            heading: vel.heading(),
            classification: track_info.classification,
            iff_status: track_info.iff_status,
            quality: track_info.quality,
            hooked: track_info.hooked,
            history: history.positions.clone(),
        })
        .collect();

    tracks.sort_by_key(|t| t.track_number);
    tracks
}

/// Build EngagementView list from active engagements.
fn build_engagements(engagements: &HashMap<u32, Engagement>) -> Vec<EngagementView> {
    let mut views: Vec<EngagementView> = engagements
        .values()
        .map(|e| EngagementView {
            engagement_id: e.id,
            track_number: e.target_track_number,
            phase: e.phase,
            weapon_type: e.weapon_type,
            pk: e.pk,
            veto_remaining_secs: e.veto_remaining_secs,
            veto_total_secs: e.veto_total_secs,
            illuminator_channel: None, // Phase 6
            time_to_intercept: e.time_to_intercept,
            result: e.result,
        })
        .collect();

    views.sort_by_key(|e| e.engagement_id);
    views
}

/// Build OwnShipView.
fn build_own_ship(own_ship_pos: &Position) -> OwnShipView {
    OwnShipView {
        position: *own_ship_pos,
    }
}

/// Build RadarView from the OwnShip's RadarSystem component.
fn build_radar(world: &World) -> RadarView {
    let active_track_count = {
        let mut query = world.query::<&TrackInfo>();
        query.iter().count() as u32
    };

    world
        .query::<(&OwnShip, &RadarSystem)>()
        .iter()
        .next()
        .map(|(_, (_, radar))| RadarView {
            mode: radar.mode,
            energy_total: radar.energy_budget,
            energy_search: radar.search_energy,
            energy_track: radar.track_energy,
            sweep_angle: radar.sweep_angle,
            sector_center: radar.sector_center,
            sector_width: radar.sector_width,
            active_track_count,
        })
        .unwrap_or_default()
}

/// Build VlsView from the OwnShip's LauncherSystem component.
fn build_vls(world: &World) -> VlsView {
    world
        .query::<(&OwnShip, &LauncherSystem)>()
        .iter()
        .next()
        .map(|(_, (_, launcher))| {
            let mut ready_standard = 0u32;
            let mut ready_extended_range = 0u32;
            let mut ready_point_defense = 0u32;

            for cell in &launcher.cells {
                match cell {
                    CellStatus::Ready(WeaponType::Standard) => ready_standard += 1,
                    CellStatus::Ready(WeaponType::ExtendedRange) => ready_extended_range += 1,
                    CellStatus::Ready(WeaponType::PointDefense) => ready_point_defense += 1,
                    _ => {}
                }
            }

            VlsView {
                total_ready: ready_standard + ready_extended_range + ready_point_defense,
                total_capacity: launcher.cells.len() as u32,
                cells: launcher.cells.clone(),
                ready_standard,
                ready_extended_range,
                ready_point_defense,
            }
        })
        .unwrap_or_default()
}

/// Build IlluminatorView list from Illuminator entities.
fn build_illuminators(world: &World) -> Vec<IlluminatorView> {
    let mut illuminators: Vec<IlluminatorView> = world
        .query::<&Illuminator>()
        .iter()
        .map(|(_, illum)| IlluminatorView {
            channel_id: illum.channel_id,
            status: illum.status,
            assigned_engagement: illum.assigned_engagement,
            queue_depth: 0, // Phase 6
        })
        .collect();

    illuminators.sort_by_key(|i| i.channel_id);
    illuminators
}
