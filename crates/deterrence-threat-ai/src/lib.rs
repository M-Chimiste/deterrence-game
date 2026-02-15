//! Threat AI for DETERRENCE.
//!
//! Implements threat behavior state machines, wave coordination,
//! and archetype-driven flight profiles.

pub mod fsm;
pub mod profiles;

pub use deterrence_core as core;

#[cfg(test)]
mod tests;
