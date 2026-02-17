//! TerrainGrid: loaded heightmap with elevation queries.

use deterrence_core::types::Position;

use crate::projection::GeoProjection;

/// Terrain grid header metadata.
#[derive(Debug, Clone)]
pub struct TerrainHeader {
    /// Southwest corner latitude (degrees).
    pub origin_lat: f64,
    /// Southwest corner longitude (degrees).
    pub origin_lon: f64,
    /// Arc-seconds per grid cell.
    pub cell_size: f64,
    /// Number of columns (west to east).
    pub width: u32,
    /// Number of rows (north to south).
    pub height: u32,
    /// Minimum elevation in the grid (meters).
    pub min_elevation: i16,
    /// Maximum elevation in the grid (meters).
    pub max_elevation: i16,
}

impl TerrainHeader {
    /// North edge latitude (degrees).
    pub fn north_lat(&self) -> f64 {
        self.origin_lat + (self.height as f64 * self.cell_size) / 3600.0
    }

    /// East edge longitude (degrees).
    pub fn east_lon(&self) -> f64 {
        self.origin_lon + (self.width as f64 * self.cell_size) / 3600.0
    }
}

/// Loaded terrain heightmap grid with geographic projection.
#[derive(Debug, Clone)]
pub struct TerrainGrid {
    pub header: TerrainHeader,
    /// Elevation values in meters, row-major (north-to-south, west-to-east).
    pub elevations: Vec<i16>,
    /// Packed ocean mask: bit 1 = land, bit 0 = ocean. One bit per cell.
    pub ocean_mask: Option<Vec<u8>>,
    /// Projection for converting sim-space to lat/lon.
    projection: GeoProjection,
}

impl TerrainGrid {
    /// Create a TerrainGrid from pre-loaded data.
    pub fn new(
        header: TerrainHeader,
        elevations: Vec<i16>,
        ocean_mask: Option<Vec<u8>>,
        projection: GeoProjection,
    ) -> Self {
        Self {
            header,
            elevations,
            ocean_mask,
            projection,
        }
    }

    /// Reference to the geo-projection used by this grid.
    pub fn projection(&self) -> &GeoProjection {
        &self.projection
    }

    /// Convert a sim-space position to grid row/col (fractional).
    /// Returns None if outside grid bounds.
    fn sim_to_grid(&self, pos: &Position) -> Option<(f64, f64)> {
        let (lat, lon, _) = self.projection.to_geo(pos);
        self.geo_to_grid(lat, lon)
    }

    /// Convert lat/lon to grid row/col (fractional).
    fn geo_to_grid(&self, lat: f64, lon: f64) -> Option<(f64, f64)> {
        let h = &self.header;

        // Column: west-to-east
        let col = (lon - h.origin_lon) * 3600.0 / h.cell_size;
        // Row: north-to-south (row 0 = north edge)
        let row = (h.north_lat() - lat) * 3600.0 / h.cell_size;

        if col < 0.0 || row < 0.0 || col >= h.width as f64 || row >= h.height as f64 {
            return None;
        }

        Some((row, col))
    }

    /// Get raw elevation at integer grid coordinates.
    fn raw_elevation(&self, row: usize, col: usize) -> i16 {
        let h = &self.header;
        if row >= h.height as usize || col >= h.width as usize {
            return 0;
        }
        self.elevations[row * h.width as usize + col]
    }

    /// Elevation at a sim-space position with bilinear interpolation.
    /// Returns None if the position is outside the grid.
    pub fn elevation_at(&self, pos: &Position) -> Option<f32> {
        let (row, col) = self.sim_to_grid(pos)?;
        Some(self.bilinear(row, col))
    }

    /// Elevation at lat/lon with bilinear interpolation.
    pub fn elevation_at_geo(&self, lat: f64, lon: f64) -> Option<f32> {
        let (row, col) = self.geo_to_grid(lat, lon)?;
        Some(self.bilinear(row, col))
    }

    /// Bilinear interpolation at fractional row/col.
    fn bilinear(&self, row: f64, col: f64) -> f32 {
        let r0 = row.floor() as usize;
        let c0 = col.floor() as usize;
        let r1 = (r0 + 1).min(self.header.height as usize - 1);
        let c1 = (c0 + 1).min(self.header.width as usize - 1);

        let fr = row - r0 as f64;
        let fc = col - c0 as f64;

        let e00 = self.raw_elevation(r0, c0) as f64;
        let e01 = self.raw_elevation(r0, c1) as f64;
        let e10 = self.raw_elevation(r1, c0) as f64;
        let e11 = self.raw_elevation(r1, c1) as f64;

        let top = e00 * (1.0 - fc) + e01 * fc;
        let bot = e10 * (1.0 - fc) + e11 * fc;
        let val = top * (1.0 - fr) + bot * fr;

        val as f32
    }

