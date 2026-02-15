//! Simulation engine for DETERRENCE.
//!
//! Owns the hecs ECS world, runs systems at a fixed tick rate,
//! and produces GameStateSnapshots for the frontend.

pub mod engagement;
pub mod engine;
pub mod systems;
pub mod world_setup;

pub use deterrence_core as core;
pub use engine::SimulationEngine;

#[cfg(test)]
mod tests;
