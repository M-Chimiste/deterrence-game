//! Geographic projection: converts between lat/lon and simulation-space coordinates.
//!
//! Uses equirectangular projection centered on a theater reference point.
//! Accurate to <0.1% within 400km of the reference point.

use deterrence_core::types::Position;

/// Meters per degree of latitude (nearly constant across the globe).
const METERS_PER_DEGREE: f64 = 111_320.0;

/// Geographic projection anchored at a theater center point.
///
/// The reference point maps to sim-space origin (0, 0, 0).
/// x = East, y = North, z = Up (altitude in meters).
#[derive(Debug, Clone)]
pub struct GeoProjection {
    /// Theater center latitude in degrees.
    pub ref_lat: f64,
    /// Theater center longitude in degrees.
    pub ref_lon: f64,
    /// Cached cos(ref_lat) for longitude scaling.
    cos_ref_lat: f64,
}

impl GeoProjection {
    /// Create a new projection centered at the given lat/lon (degrees).
    pub fn new(ref_lat: f64, ref_lon: f64) -> Self {
        Self {
            ref_lat,
            ref_lon,
            cos_ref_lat: ref_lat.to_radians().cos(),
        }
    }

    /// Convert lat/lon (degrees) + altitude (meters) to sim-space Position.
    pub fn to_sim(&self, lat: f64, lon: f64, alt: f64) -> Position {
        let x = (lon - self.ref_lon) * METERS_PER_DEGREE * self.cos_ref_lat;
        let y = (lat - self.ref_lat) * METERS_PER_DEGREE;
        Position::new(x, y, alt)
    }

    /// Convert sim-space Position to (lat, lon, altitude) in degrees/meters.
    pub fn to_geo(&self, pos: &Position) -> (f64, f64, f64) {
        let lon = self.ref_lon + pos.x / (METERS_PER_DEGREE * self.cos_ref_lat);
        let lat = self.ref_lat + pos.y / METERS_PER_DEGREE;
        (lat, lon, pos.z)
    }

    /// Reference latitude (degrees).
    pub fn ref_lat(&self) -> f64 {
        self.ref_lat
    }

    /// Reference longitude (degrees).
    pub fn ref_lon(&self) -> f64 {
        self.ref_lon
    }

    /// Get the longitude scale factor (meters per degree of longitude at this latitude).
    pub fn lon_scale(&self) -> f64 {
        METERS_PER_DEGREE * self.cos_ref_lat
    }

    /// Get the latitude scale factor (meters per degree of latitude).
    pub fn lat_scale(&self) -> f64 {
        METERS_PER_DEGREE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_projection_roundtrip() {
        let proj = GeoProjection::new(26.5, 56.2);
        let lat = 26.8;
        let lon = 56.5;
        let alt = 100.0;

        let pos = proj.to_sim(lat, lon, alt);
        let (lat2, lon2, alt2) = proj.to_geo(&pos);

        assert!((lat - lat2).abs() < 1e-10, "lat roundtrip: {lat} vs {lat2}");
        assert!((lon - lon2).abs() < 1e-10, "lon roundtrip: {lon} vs {lon2}");
        assert!((alt - alt2).abs() < 1e-10, "alt roundtrip: {alt} vs {alt2}");
    }

    #[test]
    fn test_geo_projection_distances() {
        // At equator, 1 degree ≈ 111,320 m
        let proj = GeoProjection::new(0.0, 0.0);

        // 1 degree north
        let pos = proj.to_sim(1.0, 0.0, 0.0);
        assert!(
            (pos.y - 111_320.0).abs() < 1.0,
            "1 degree lat at equator: {} vs 111320",
            pos.y
        );
        assert!(pos.x.abs() < 1e-6, "no east offset");

        // 1 degree east at equator
        let pos = proj.to_sim(0.0, 1.0, 0.0);
        assert!(
            (pos.x - 111_320.0).abs() < 1.0,
            "1 degree lon at equator: {} vs 111320",
            pos.x
        );

        // At 60°N, 1 degree longitude ≈ 111,320 * cos(60°) ≈ 55,660 m
        let proj60 = GeoProjection::new(60.0, 0.0);
        let pos = proj60.to_sim(60.0, 1.0, 0.0);
        let expected = 111_320.0 * 60.0_f64.to_radians().cos();
        assert!(
            (pos.x - expected).abs() < 1.0,
            "1 degree lon at 60N: {} vs {expected}",
            pos.x
        );
    }

    #[test]
    fn test_origin_maps_to_zero() {
        let proj = GeoProjection::new(26.5, 56.2);
        let pos = proj.to_sim(26.5, 56.2, 0.0);
        assert!(pos.x.abs() < 1e-6);
        assert!(pos.y.abs() < 1e-6);
        assert!(pos.z.abs() < 1e-6);
    }
}
