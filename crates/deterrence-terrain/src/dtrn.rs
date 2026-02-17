//! .dtrn binary format loader and writer.
//!
//! Custom compact terrain format for fast loading.
//! See terrain_data_guide.md for full specification.

use std::io::{self, Write};
use std::path::Path;

use crate::grid::{TerrainGrid, TerrainHeader};
use crate::projection::GeoProjection;

/// .dtrn magic bytes.
const DTRN_MAGIC: [u8; 4] = *b"DTRN";

/// Current format version.
const DTRN_VERSION: u16 = 1;

/// Header flag: has ocean mask.
const FLAG_HAS_OCEAN_MASK: u16 = 0x0001;

/// Header flag: has texture index (reserved for future use).
#[allow(dead_code)]
const FLAG_HAS_TEXTURE: u16 = 0x0002;

/// Total header size in bytes.
const HEADER_SIZE: usize = 64;

/// Load a TerrainGrid from a .dtrn file.
pub fn load_dtrn(path: &Path, projection: &GeoProjection) -> io::Result<TerrainGrid> {
    let data = std::fs::read(path)?;
    parse_dtrn(&data, projection)
}

/// Parse a .dtrn from a byte buffer.
pub fn parse_dtrn(data: &[u8], projection: &GeoProjection) -> io::Result<TerrainGrid> {
    if data.len() < HEADER_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too small for .dtrn header",
        ));
    }

    // Parse header
    let magic = &data[0..4];
    if magic != DTRN_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid .dtrn magic bytes",
        ));
    }

    let version = u16::from_le_bytes([data[4], data[5]]);
    if version != DTRN_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported .dtrn version: {version}"),
        ));
    }

    let flags = u16::from_le_bytes([data[6], data[7]]);
    let origin_lat = f64::from_le_bytes(data[8..16].try_into().unwrap());
    let origin_lon = f64::from_le_bytes(data[16..24].try_into().unwrap());
    let cell_size = f64::from_le_bytes(data[24..32].try_into().unwrap());
    let width = u32::from_le_bytes(data[32..36].try_into().unwrap());
    let height = u32::from_le_bytes(data[36..40].try_into().unwrap());
    let min_elevation = i16::from_le_bytes([data[40], data[41]]);
    let max_elevation = i16::from_le_bytes([data[42], data[43]]);
    // Bytes 44..64 are reserved

    let cell_count = (width as usize) * (height as usize);
    let elev_size = cell_count * 2;
    let elev_start = HEADER_SIZE;
    let elev_end = elev_start + elev_size;

    if data.len() < elev_end {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too small for elevation data",
        ));
    }

    // Parse elevations (big-endian i16 matching HGT convention)
    let mut elevations = Vec::with_capacity(cell_count);
    for i in 0..cell_count {
        let offset = elev_start + i * 2;
        let val = i16::from_be_bytes([data[offset], data[offset + 1]]);
        elevations.push(val);
    }

    // Parse optional ocean mask
    let ocean_mask = if flags & FLAG_HAS_OCEAN_MASK != 0 {
        let mask_start = elev_end;
        let mask_bytes = cell_count.div_ceil(8);
        let mask_end = mask_start + mask_bytes;
        if data.len() >= mask_end {
            Some(data[mask_start..mask_end].to_vec())
        } else {
            None
        }
    } else {
        None
    };

    let header = TerrainHeader {
        origin_lat,
        origin_lon,
        cell_size,
        width,
        height,
        min_elevation,
        max_elevation,
    };

    Ok(TerrainGrid::new(
        header,
        elevations,
        ocean_mask,
        projection.clone(),
    ))
}

/// Write a TerrainGrid to a .dtrn file.
pub fn write_dtrn(grid: &TerrainGrid, path: &Path) -> io::Result<()> {
    let data = serialize_dtrn(grid);
    std::fs::write(path, data)
}

