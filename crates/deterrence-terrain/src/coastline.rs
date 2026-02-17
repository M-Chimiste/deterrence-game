//! Coastline extraction from terrain data.
//!
//! Walks the elevation grid to find land/ocean boundaries and
//! produces polyline segments in sim-space coordinates.

use crate::grid::TerrainGrid;

/// Extract coastline polyline segments from a terrain grid.
///
/// Returns a vector of polylines, where each polyline is a vector of [x, y] points
/// in sim-space coordinates (meters). Coastlines are defined as boundaries between
/// cells where elevation transitions from <= 0 (ocean) to > 0 (land) or vice versa.
pub fn extract_coastlines(grid: &TerrainGrid) -> Vec<Vec<[f64; 2]>> {
    let h = &grid.header;
    let w = h.width as usize;
    let ht = h.height as usize;

    if w < 2 || ht < 2 {
        return Vec::new();
    }

    let proj = grid.projection();

    // Collect boundary crossing points
    let mut points: Vec<[f64; 2]> = Vec::new();

    for r in 0..ht - 1 {
        for c in 0..w - 1 {
            let e00 = grid.elevations[r * w + c];
            let e01 = grid.elevations[r * w + c + 1];
            let e10 = grid.elevations[(r + 1) * w + c];

            let l00 = e00 > 0;
            let l01 = e01 > 0;
            let l10 = e10 > 0;

            // Horizontal edge: between (r,c) and (r,c+1)
            if l00 != l01 {
                let lat = h.north_lat() - r as f64 * h.cell_size / 3600.0;
                let lon1 = h.origin_lon + c as f64 * h.cell_size / 3600.0;
                let lon2 = h.origin_lon + (c + 1) as f64 * h.cell_size / 3600.0;

                let e1 = e00 as f64;
                let e2 = e01 as f64;
                let t = if (e2 - e1).abs() > 0.01 {
                    (-e1 / (e2 - e1)).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                let lon_mid = lon1 + t * (lon2 - lon1);
                let p = proj.to_sim(lat, lon_mid, 0.0);
                points.push([p.x, p.y]);
            }

            // Vertical edge: between (r,c) and (r+1,c)
            if l00 != l10 {
                let lat1 = h.north_lat() - r as f64 * h.cell_size / 3600.0;
                let lat2 = h.north_lat() - (r + 1) as f64 * h.cell_size / 3600.0;
                let lon = h.origin_lon + c as f64 * h.cell_size / 3600.0;

                let e1 = e00 as f64;
                let e2 = e10 as f64;
                let t = if (e2 - e1).abs() > 0.01 {
                    (-e1 / (e2 - e1)).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                let lat_mid = lat1 + t * (lat2 - lat1);
                let p = proj.to_sim(lat_mid, lon, 0.0);
                points.push([p.x, p.y]);
            }
        }
    }

    // Also check right column vertical edges and bottom row horizontal edges
    for r in 0..ht - 1 {
        let c = w - 1;
        if c > 0 {
            let e_this = grid.elevations[r * w + c];
            let e_below = grid.elevations[(r + 1) * w + c];
            if (e_this > 0) != (e_below > 0) {
                let lat1 = h.north_lat() - r as f64 * h.cell_size / 3600.0;
                let lat2 = h.north_lat() - (r + 1) as f64 * h.cell_size / 3600.0;
                let lon = h.origin_lon + c as f64 * h.cell_size / 3600.0;
                let e1 = e_this as f64;
                let e2 = e_below as f64;
                let t = if (e2 - e1).abs() > 0.01 {
                    (-e1 / (e2 - e1)).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                let lat_mid = lat1 + t * (lat2 - lat1);
                let p = proj.to_sim(lat_mid, lon, 0.0);
                points.push([p.x, p.y]);
            }
        }
    }

    for c in 0..w - 1 {
        let r = ht - 1;
        if r > 0 {
            let e_this = grid.elevations[r * w + c];
            let e_right = grid.elevations[r * w + c + 1];
            if (e_this > 0) != (e_right > 0) {
                let lat = h.north_lat() - r as f64 * h.cell_size / 3600.0;
                let lon1 = h.origin_lon + c as f64 * h.cell_size / 3600.0;
                let lon2 = h.origin_lon + (c + 1) as f64 * h.cell_size / 3600.0;
                let e1 = e_this as f64;
                let e2 = e_right as f64;
                let t = if (e2 - e1).abs() > 0.01 {
                    (-e1 / (e2 - e1)).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                let lon_mid = lon1 + t * (lon2 - lon1);
                let p = proj.to_sim(lat, lon_mid, 0.0);
                points.push([p.x, p.y]);
            }
        }
    }

    if points.is_empty() {
        return Vec::new();
    }

    // Chain nearby points into polylines using cell-size-proportional threshold.
    // Cell size in meters ≈ cell_size_arcsec * 30.87 (at mid-latitudes)
    let cell_meters = h.cell_size * 30.87;
    let threshold = cell_meters * 2.0;
    chain_points_into_polylines(points, threshold)
}

/// Chain individual points into connected polylines by proximity.
fn chain_points_into_polylines(points: Vec<[f64; 2]>, threshold: f64) -> Vec<Vec<[f64; 2]>> {
    let threshold_sq = threshold * threshold;
    let mut used = vec![false; points.len()];
    let mut polylines = Vec::new();

    for start in 0..points.len() {
        if used[start] {
            continue;
        }

        let mut polyline = vec![points[start]];
        used[start] = true;

        // Greedily extend the polyline by finding the nearest unused point
        loop {
            let last = *polyline.last().unwrap();
            let mut best_idx = None;
            let mut best_dist = threshold_sq;

            for (i, pt) in points.iter().enumerate() {
                if used[i] {
                    continue;
                }
                let dx = pt[0] - last[0];
                let dy = pt[1] - last[1];
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < best_dist {
                    best_dist = dist_sq;
                    best_idx = Some(i);
                }
            }

            if let Some(idx) = best_idx {
                polyline.push(points[idx]);
                used[idx] = true;
            } else {
                break;
            }
        }

        if polyline.len() >= 2 {
            polylines.push(polyline);
        }
    }

    polylines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::TerrainHeader;
    use crate::projection::GeoProjection;

    #[test]
    fn test_coastline_extraction() {
        let proj = GeoProjection::new(26.5, 56.2);
        let cell_size = 30.0; // 30 arc-seconds
        let width = 10u32;
        let height = 10u32;
        let origin_lat = 26.5 - (height as f64 * cell_size / 3600.0) / 2.0;
        let origin_lon = 56.2 - (width as f64 * cell_size / 3600.0) / 2.0;

        // Create an "island" grid: ocean around edges, land in center
        let mut elevations = vec![0i16; (width * height) as usize];
        for r in 3..7 {
            for c in 3..7 {
                elevations[r * width as usize + c] = 100;
            }
        }

        let grid = TerrainGrid::new(
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
        );

        let coastlines = extract_coastlines(&grid);

        // Should have at least one polyline
        assert!(
            !coastlines.is_empty(),
            "Island should produce coastline polylines"
        );

        // Total coastline points should be reasonable (perimeter of 4x4 island)
        let total_points: usize = coastlines.iter().map(|p| p.len()).sum();
        assert!(
            total_points >= 4,
            "Coastline should have at least 4 points, got {total_points}"
        );
    }

    #[test]
    fn test_coastline_all_ocean() {
        let proj = GeoProjection::new(26.5, 56.2);
        let elevations = vec![0i16; 25];
        let grid = TerrainGrid::new(
            TerrainHeader {
                origin_lat: 26.0,
                origin_lon: 56.0,
                cell_size: 30.0,
                width: 5,
                height: 5,
                min_elevation: 0,
                max_elevation: 0,
            },
            elevations,
            None,
            proj,
        );

        let coastlines = extract_coastlines(&grid);
        assert!(
            coastlines.is_empty(),
            "All-ocean grid should have no coastlines"
        );
    }

    #[test]
    fn test_coastline_all_land() {
        let proj = GeoProjection::new(26.5, 56.2);
        let elevations = vec![100i16; 25];
        let grid = TerrainGrid::new(
            TerrainHeader {
                origin_lat: 26.0,
                origin_lon: 56.0,
                cell_size: 30.0,
                width: 5,
                height: 5,
                min_elevation: 100,
                max_elevation: 100,
            },
            elevations,
            None,
            proj,
        );

        let coastlines = extract_coastlines(&grid);
        assert!(
            coastlines.is_empty(),
            "All-land grid should have no coastlines"
        );
    }
}
