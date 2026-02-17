//! Line-of-sight calculation with terrain occlusion.
//!
//! Uses stepped ray traversal with Earth curvature correction
//! and standard atmospheric refraction (4/3 Earth radius model).

use deterrence_core::types::Position;

use crate::grid::TerrainGrid;

/// Effective Earth radius accounting for standard atmospheric refraction (4/3 model).
const EFFECTIVE_EARTH_RADIUS: f64 = 6_371_000.0 * 4.0 / 3.0;

/// Default sample interval for LOS checks (meters).
const LOS_SAMPLE_INTERVAL: f64 = 100.0;

/// Check line-of-sight between two sim-space points, accounting for terrain and Earth curvature.
///
/// Returns true if there is clear LOS between `from` and `to`.
/// The check steps along the straight-line path and compares the LOS ray height
/// against terrain elevation at each step. Earth curvature causes the ground to
/// "drop away" from the straight-line ray.
pub fn has_line_of_sight(grid: &TerrainGrid, from: &Position, to: &Position) -> bool {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let dz = to.z - from.z;
    let horiz_dist = (dx * dx + dy * dy).sqrt();

    if horiz_dist < LOS_SAMPLE_INTERVAL {
        return true; // Too close for terrain to matter
    }

    let num_samples = (horiz_dist / LOS_SAMPLE_INTERVAL).ceil() as usize;
    let num_samples = num_samples.max(2);

    for i in 1..num_samples {
        let t = i as f64 / num_samples as f64;

        // Position along the straight-line ray
        let sample_x = from.x + dx * t;
        let sample_y = from.y + dy * t;

        // Height of the LOS ray at this point (linear interpolation in height)
        let ray_height = from.z + dz * t;

        // Earth curvature correction: ground drops away from the ray
        let d_from = horiz_dist * t;
        let d_to = horiz_dist * (1.0 - t);
        // Curvature drop relative to the straight-line ray between the two endpoints
        let earth_drop = (d_from * d_to) / (2.0 * EFFECTIVE_EARTH_RADIUS);

        // Query terrain elevation at this sample point
        let sample_pos = Position::new(sample_x, sample_y, 0.0);
        let terrain_elev = grid.elevation_at(&sample_pos).unwrap_or(0.0) as f64;

        // The effective terrain height relative to the LOS ray
        // Earth curvature causes the terrain to be lower than it would be on a flat earth
        let effective_terrain = terrain_elev - earth_drop;

        if effective_terrain > ray_height {
            return false; // Terrain blocks LOS
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::TerrainHeader;
    use crate::projection::GeoProjection;

    /// Create a flat terrain grid (all elevation = 0).
    fn make_flat_grid() -> TerrainGrid {
        let proj = GeoProjection::new(26.5, 56.2);
        let cell_size = 3.0; // 3 arc-seconds (~90m)
        let width = 100u32;
        let height = 100u32;
        let origin_lat = 26.5 - (height as f64 * cell_size / 3600.0) / 2.0;
        let origin_lon = 56.2 - (width as f64 * cell_size / 3600.0) / 2.0;

        let elevations = vec![0i16; (width * height) as usize];

        TerrainGrid::new(
            TerrainHeader {
                origin_lat,
                origin_lon,
                cell_size,
                width,
                height,
                min_elevation: 0,
                max_elevation: 0,
            },
            elevations,
            None,
            proj,
        )
    }

    /// Create a grid with a hill in the center.
    fn make_hill_grid() -> TerrainGrid {
        let proj = GeoProjection::new(26.5, 56.2);
        let cell_size = 3.0;
        let width = 100u32;
        let height = 100u32;
        let origin_lat = 26.5 - (height as f64 * cell_size / 3600.0) / 2.0;
        let origin_lon = 56.2 - (width as f64 * cell_size / 3600.0) / 2.0;

        let mut elevations = vec![0i16; (width * height) as usize];

        // Place a 500m hill in the center (rows 45-55, cols 45-55)
        for r in 45..55 {
            for c in 45..55 {
                elevations[r * width as usize + c] = 500;
            }
        }

        TerrainGrid::new(
            TerrainHeader {
                origin_lat,
                origin_lon,
                cell_size,
                width,
                height,
                min_elevation: 0,
                max_elevation: 500,
            },
            elevations,
            None,
            proj,
        )
    }

    #[test]
    fn test_los_flat_terrain() {
        let grid = make_flat_grid();

        // Two points at 100m altitude on flat terrain — clear LOS
        let from = Position::new(0.0, -5000.0, 100.0);
        let to = Position::new(0.0, 5000.0, 100.0);

        assert!(
            has_line_of_sight(&grid, &from, &to),
            "LOS should be clear on flat terrain"
        );
    }

    #[test]
    fn test_los_blocked_by_hill() {
        let grid = make_hill_grid();

        // Two points at 10m altitude on opposite sides of the hill — blocked
        let from = Position::new(0.0, -5000.0, 10.0);
        let to = Position::new(0.0, 5000.0, 10.0);

        assert!(
            !has_line_of_sight(&grid, &from, &to),
            "LOS should be blocked by the 500m hill"
        );
    }

    #[test]
    fn test_los_over_hill() {
        let grid = make_hill_grid();

        // Two points at 1000m altitude — high enough to see over the 500m hill
        let from = Position::new(0.0, -5000.0, 1000.0);
        let to = Position::new(0.0, 5000.0, 1000.0);

        assert!(
            has_line_of_sight(&grid, &from, &to),
            "LOS should be clear at 1000m over 500m hill"
        );
    }

    #[test]
    fn test_los_close_range() {
        let grid = make_hill_grid();

        // Very close positions — always clear
        let from = Position::new(0.0, 0.0, 10.0);
        let to = Position::new(50.0, 50.0, 10.0);

        assert!(
            has_line_of_sight(&grid, &from, &to),
            "LOS should be clear at very close range"
        );
    }
}
