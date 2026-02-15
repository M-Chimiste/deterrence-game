//! DETERRENCE Tauri application.
//!
//! This crate wires together all simulation crates and exposes
//! them to the frontend via Tauri IPC commands and events.

pub mod game_loop;
pub mod ipc;
pub mod state;

pub use deterrence_core as core;
