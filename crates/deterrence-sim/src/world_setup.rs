//! Entity spawn factories for setting up the simulation world.
//!
//! Creates own ship, illuminators, and threat entities with
//! appropriate component bundles.

use hecs::World;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use deterrence_core::components::*;
use deterrence_core::constants::*;
use deterrence_core::enums::*;
use deterrence_core::types::{Position, Velocity};

/// Set up the initial mission world: own ship and illuminators.
/// Threats are now spawned by the wave scheduler system.
pub fn setup_mission(world: &mut World) {
    spawn_own_ship(world);
    spawn_illuminators(world);
}

/// Spawn the player's own ship at the origin with full systems.
pub fn spawn_own_ship(world: &mut World) -> hecs::Entity {
    let radar = RadarSystem {
        energy_budget: RADAR_TOTAL_ENERGY,
        search_energy: RADAR_TOTAL_ENERGY,
        track_energy: 0.0,
        sector_center: 0.0,
        sector_width: RADAR_DEFAULT_SECTOR_WIDTH,
        mode: RadarMode::default(),
        sweep_angle: 0.0,
    };

    let launcher = LauncherSystem {
        cells: build_default_vls_loadout(),
    };

    world.spawn((
        OwnShip,
        Position::new(0.0, 0.0, 0.0),
        Velocity::new(0.0, 0.0, 0.0),
        radar,
        launcher,
    ))
}

/// Build the default VLS loadout: 64 cells.
/// 32 Standard, 16 Extended Range, 16 Point Defense.
fn build_default_vls_loadout() -> Vec<CellStatus> {
    let mut cells = Vec::with_capacity(VLS_CELL_COUNT);
    for i in 0..VLS_CELL_COUNT {
        let status = if i < 32 {
            CellStatus::Ready(WeaponType::Standard)
        } else if i < 48 {
            CellStatus::Ready(WeaponType::ExtendedRange)
        } else {
            CellStatus::Ready(WeaponType::PointDefense)
        };
        cells.push(status);
    }
    cells
}

/// Spawn illuminator entities (separate from own ship for ECS flexibility).
pub fn spawn_illuminators(world: &mut World) {
    for i in 0..ILLUMINATOR_COUNT {
        world.spawn((Illuminator {
            channel_id: i,
            status: IlluminatorStatus::default(),
            assigned_engagement: None,
        },));
    }
}

/// Spawn a wave of threats at random bearings, heading toward the origin.
/// Threats start undetected (with DetectionCounter, not TrackInfo).
pub fn spawn_threat_wave(world: &mut World, rng: &mut ChaCha8Rng, count: usize) {
    for _ in 0..count {
        spawn_threat(world, rng, ThreatArchetype::SeaSkimmerMk1);
    }
}

/// Spawn a single threat entity heading toward the origin.
/// The threat starts undetected — the radar detection system will
/// promote it to a tracked entity after enough consecutive hits.
pub fn spawn_threat(
    world: &mut World,
    rng: &mut ChaCha8Rng,
    archetype: ThreatArchetype,
) -> hecs::Entity {
    let (speed, altitude, rcs) = threat_archetype_params(archetype);

    // Random bearing from 0..TAU, at a range near the radar horizon.
    let bearing: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
    let range: f64 = rng.gen_range(150_000.0..180_000.0);

    // Position: bearing is measured from North (y-axis) clockwise to East (x-axis).
    let x = range * bearing.sin();
    let y = range * bearing.cos();
    let position = Position::new(x, y, altitude);

    // Velocity: heading toward origin.
    let to_origin_bearing = (bearing + std::f64::consts::PI) % std::f64::consts::TAU;
    let vx = speed * to_origin_bearing.sin();
    let vy = speed * to_origin_bearing.cos();
    let velocity = Velocity::new(vx, vy, 0.0);

    let threat_profile = ThreatProfile {
        archetype,
        phase: ThreatPhase::Cruise,
        target: Position::new(0.0, 0.0, 0.0),
        phase_start_tick: 0,
        is_engaged: false,
    };

    world.spawn((
        Threat,
        position,
        velocity,
        DetectionCounter::default(),
        threat_profile,
        RadarCrossSection { base_rcs_m2: rcs },
        PositionHistory::default(),
    ))
}

/// Spawn a pre-tracked threat entity (for tests that need immediate tracks).
/// Unlike `spawn_threat`, this creates the entity with `TrackInfo` already attached.
#[cfg(test)]
pub fn spawn_tracked_threat(
    world: &mut World,
    rng: &mut ChaCha8Rng,
    next_track: &mut u32,
    archetype: ThreatArchetype,
) -> hecs::Entity {
    let (speed, altitude, rcs) = threat_archetype_params(archetype);

    let bearing: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
    let range: f64 = rng.gen_range(150_000.0..180_000.0);

    let x = range * bearing.sin();
    let y = range * bearing.cos();
    let position = Position::new(x, y, altitude);

    let to_origin_bearing = (bearing + std::f64::consts::PI) % std::f64::consts::TAU;
    let vx = speed * to_origin_bearing.sin();
    let vy = speed * to_origin_bearing.cos();
    let velocity = Velocity::new(vx, vy, 0.0);

    let track_number = *next_track;
    *next_track += 1;

    let track_info = TrackInfo {
        track_number,
        quality: 1.0,
        classification: Classification::Hostile,
        iff_status: IffStatus::NoValidResponse,
        hooked: false,
        hits: TRACK_INITIATE_HITS,
        misses: 0,
    };

    let threat_profile = ThreatProfile {
        archetype,
        phase: ThreatPhase::Cruise,
        target: Position::new(0.0, 0.0, 0.0),
        phase_start_tick: 0,
        is_engaged: false,
    };

    world.spawn((
        Threat,
        position,
        velocity,
        track_info,
        threat_profile,
        RadarCrossSection { base_rcs_m2: rcs },
        PositionHistory::default(),
    ))
}

/// Get kinematic parameters for a threat archetype: (speed m/s, altitude m, rcs m²).
fn threat_archetype_params(archetype: ThreatArchetype) -> (f64, f64, f64) {
    match archetype {
        ThreatArchetype::SeaSkimmerMk1 => {
            (SEA_SKIMMER_SPEED, SEA_SKIMMER_ALTITUDE, SEA_SKIMMER_RCS)
        }
        ThreatArchetype::SeaSkimmerMk2 => (
            SEA_SKIMMER_SPEED * 1.1,
            SEA_SKIMMER_ALTITUDE * 0.8,
            SEA_SKIMMER_RCS * 0.7,
        ),
        ThreatArchetype::SupersonicCruiser => {
            (SUPERSONIC_CRUISER_SPEED, 5000.0, SUPERSONIC_CRUISER_RCS)
        }
        ThreatArchetype::SubsonicDrone => (100.0, 3000.0, 0.5),
        ThreatArchetype::TacticalBallistic => (1500.0, 30_000.0, 1.0),
    }
}
