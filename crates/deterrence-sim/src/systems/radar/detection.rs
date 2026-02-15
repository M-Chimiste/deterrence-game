//! Radar detection system.
//!
//! Each tick, checks entities within the current radar beam.
//! Computes probability of detection (Pd) from a simplified radar equation.
//! Updates DetectionCounter for pre-track entities and TrackInfo for tracked entities.

use hecs::World;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use deterrence_core::components::{
    DetectionCounter, OwnShip, RadarCrossSection, RadarSystem, TrackInfo,
};
use deterrence_core::constants::*;
use deterrence_core::types::Position;

/// Half-width of the radar beam in radians.
fn beam_half_width() -> f64 {
    RADAR_SWEEP_RATE * DT * RADAR_BEAM_HALF_WIDTH_TICKS
}

/// Check if a bearing (radians) is within the current beam.
fn in_beam(entity_bearing: f64, sweep_angle: f64) -> bool {
    let half = beam_half_width();
    let diff = (entity_bearing - sweep_angle).rem_euclid(std::f64::consts::TAU);
    diff <= half || diff >= (std::f64::consts::TAU - half)
}

/// Check if a bearing is within the radar's search sector.
fn in_sector(entity_bearing: f64, sector_center: f64, sector_width: f64) -> bool {
    if sector_width >= std::f64::consts::TAU - 0.001 {
        return true; // Full 360-degree search
    }
    let half = sector_width / 2.0;
    let diff = (entity_bearing - sector_center).rem_euclid(std::f64::consts::TAU);
    diff <= half || diff >= (std::f64::consts::TAU - half)
}

/// Compute probability of detection from simplified radar equation.
///
/// Model: `SNR = K * (search_energy / total_energy) * (rcs / range^4)`
/// then `Pd = 1 - exp(-SNR)`.
///
/// This produces the characteristic fourth-root law: doubling RCS extends
/// detection range by a factor of 2^(1/4) ≈ 1.19.
pub fn compute_pd(range: f64, rcs: f64, search_energy: f64, energy_total: f64) -> f64 {
    if range < RADAR_MIN_RANGE {
        return 1.0;
    }
    let range_sq = range * range;
    let range_4 = range_sq * range_sq;
    let energy_fraction = search_energy / energy_total;
    let snr = RADAR_K * energy_fraction * rcs / range_4;
    1.0 - (-snr).exp()
}

