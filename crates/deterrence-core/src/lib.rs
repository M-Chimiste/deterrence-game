//! Core types and definitions for the DETERRENCE simulation.
//!
//! This crate defines the vocabulary shared across all other crates:
//! components, commands, state snapshots, events, and constants.
//! It has no dependency on Tauri or any runtime framework.

pub mod commands;
pub mod components;
pub mod constants;
pub mod enums;
pub mod events;
pub mod state;
pub mod types;

#[cfg(test)]
mod tests;
