//! ECS systems that operate on the simulation world each tick.
//!
//! Systems are pure functions that take `&mut World` (or `&World` for read-only).
//! They do not own state — all state lives in components.

pub mod cleanup;
pub mod fire_control;
pub mod illuminator;
pub mod intercept;
pub mod missile_kinematics;
pub mod movement;
pub mod radar;
pub mod snapshot;
pub mod threat_ai;
pub mod wave_spawner;
