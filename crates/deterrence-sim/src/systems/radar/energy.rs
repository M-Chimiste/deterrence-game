//! Radar energy budget management.
//!
//! Splits total energy between search and tracking based on active track count.
//! Each tracked contact consumes RADAR_ENERGY_PER_TRACK from the budget.
//! Search energy is what remains after track allocation.
//! Also advances the radar sweep angle each tick.

use hecs::World;

use deterrence_core::components::{OwnShip, RadarSystem, TrackInfo};
use deterrence_core::constants::{
    DT, RADAR_ENERGY_PER_TRACK, RADAR_MIN_SEARCH_ENERGY, RADAR_SWEEP_RATE,
};

/// Update radar energy allocation and advance the sweep angle.
pub fn run(world: &mut World) {
    let active_track_count = {
        let mut query = world.query::<&TrackInfo>();
        query.iter().count() as u32
    };

    for (_entity, (_own, radar)) in world.query_mut::<(&OwnShip, &mut RadarSystem)>() {
        // Energy allocation: tracks consume energy, remainder goes to search
        let track_energy = (active_track_count as f64) * RADAR_ENERGY_PER_TRACK;
        let search_energy = (radar.energy_budget - track_energy).max(RADAR_MIN_SEARCH_ENERGY);
        let actual_track_energy = radar.energy_budget - search_energy;

        radar.track_energy = actual_track_energy;
        radar.search_energy = search_energy;

        // Advance sweep angle
        radar.sweep_angle =
            (radar.sweep_angle + RADAR_SWEEP_RATE * DT).rem_euclid(std::f64::consts::TAU);
    }
}