    /// Check if position is over ocean (using ocean mask or elevation <= 0).
    pub fn is_ocean(&self, pos: &Position) -> bool {
        if let Some(ref mask) = self.ocean_mask {
            if let Some((row, col)) = self.sim_to_grid(pos) {
                let r = row.round() as usize;
                let c = col.round() as usize;
                let idx = r * self.header.width as usize + c;
                let byte_idx = idx / 8;
                let bit_idx = idx % 8;
                if byte_idx < mask.len() {
                    return mask[byte_idx] & (1 << bit_idx) == 0; // 0 = ocean
                }
            }
            return true; // outside grid = ocean
        }
        // No mask: use elevation
        self.elevation_at(pos).is_none_or(|e| e <= 0.0)
    }

    /// Downsample the elevation grid to a target resolution.
    /// Returns a new flat Vec<i16> of size target_width * target_height.
    pub fn downsample(&self, target_width: u32, target_height: u32) -> Vec<i16> {
        let h = &self.header;
        let mut result = Vec::with_capacity((target_width * target_height) as usize);

        for tr in 0..target_height {
            for tc in 0..target_width {
                // Map target cell to source cell
                let sr = (tr as f64 / target_height as f64) * h.height as f64;
                let sc = (tc as f64 / target_width as f64) * h.width as f64;
                result.push(self.bilinear(sr, sc) as i16);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a simple 5×5 test grid centered at (26.5, 56.2).
    fn make_test_grid() -> TerrainGrid {
        let proj = GeoProjection::new(26.5, 56.2);
        // 5×5 grid, 1 arc-second per cell
        // Grid spans from (26.4986, 56.1986) to (26.5014, 56.2014) approx
        let cell_size = 1.0; // 1 arc-second
        let width = 5u32;
        let height = 5u32;
        let origin_lat = 26.5 - (height as f64 * cell_size / 3600.0) / 2.0;
        let origin_lon = 56.2 - (width as f64 * cell_size / 3600.0) / 2.0;

        // Elevation: center cell is 100m, edges are 0
        #[rustfmt::skip]
        let elevations: Vec<i16> = vec![
            0,   0,   0,   0,   0,
            0,  50,  50,  50,   0,
            0,  50, 100,  50,   0,
            0,  50,  50,  50,   0,
            0,   0,   0,   0,   0,
        ];

        TerrainGrid::new(
            TerrainHeader {
                origin_lat,
                origin_lon,
                cell_size,
                width,
                height,
                min_elevation: 0,
                max_elevation: 100,
            },
            elevations,
            None,
            proj,
        )
    }

    #[test]
    fn test_elevation_query_center() {
        let grid = make_test_grid();
        let h = &grid.header;

        // The peak cell is at grid row=2, col=2 (0-indexed).
        // Convert that to lat/lon, then to sim-space.
        let peak_lat = h.north_lat() - 2.0 * h.cell_size / 3600.0;
        let peak_lon = h.origin_lon + 2.0 * h.cell_size / 3600.0;
        let pos = grid.projection().to_sim(peak_lat, peak_lon, 0.0);

        let elev = grid.elevation_at(&pos);
        assert!(elev.is_some(), "Peak cell should be within grid");
        let e = elev.unwrap();
        assert!(
            (e - 100.0).abs() < 1.0,
            "Peak elevation should be ~100m, got {e}"
        );
    }

    #[test]
    fn test_elevation_query_edges() {
        let grid = make_test_grid();

        // Far outside the grid (1 degree away)
        let far = Position::new(111_320.0, 0.0, 0.0);
        assert!(
            grid.elevation_at(&far).is_none(),
            "Far position should be outside grid"
        );
    }

    #[test]
    fn test_elevation_bilinear_interpolation() {
        let grid = make_test_grid();
        // A point between center (100m) and edge (50m) should interpolate
        let h = &grid.header;

        // Position at row=1.5, col=2 (between rows 1 and 2, center column)
        // Row 1 col 2 = 50, Row 2 col 2 = 100 → interpolated = 75
        let lat = h.north_lat() - 1.5 * h.cell_size / 3600.0;
        let lon = h.origin_lon + 2.0 * h.cell_size / 3600.0;
        let pos = grid.projection.to_sim(lat, lon, 0.0);
        let elev = grid.elevation_at(&pos).unwrap();
        assert!(
            (elev - 75.0).abs() < 1.0,
            "Interpolated elevation should be ~75m, got {elev}"
        );
    }

    #[test]
    fn test_is_ocean_no_mask() {
        let grid = make_test_grid();
        // Center has elevation 100 → not ocean
        let center = Position::new(0.0, 0.0, 0.0);
        assert!(!grid.is_ocean(&center), "Center (100m) should not be ocean");

        // Edge has elevation 0 → ocean
        let h = &grid.header;
        let edge_lat = h.origin_lat + 0.1 * h.cell_size / 3600.0;
        let edge_lon = h.origin_lon + 0.1 * h.cell_size / 3600.0;
        let edge = grid.projection.to_sim(edge_lat, edge_lon, 0.0);
        assert!(grid.is_ocean(&edge), "Edge (0m) should be ocean");
    }

    #[test]
    fn test_downsample() {
        let grid = make_test_grid();
        let ds = grid.downsample(3, 3);
        assert_eq!(ds.len(), 9, "3×3 downsample should have 9 values");
        // Center of downsampled grid should approximate center of original
        assert!(ds[4] > 50, "Center of downsample should be elevated");
    }
}