/// Serialize a TerrainGrid to .dtrn bytes.
pub fn serialize_dtrn(grid: &TerrainGrid) -> Vec<u8> {
    let h = &grid.header;
    let cell_count = (h.width as usize) * (h.height as usize);

    let mut flags: u16 = 0;
    if grid.ocean_mask.is_some() {
        flags |= FLAG_HAS_OCEAN_MASK;
    }

    let mask_bytes = if grid.ocean_mask.is_some() {
        cell_count.div_ceil(8)
    } else {
        0
    };
    let total_size = HEADER_SIZE + cell_count * 2 + mask_bytes;
    let mut buf = Vec::with_capacity(total_size);

    // Header (64 bytes)
    buf.write_all(&DTRN_MAGIC).unwrap();
    buf.write_all(&DTRN_VERSION.to_le_bytes()).unwrap();
    buf.write_all(&flags.to_le_bytes()).unwrap();
    buf.write_all(&h.origin_lat.to_le_bytes()).unwrap();
    buf.write_all(&h.origin_lon.to_le_bytes()).unwrap();
    buf.write_all(&h.cell_size.to_le_bytes()).unwrap();
    buf.write_all(&h.width.to_le_bytes()).unwrap();
    buf.write_all(&h.height.to_le_bytes()).unwrap();
    buf.write_all(&h.min_elevation.to_le_bytes()).unwrap();
    buf.write_all(&h.max_elevation.to_le_bytes()).unwrap();
    // Reserved bytes (pad to 64)
    let written = 4 + 2 + 2 + 8 + 8 + 8 + 4 + 4 + 2 + 2; // = 44
    buf.write_all(&vec![0u8; HEADER_SIZE - written]).unwrap();

    // Elevation data (big-endian i16)
    for &elev in &grid.elevations {
        buf.write_all(&elev.to_be_bytes()).unwrap();
    }

    // Ocean mask (if present)
    if let Some(ref mask) = grid.ocean_mask {
        buf.write_all(mask).unwrap();
        // Pad to expected size if needed
        if mask.len() < mask_bytes {
            buf.write_all(&vec![0u8; mask_bytes - mask.len()]).unwrap();
        }
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dtrn_roundtrip() {
        let proj = GeoProjection::new(26.5, 56.2);
        let header = TerrainHeader {
            origin_lat: 26.0,
            origin_lon: 56.0,
            cell_size: 3.0,
            width: 4,
            height: 4,
            min_elevation: -10,
            max_elevation: 500,
        };
        let elevations: Vec<i16> =
            vec![0, 10, 20, 30, 40, 50, 100, 200, 0, 0, 0, 0, -10, -5, 0, 500];

        let grid = TerrainGrid::new(header, elevations.clone(), None, proj.clone());

        // Serialize
        let bytes = serialize_dtrn(&grid);

        // Deserialize
        let grid2 = parse_dtrn(&bytes, &proj).expect("Failed to parse .dtrn");

        assert_eq!(grid2.header.width, 4);
        assert_eq!(grid2.header.height, 4);
        assert!((grid2.header.origin_lat - 26.0).abs() < 1e-10);
        assert!((grid2.header.origin_lon - 56.0).abs() < 1e-10);
        assert!((grid2.header.cell_size - 3.0).abs() < 1e-10);
        assert_eq!(grid2.header.min_elevation, -10);
        assert_eq!(grid2.header.max_elevation, 500);
        assert_eq!(grid2.elevations, elevations);
    }

    #[test]
    fn test_dtrn_with_ocean_mask() {
        let proj = GeoProjection::new(26.5, 56.2);
        let header = TerrainHeader {
            origin_lat: 26.0,
            origin_lon: 56.0,
            cell_size: 3.0,
            width: 4,
            height: 2,
            min_elevation: 0,
            max_elevation: 100,
        };
        let elevations: Vec<i16> = vec![0, 100, 0, 0, 0, 0, 50, 0];
        // Ocean mask: 8 cells = 1 byte. Cells 1 and 6 are land.
        let ocean_mask = vec![0b0100_0010u8]; // bit 1 and bit 6 set

        let grid = TerrainGrid::new(
            header,
            elevations.clone(),
            Some(ocean_mask.clone()),
            proj.clone(),
        );

        let bytes = serialize_dtrn(&grid);
        let grid2 = parse_dtrn(&bytes, &proj).expect("Failed to parse .dtrn with mask");

        assert!(grid2.ocean_mask.is_some());
        assert_eq!(grid2.ocean_mask.unwrap(), ocean_mask);
    }

    #[test]
    fn test_dtrn_invalid_magic() {
        let proj = GeoProjection::new(0.0, 0.0);
        let data = vec![0u8; 64]; // All zeros, wrong magic
        let result = parse_dtrn(&data, &proj);
        assert!(result.is_err());
    }
}
