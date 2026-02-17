//! Terrain system for DETERRENCE.
//!
//! Heightmap loading, line-of-sight calculation,
//! and radar terrain masking.

pub use deterrence_core as core;

pub mod coastline;
pub mod dtrn;
pub mod grid;
pub mod hgt;
pub mod los;
pub mod projection;

// Re-export key types for convenience.
pub use coastline::extract_coastlines;
pub use grid::{TerrainGrid, TerrainHeader};
pub use los::has_line_of_sight;
pub use projection::GeoProjection;
