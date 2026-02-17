//! terrain-prep: HGT → .dtrn conversion tool and synthetic terrain generator.
//!
//! Usage:
//!   terrain-prep convert --hgt N25E056.hgt --center 26.5,56.2 --output terrain.dtrn
//!   terrain-prep synthetic --center 26.5,56.2 --output hormuz_synth.dtrn

use std::path::PathBuf;
use std::process;

use deterrence_terrain::dtrn::write_dtrn;
use deterrence_terrain::grid::{TerrainGrid, TerrainHeader};
use deterrence_terrain::hgt::load_hgt;
use deterrence_terrain::GeoProjection;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "convert" => cmd_convert(&args[2..]),
        "synthetic" => cmd_synthetic(&args[2..]),
        "help" | "--help" | "-h" => print_usage(),
        other => {
            eprintln!("Unknown command: {other}");
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!(
        "terrain-prep: DETERRENCE terrain preprocessing tool\n\
         \n\
         Commands:\n\
         \n\
         convert   Convert NASADEM HGT file(s) to .dtrn format\n\
         \n\
           --hgt <path>       HGT file to convert (can specify multiple)\n\
           --center <lat,lon> Theater center coordinates\n\
           --output <path>    Output .dtrn file path\n\
           --resample <N>     Resample to NxN grid (optional, default: native)\n\
         \n\
         synthetic Generate a synthetic terrain .dtrn for testing/demo\n\
         \n\
           --center <lat,lon> Theater center coordinates\n\
           --output <path>    Output .dtrn file path\n\
           --size <N>         Grid size (default: 256)\n\
         \n\
         Examples:\n\
         \n\
           terrain-prep convert --hgt N25E056.hgt --center 26.5,56.2 --output hormuz.dtrn\n\
           terrain-prep synthetic --center 26.5,56.2 --output public/terrain/hormuz_synth.dtrn\n"
    );
}

fn parse_center(args: &[String]) -> Option<(f64, f64)> {
    for i in 0..args.len() {
        if args[i] == "--center" && i + 1 < args.len() {
            let parts: Vec<&str> = args[i + 1].split(',').collect();
            if parts.len() == 2 {
                let lat: f64 = parts[0].parse().ok()?;
                let lon: f64 = parts[1].parse().ok()?;
                return Some((lat, lon));
            }
        }
    }
    None
}

fn parse_output(args: &[String]) -> Option<PathBuf> {
    for i in 0..args.len() {
        if args[i] == "--output" && i + 1 < args.len() {
            return Some(PathBuf::from(&args[i + 1]));
        }
    }
    None
}

fn parse_hgt_paths(args: &[String]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for i in 0..args.len() {
        if args[i] == "--hgt" && i + 1 < args.len() {
            paths.push(PathBuf::from(&args[i + 1]));
        }
    }
    paths
}

fn parse_size(args: &[String], default: u32) -> u32 {
    for i in 0..args.len() {
        if (args[i] == "--size" || args[i] == "--resample") && i + 1 < args.len() {
            if let Ok(n) = args[i + 1].parse::<u32>() {
                return n;
            }
        }
    }
    default
}

// --- Convert command ---

