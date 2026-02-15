//! Fundamental geometric and simulation types.

use serde::{Deserialize, Serialize};

/// 3D position in simulation space (meters, Cartesian).
/// x = East, y = North, z = Up (altitude).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// 3D velocity in simulation space (m/s).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Simulation time tracking.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SimTime {
    /// Current tick number (increments by 1 each tick).
    pub tick: u64,
    /// Elapsed simulation time in seconds.
    pub elapsed_secs: f64,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Range to another position in meters (3D distance).
    pub fn range_to(&self, other: &Position) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Horizontal range (ignoring altitude).
    pub fn horizontal_range_to(&self, other: &Position) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Bearing to another position in radians (0 = North, clockwise).
    pub fn bearing_to(&self, other: &Position) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        dx.atan2(dy).rem_euclid(std::f64::consts::TAU)
    }
}

impl Velocity {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Speed magnitude (m/s).
    pub fn speed(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Horizontal speed (ignoring vertical component).
    pub fn horizontal_speed(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Heading in radians (0 = North, clockwise).
    pub fn heading(&self) -> f64 {
        self.x.atan2(self.y).rem_euclid(std::f64::consts::TAU)
    }
}

impl SimTime {
    /// Seconds per tick at the default tick rate.
    pub fn dt(&self) -> f64 {
        1.0 / crate::constants::TICK_RATE as f64
    }

    /// Advance by one tick.
    pub fn advance(&mut self) {
        self.tick += 1;
        self.elapsed_secs += self.dt();
    }
}
