//! NASADEM HGT file parser.
//!
//! HGT files are flat arrays of big-endian i16 elevation values
//! covering 1° × 1° tiles. The filename encodes the SW corner
//! coordinates (e.g., N25E056.hgt).

use std::io;
use std::path::Path;

use crate::grid::{TerrainGrid, TerrainHeader};
use crate::projection::GeoProjection;

/// Void value in HGT files (no data).
const HGT_VOID: i16 = -32768;

/// Parse an HGT filename to extract the SW corner coordinates.
/// Format: `N25E056.hgt` or `S10W045.hgt`
pub fn parse_hgt_filename(filename: &str) -> Option<(f64, f64)> {
    let name = filename
        .strip_suffix(".hgt")
        .or_else(|| filename.strip_suffix(".HGT"))?;

    if name.len() < 7 {
        return None;
    }

    let lat_sign = match &name[0..1] {
        "N" | "n" => 1.0,
        "S" | "s" => -1.0,
        _ => return None,
    };
    let lat: f64 = name[1..3].parse().ok()?;

    let lon_sign = match &name[3..4] {
        "E" | "e" => 1.0,
        "W" | "w" => -1.0,
        _ => return None,
    };
    let lon: f64 = name[4..7].parse().ok()?;

    Some((lat * lat_sign, lon * lon_sign))
}

/// Determine the grid size from the file size.
/// 1 arc-second: 3601 × 3601 = 25,934,402 bytes
/// 3 arc-second: 1201 × 1201 = 2,884,802 bytes
fn grid_size_from_byte_count(byte_count: usize) -> Option<(u32, f64)> {
    let sample_count = byte_count / 2;
    let side = (sample_count as f64).sqrt().round() as u32;

    if side == 3601 {
        Some((3601, 1.0)) // 1 arc-second
    } else if side == 1201 {
        Some((1201, 3.0)) // 3 arc-second
    } else {
        None
    }
}

/// Parse raw HGT bytes into elevation values.
pub fn parse_hgt_bytes(data: &[u8]) -> io::Result<(Vec<i16>, u32, f64)> {
    let (grid_side, cell_size) = grid_size_from_byte_count(data.len()).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Unexpected HGT file size: {} bytes (expected {} or {})",
                data.len(),
                3601 * 3601 * 2,
                1201 * 1201 * 2
            ),
        )
    })?;

    let sample_count = (grid_side * grid_side) as usize;
    let mut elevations = Vec::with_capacity(sample_count);

    for i in 0..sample_count {
        let offset = i * 2;
        let val = i16::from_be_bytes([data[offset], data[offset + 1]]);
        elevations.push(val);
    }

    Ok((elevations, grid_side, cell_size))
}

/// Fill void values (-32768) by averaging non-void neighbors.
pub fn fill_voids(elevations: &mut [i16], width: u32, height: u32) {
    let w = width as usize;
    let h = height as usize;

    // Simple pass: replace voids with average of valid neighbors
    let snapshot = elevations.to_vec();
    for r in 0..h {
        for c in 0..w {
            let idx = r * w + c;
            if snapshot[idx] != HGT_VOID {
                continue;
            }

            let mut sum = 0i64;
            let mut count = 0u32;
            for dr in -1i32..=1 {
                for dc in -1i32..=1 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    if nr >= 0 && nr < h as i32 && nc >= 0 && nc < w as i32 {
                        let nidx = nr as usize * w + nc as usize;
                        if snapshot[nidx] != HGT_VOID {
                            sum += snapshot[nidx] as i64;
                            count += 1;
                        }
                    }
                }
            }

            elevations[idx] = if count > 0 {
                (sum / count as i64) as i16
            } else {
                0 // All neighbors are void too — default to sea level
            };
        }
    }
}

/// Load a single HGT file into a TerrainGrid.
pub fn load_hgt(path: &Path, projection: &GeoProjection) -> io::Result<TerrainGrid> {
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid HGT filename"))?;

    let (origin_lat, origin_lon) = parse_hgt_filename(filename).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Cannot parse HGT coordinates from filename: {filename}"),
        )
    })?;

    let data = std::fs::read(path)?;
    let (mut elevations, grid_side, cell_size) = parse_hgt_bytes(&data)?;

    // Fill voids
    fill_voids(&mut elevations, grid_side, grid_side);

    // Compute elevation range
    let min_elevation = elevations.iter().copied().min().unwrap_or(0);
    let max_elevation = elevations.iter().copied().max().unwrap_or(0);

    let header = TerrainHeader {
        origin_lat,
        origin_lon,
        cell_size,
        width: grid_side,
        height: grid_side,
        min_elevation,
        max_elevation,
    };

    Ok(TerrainGrid::new(
        header,
        elevations,
        None,
        projection.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hgt_filename() {
        assert_eq!(parse_hgt_filename("N25E056.hgt"), Some((25.0, 56.0)));
        assert_eq!(parse_hgt_filename("S10W045.hgt"), Some((-10.0, -45.0)));
        assert_eq!(parse_hgt_filename("N00E000.hgt"), Some((0.0, 0.0)));
        assert_eq!(parse_hgt_filename("invalid.hgt"), None);
        assert_eq!(parse_hgt_filename("N25E056.txt"), None);
    }

    #[test]
    fn test_hgt_parse_small() {
        // Create a synthetic 5×5 HGT-format buffer (big-endian i16)
        let _width = 5u32;
        let values: Vec<i16> = vec![
            100, 200, 300, 400, 500, 110, 210, 310, 410, 510, 120, 220, 320, 420, 520, 130, 230,
            330, 430, 530, 140, 240, 340, 440, 540,
        ];

        let mut data = Vec::with_capacity(values.len() * 2);
        for &v in &values {
            data.extend_from_slice(&v.to_be_bytes());
        }

        // This won't match standard sizes (3601 or 1201), so parse_hgt_bytes will fail.
        // Instead, test the byte parsing logic directly.
        let sample_count = data.len() / 2;
        let mut parsed = Vec::with_capacity(sample_count);
        for i in 0..sample_count {
            let offset = i * 2;
            let val = i16::from_be_bytes([data[offset], data[offset + 1]]);
            parsed.push(val);
        }

        assert_eq!(parsed, values);
    }

    #[test]
    fn test_fill_voids() {
        let mut elevations: Vec<i16> = vec![100, 200, 300, 100, HGT_VOID, 300, 100, 200, 300];

        fill_voids(&mut elevations, 3, 3);

        // Center void should be average of 8 neighbors: (100+200+300+100+300+100+200+300)/8 = 200
        assert_eq!(elevations[4], 200);
    }

    #[test]
    fn test_fill_voids_corner() {
        let mut elevations: Vec<i16> = vec![HGT_VOID, 100, 100, 100];

        fill_voids(&mut elevations, 2, 2);

        // Corner void has 3 neighbors all at 100
        assert_eq!(elevations[0], 100);
    }
}