fn cmd_convert(args: &[String]) {
    let hgt_paths = parse_hgt_paths(args);
    if hgt_paths.is_empty() {
        eprintln!("Error: --hgt <path> is required");
        process::exit(1);
    }

    let (center_lat, center_lon) = match parse_center(args) {
        Some(c) => c,
        None => {
            eprintln!("Error: --center <lat,lon> is required");
            process::exit(1);
        }
    };

    let output = match parse_output(args) {
        Some(p) => p,
        None => {
            eprintln!("Error: --output <path> is required");
            process::exit(1);
        }
    };

    let resample = parse_size(args, 0);
    let projection = GeoProjection::new(center_lat, center_lon);

    eprintln!("Theater center: {center_lat}°N, {center_lon}°E");
    eprintln!("Loading {} HGT file(s)...", hgt_paths.len());

    // Load the first HGT file (multi-tile merging is a future enhancement)
    let grid = match load_hgt(&hgt_paths[0], &projection) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Error loading HGT file: {e}");
            process::exit(1);
        }
    };

    eprintln!(
        "Loaded: {}×{} grid, cell_size={} arcsec, elevation range {}..{}m",
        grid.header.width,
        grid.header.height,
        grid.header.cell_size,
        grid.header.min_elevation,
        grid.header.max_elevation,
    );

    // Optionally resample
    let final_grid = if resample > 0 && resample < grid.header.width {
        eprintln!("Resampling to {resample}×{resample}...");
        let elevations = grid.downsample(resample, resample);
        let min_elevation = elevations.iter().copied().min().unwrap_or(0);
        let max_elevation = elevations.iter().copied().max().unwrap_or(0);

        // Compute new cell size based on original geographic extent
        let orig_lat_span = grid.header.height as f64 * grid.header.cell_size;
        let orig_lon_span = grid.header.width as f64 * grid.header.cell_size;
        let new_cell_size_lat = orig_lat_span / resample as f64;
        let new_cell_size_lon = orig_lon_span / resample as f64;
        let new_cell_size = (new_cell_size_lat + new_cell_size_lon) / 2.0;

        TerrainGrid::new(
            TerrainHeader {
                origin_lat: grid.header.origin_lat,
                origin_lon: grid.header.origin_lon,
                cell_size: new_cell_size,
                width: resample,
                height: resample,
                min_elevation,
                max_elevation,
            },
            elevations,
            None,
            projection,
        )
    } else {
        grid
    };

    eprintln!("Writing .dtrn to {}...", output.display());
    match write_dtrn(&final_grid, &output) {
        Ok(()) => {
            let file_size = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
            eprintln!("Done! Output: {} ({} bytes)", output.display(), file_size);
        }
        Err(e) => {
            eprintln!("Error writing .dtrn: {e}");
            process::exit(1);
        }
    }
}

// --- Synthetic terrain command ---

fn cmd_synthetic(args: &[String]) {
    let (center_lat, center_lon) = parse_center(args).unwrap_or((26.5, 56.2));

    let output = match parse_output(args) {
        Some(p) => p,
        None => PathBuf::from("terrain_synth.dtrn"),
    };

    let size = parse_size(args, 256);
    let projection = GeoProjection::new(center_lat, center_lon);

    eprintln!("Generating {size}×{size} synthetic terrain...");
    eprintln!("Theater center: {center_lat}°N, {center_lon}°E");

    let grid = generate_synthetic_hormuz(size, center_lat, center_lon, &projection);

    eprintln!(
        "Elevation range: {}..{}m",
        grid.header.min_elevation, grid.header.max_elevation
    );

    eprintln!("Writing .dtrn to {}...", output.display());
    match write_dtrn(&grid, &output) {
        Ok(()) => {
            let file_size = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
            eprintln!("Done! Output: {} ({} bytes)", output.display(), file_size);
        }
        Err(e) => {
            eprintln!("Error writing .dtrn: {e}");
            process::exit(1);
        }
    }
}

/// Generate a synthetic terrain grid inspired by the Strait of Hormuz geography.
///
/// Creates a recognizable coastline with:
/// - Ocean in the center and south (representing the strait)
/// - Mountainous land to the north (Iran)
/// - Island/peninsula to the south-east (Oman/UAE coast)
/// - A few small islands
fn generate_synthetic_hormuz(
    size: u32,
    center_lat: f64,
    center_lon: f64,
    projection: &GeoProjection,
) -> TerrainGrid {
    // Grid covers approximately 2° × 2° centered on the theater center
    let span_deg = 2.0;
    let cell_size_arcsec = (span_deg * 3600.0) / size as f64;
    let origin_lat = center_lat - span_deg / 2.0;
    let origin_lon = center_lon - span_deg / 2.0;

    let mut elevations = Vec::with_capacity((size * size) as usize);
    let mut min_elev: i16 = 0;
    let mut max_elev: i16 = 0;

    for row in 0..size {
        for col in 0..size {
            // Normalized coordinates (0..1)
            let nx = col as f64 / size as f64;
            let ny = 1.0 - (row as f64 / size as f64); // 0=south, 1=north

            let elev = synthetic_elevation(nx, ny);
            let e = elev as i16;
            elevations.push(e);

            if e < min_elev {
                min_elev = e;
            }
            if e > max_elev {
                max_elev = e;
            }
        }
    }

    // Build ocean mask from elevations
    let cell_count = (size * size) as usize;
    let mask_bytes = cell_count.div_ceil(8);
    let mut ocean_mask = vec![0u8; mask_bytes];
    for i in 0..cell_count {
        if elevations[i] > 0 {
            // Land: set bit
            ocean_mask[i / 8] |= 1 << (i % 8);
        }
    }

    TerrainGrid::new(
        TerrainHeader {
            origin_lat,
            origin_lon,
            cell_size: cell_size_arcsec,
            width: size,
            height: size,
            min_elevation: min_elev,
            max_elevation: max_elev,
        },
        elevations,
        Some(ocean_mask),
        projection.clone(),
    )
}