/// Run radar detection for all detectable entities.
///
/// Must be called AFTER energy.rs (so search_energy is current) and
/// BEFORE tracking.rs (so hit/miss counts are up to date for promotion/drop).
pub fn run(world: &mut World, rng: &mut ChaCha8Rng, current_tick: u64) {
    // Get own ship position and radar state
    let (own_pos, sweep_angle, sector_center, sector_width, search_energy, energy_total) = {
        let mut q = world.query::<(&OwnShip, &Position, &RadarSystem)>();
        match q.iter().next() {
            Some((_, (_, pos, radar))) => (
                *pos,
                radar.sweep_angle,
                radar.sector_center,
                radar.sector_width,
                radar.search_energy,
                radar.energy_budget,
            ),
            None => return,
        }
    };

    // Pass 1: Entities with DetectionCounter (pre-track)
    {
        let mut query = world.query::<(&Position, &RadarCrossSection, &mut DetectionCounter)>();
        for (_entity, (pos, rcs, counter)) in query.iter() {
            // Skip if already evaluated this sweep pass
            if counter.last_sweep_tick == current_tick {
                continue;
            }

            let bearing = own_pos.bearing_to(pos);

            // Must be in both the search sector and the current beam
            if !in_sector(bearing, sector_center, sector_width) {
                continue;
            }
            if !in_beam(bearing, sweep_angle) {
                continue;
            }

            counter.last_sweep_tick = current_tick;

            let range = own_pos.range_to(pos);

            // Beyond max range — automatic miss
            if range > RADAR_MAX_RANGE {
                counter.misses += 1;
                counter.hits = 0;
                continue;
            }

            let pd = compute_pd(range, rcs.base_rcs_m2, search_energy, energy_total);
            let detected = rng.gen_bool(pd.clamp(0.0, 1.0));

            if detected {
                counter.hits += 1;
                counter.misses = 0;
            } else {
                counter.misses += 1;
                counter.hits = 0;
            }
        }
    }

    // Pass 2: Entities with TrackInfo (already tracked — update quality)
    {
        let mut query = world.query::<(&Position, &RadarCrossSection, &mut TrackInfo)>();
        for (_entity, (pos, rcs, track)) in query.iter() {
            let bearing = own_pos.bearing_to(pos);

            if !in_sector(bearing, sector_center, sector_width) {
                continue;
            }
            if !in_beam(bearing, sweep_angle) {
                continue;
            }

            let range = own_pos.range_to(pos);

            if range > RADAR_MAX_RANGE {
                track.misses += 1;
                track.hits = 0;
                track.quality = (track.quality - TRACK_QUALITY_MISS_LOSS).max(0.0);
                continue;
            }

            let pd = compute_pd(range, rcs.base_rcs_m2, search_energy, energy_total);
            let detected = rng.gen_bool(pd.clamp(0.0, 1.0));

            if detected {
                track.hits += 1;
                track.misses = 0;
                track.quality = (track.quality + TRACK_QUALITY_HIT_GAIN).min(TRACK_QUALITY_MAX);
            } else {
                track.misses += 1;
                track.hits = 0;
                track.quality = (track.quality - TRACK_QUALITY_MISS_LOSS).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pd_fourth_root_law() {
        let range = 100_000.0;
        let pd_base = compute_pd(range, 1.0, 100.0, 100.0);
        // Doubling RCS at range * 2^(1/4) should give the same Pd
        let extended_range = range * 2.0_f64.powf(0.25);
        let pd_doubled_rcs = compute_pd(extended_range, 2.0, 100.0, 100.0);
        assert!(
            (pd_base - pd_doubled_rcs).abs() < 0.01,
            "Fourth-root law: pd_base={pd_base}, pd_doubled_rcs={pd_doubled_rcs}"
        );
    }

    #[test]
    fn test_pd_vs_range() {
        let pd_close = compute_pd(50_000.0, 1.0, 100.0, 100.0);
        let pd_mid = compute_pd(200_000.0, 1.0, 100.0, 100.0);
        let pd_far = compute_pd(350_000.0, 1.0, 100.0, 100.0);
        assert!(pd_close > pd_mid, "Close should have higher Pd than mid");
        assert!(pd_mid > pd_far, "Mid should have higher Pd than far");
        assert!(
            pd_close > 0.99,
            "Very close should be near certain: {pd_close}"
        );
        // At max range for 1m² target, SNR≈1.0 so Pd≈0.63 by design.
        // Sea-skimmer (0.1m²) at same range would be much lower.
        let pd_sea_skimmer_far = compute_pd(350_000.0, 0.1, 100.0, 100.0);
        assert!(
            pd_sea_skimmer_far < 0.15,
            "Sea skimmer at far range should be hard to detect: {pd_sea_skimmer_far}"
        );
    }

    #[test]
    fn test_pd_vs_rcs() {
        let pd_small = compute_pd(200_000.0, 0.1, 100.0, 100.0);
        let pd_large = compute_pd(200_000.0, 3.0, 100.0, 100.0);
        assert!(
            pd_large > pd_small,
            "Larger RCS should have higher Pd: large={pd_large}, small={pd_small}"
        );
    }

    #[test]
    fn test_pd_vs_search_energy() {
        let pd_low = compute_pd(200_000.0, 1.0, 20.0, 100.0);
        let pd_high = compute_pd(200_000.0, 1.0, 80.0, 100.0);
        assert!(
            pd_high > pd_low,
            "More search energy should give higher Pd: high={pd_high}, low={pd_low}"
        );
    }

    #[test]
    fn test_pd_minimum_range_guaranteed() {
        let pd = compute_pd(100.0, 0.01, 10.0, 100.0);
        assert!(
            (pd - 1.0).abs() < 0.001,
            "Below minimum range should be guaranteed: {pd}"
        );
    }
}
