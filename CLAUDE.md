# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**DETERRENCE** is a real-time Integrated Air and Missile Defense (IAMD) simulation game. The player operates as a battle manager supervising automated defense systems through authentic radar console interfaces, making identification and engagement decisions under time pressure.

Built with **Tauri 2.x** (Rust backend) + **TypeScript frontend** (Three.js + Preact).

## Architecture

**Strict simulation-authority pattern: Rust computes, TypeScript renders, Tauri connects.**

- All game state and logic lives in Rust — the frontend never computes game logic
- Frontend sends `PlayerCommand` enums via Tauri `invoke()`; receives `GameStateSnapshot` via Tauri events
- Simulation runs at a fixed 30Hz tick rate; frontend interpolates between snapshots for smooth rendering
- Snapshots are complete state (not deltas) — simpler to reason about, enables replay

### Crate Structure (Cargo workspace)

```
crates/
  deterrence-core/       — Core types, enums, IPC contract (no Tauri dep)
  deterrence-sim/        — Simulation engine, hecs ECS, systems
  deterrence-threat-ai/  — Threat behavior FSMs
  deterrence-terrain/    — Heightmap loading, LOS calculation
  deterrence-procgen/    — Procedural mission generation
  deterrence-campaign/   — Campaign state, progression, save/load
  deterrence-app/        — Tauri entry point, IPC handlers, game loop
```

### Tech Stack

| Layer | Technology | Notes |
|---|---|---|
| Backend | Rust (2021 edition), Tauri 2.x | All authoritative state |
| ECS | hecs 0.10 | Entity-Component-System for tracks, missiles, batteries |
| Math | glam 0.29 | 3D kinematics, intercept geometry |
| RNG | rand 0.8 + rand_chacha 0.3 | Deterministic with seed |
| Frontend | TypeScript 5.x, Three.js 0.171, Preact 10.x | 3D world + CIC console UI |
| State mgmt | Zustand 5.x | Reactive store for game state snapshots |
| Audio | Howler.js 2.x | Alerts, ambient, voice callouts |
| Build | Vite 6.x + Tauri CLI 2.x | HMR in dev, bundled in release |

### Key Source Files

- IPC contract (Rust): `crates/deterrence-core/src/commands.rs`, `crates/deterrence-core/src/state.rs`
- IPC contract (TypeScript): `frontend/src/ipc/commands.ts`, `frontend/src/ipc/state.ts`
- Tauri config: `crates/deterrence-app/tauri.conf.json`
- Frontend entry: `frontend/src/main.tsx`

### System Execution Order (per tick)

1. ThreatAI → 2. RadarDetection → 3. Identification → 4. FireControl → 5. Engagement → 6. MissileKinematics → 7. Intercept → 8. Environment → 9. Network → 10. Alert → 11. Scoring

## Build Commands

```bash
# Full dev mode (Rust + frontend with HMR)
cargo tauri dev --manifest-path crates/deterrence-app/Cargo.toml

# Rust-only testing (headless simulation)
cargo test --workspace

# Single test
cargo test --workspace test_name

# Frontend only
cd frontend && npm run dev

# Frontend type check
cd frontend && npx tsc --noEmit

# Frontend build
cd frontend && npx vite build

# Lint (0 warnings policy)
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all

# Check format without modifying
cargo fmt --all --check
```

## Coding Standards

### Rust
- `clippy` with `-D warnings` — all warnings are errors
- `rustfmt` defaults — run `cargo fmt --all` before commit
- No `unwrap()` in simulation code — use `?` or explicit handling; `unwrap()` OK in tests with invariant comments
- Components are plain structs with pub fields; systems are standalone functions taking `&World` / `&mut World`
- Use `#[derive(Default)]` with `#[default]` attribute on enums (not manual `impl Default`)
- Avoid allocations in the hot loop (simulation tick); pre-allocate and reuse buffers

### TypeScript
- Strict mode, no `any` — use `unknown` and type guards
- Preact for UI components (`jsxImportSource: "preact"` in tsconfig)
- IPC types in `frontend/src/ipc/` mirror Rust types exactly

### IPC Contract
- Rust `PlayerCommand` and `GameStateSnapshot` are the canonical type definitions
- TypeScript mirrors in `frontend/src/ipc/` must be kept in sync manually
- Changes to IPC contract require updating both sides

## Key Game Mechanics

- **Veto Clock**: AUTO-SPECIAL doctrine — system auto-engages unless player vetoes within countdown
- **Radar energy budget**: Finite energy traded between search and track modes
- **Illuminator bottleneck**: 3 channels for terminal guidance; saturation forces prioritization
- **IFF identification dilemma**: Civilian traffic mixed with threats
- **NTDS symbology**: Authentic radar display symbols (Unknown=circle, Hostile=diamond, Friendly=semicircle)

## Performance Targets

| Metric | Target |
|---|---|
| Simulation tick rate | 30Hz sustained, 500+ entities |
| Frontend FPS | 60fps on integrated GPU |
| IPC serialization | < 3ms per snapshot |
| Snapshot size | < 100KB per tick |
| Memory | < 500MB for complex scenarios |
| Startup to menu | < 3 seconds |

## Assets

- `public/music/` — WAV audio (intro + loop pairs for menu, title, levels, gameover)
- `project_docs/` — Design documents (project_brief, product_context, system_patterns, tech_context, terrain_data_guide)

## Implementation Plan

See `/Users/c/.claude/plans/giggly-fluttering-teacup.md` for the 10-phase implementation plan.
Current status: **Phase 3 complete** (IPC bridge, game loop thread, frontend connection, debug overlay).