/// Compute synthetic elevation at normalized coordinates.
/// nx: 0=west, 1=east. ny: 0=south, 1=north.
///
/// Creates a pattern reminiscent of the Strait of Hormuz:
/// - Northern landmass (Iran) with mountains
/// - Southern coast (Oman/UAE) with lower terrain
/// - Central strait (ocean)
/// - A few rocky islands
fn synthetic_elevation(nx: f64, ny: f64) -> f64 {
    // Northern landmass: land when ny > ~0.6, rising to mountains
    let north_shore = 0.55 + 0.05 * (nx * 8.0).sin() + 0.03 * (nx * 15.0).sin();
    let north_land = if ny > north_shore {
        let depth = (ny - north_shore) / (1.0 - north_shore);
        // Mountains rise steeply from coast
        let base = depth * 1200.0;
        let ridge = 400.0 * ((nx * 12.0).sin() * 0.5 + 0.5);
        let noise = 150.0 * ((nx * 30.0 + ny * 20.0).sin() * (ny * 25.0).cos());
        base + ridge + noise
    } else {
        0.0
    };

    // Southern landmass: land when ny < ~0.3, moderate terrain
    let south_shore = 0.3 - 0.04 * (nx * 6.0).sin() + 0.02 * (nx * 20.0).cos();
    // Eastern hook — the peninsula/coast juts up more on the east side
    let south_shore = south_shore + 0.15 * smooth_step(nx, 0.6, 0.8);
    let south_land = if ny < south_shore {
        let depth = (south_shore - ny) / south_shore;
        let base = depth * 400.0;
        let hills = 100.0 * ((nx * 10.0).sin() * (ny * 15.0).cos());
        base + hills
    } else {
        0.0
    };

    // Islands (Qeshm-like large island + smaller ones)
    let island1 = island_elevation(nx, ny, 0.5, 0.48, 0.12, 0.03, 200.0);
    let island2 = island_elevation(nx, ny, 0.35, 0.42, 0.04, 0.02, 80.0);
    let island3 = island_elevation(nx, ny, 0.65, 0.52, 0.03, 0.02, 120.0);

    // Combine: take max of all land features
    let elev = north_land
        .max(south_land)
        .max(island1)
        .max(island2)
        .max(island3);

    if elev < 1.0 {
        // Ocean floor (shallow negative)
        -20.0 - 30.0 * ((nx * 5.0 + ny * 3.0).sin().abs()) - 10.0 * ((nx * 12.0 + ny * 8.0).sin())
    } else {
        elev
    }
}

/// Elliptical island at (cx, cy) with semi-axes (rx, ry) and peak height.
fn island_elevation(nx: f64, ny: f64, cx: f64, cy: f64, rx: f64, ry: f64, peak: f64) -> f64 {
    let dx = (nx - cx) / rx;
    let dy = (ny - cy) / ry;
    let dist_sq = dx * dx + dy * dy;
    if dist_sq > 1.0 {
        return 0.0;
    }
    let t = 1.0 - dist_sq;
    peak * t * t // Smooth falloff
}

/// Smooth step function: 0 when x < edge0, 1 when x > edge1, smooth between.
fn smooth_step(x: f64, edge0: f64, edge1: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
