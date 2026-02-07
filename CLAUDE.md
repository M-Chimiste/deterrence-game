# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Deterrence is a desktop strategy game — a physics-based missile defense simulation with Cold War retro CRT aesthetics. Built with **Tauri v2** (Rust backend + PixiJS/TypeScript frontend). Currently in **design phase** with comprehensive documentation but no implementation code yet.

## Technology Stack

- **Backend:** Rust (2024 edition) — physics simulation, game state, campaign logic
- **Frontend:** TypeScript + PixiJS v8 — WebGL 2 rendering, CRT shaders, UI
- **Shell:** Tauri v2 — native window, filesystem, IPC bridge
- **Build:** Cargo + Vite
- **Physics math:** glam | **RNG:** rand + rand_chacha (deterministic seeded) | **Serialization:** serde + serde_json

## Build & Run Commands

```bash
# Development
cargo tauri dev

# Production build
cargo tauri build

# Rust tests
cargo test

# Rust linting
cargo clippy
cargo fmt --check

# Frontend
npm install
npm run build
npm run lint
```

## Architecture

**Authoritative Server / Dumb Client pattern** — Rust backend is the single source of truth; PixiJS frontend is a pure renderer.

```
Rust Backend (60Hz fixed timestep) ←—IPC (Tauri events/commands)—→ PixiJS Frontend (interpolated rendering)
```

**Rust backend** owns: physics simulation, ECS world, game state, campaign logic, wave spawning, command validation.
**TypeScript frontend** owns: rendering, input capture, UI, snapshot interpolation, particle effects, CRT shaders.

### ECS (Custom Lightweight)

Not using Bevy — custom ECS with ~15 component types and ~14 systems executing in fixed order:
InputSystem → WaveSpawner → Thrust → Gravity → Drag → Movement → MirvSplit → Collision → Detonation → Shockwave → Damage → Cleanup → Detection → StateSnapshot

### IPC Flow

- Backend pushes state snapshots via Tauri events (`game:state_snapshot`, `game:detonation`, `game:impact`, etc.)
- Frontend invokes Tauri commands for player actions (launch interceptor, select battery, etc.)
- Events include tick counter for sequencing

### Planned File Structure

```
src-tauri/src/
├── commands/       # Tauri invoke handlers (tactical, strategic, campaign)
├── engine/         # Game loop & simulation orchestration
├── ecs/            # ECS world, components, entities
├── systems/        # One file per system (physics, spawning, damage, etc.)
├── campaign/       # Territory, economy, upgrades, wave composition
├── state/          # Game state types
├── events/         # Event definitions
└── persistence/    # Save/load logic

src/
├── renderer/       # PixiJS (GameRenderer, TacticalView, StrategicView, HUD)
│   ├── entities/   # MissileRenderer, InterceptorRenderer, etc.
│   ├── effects/    # ParticleManager, WeatherOverlay, ImpactEffect
│   └── shaders/    # CRT, phosphor, filmgrain GLSL shaders
├── input/          # Input handling & keybinds
├── ui/             # Strategic phase UI components
├── state/          # RenderState, UIState, AudioState
├── audio/          # AudioManager, SoundBank, MusicController
├── bridge/         # Tauri IPC commands and events
└── types/          # TypeScript type definitions
```

## Key Design Constraints

- **Fixed 60Hz physics timestep** — frame-rate independent, deterministic simulation
- **Deterministic RNG** — seeded waves produce identical outcomes for reproducibility
- **Frontend interpolation** — lerp between backend snapshots for smooth rendering at any refresh rate
- **All state mutations in Rust only** — frontend never modifies game state
- **Performance target:** 100+ simultaneous entities on integrated GPU (~2018 Intel UHD 620+)

## Design Documentation

All specifications live in `project_docs/`:
- **project_brief.md** — Full PRD: gameplay, physics, controls, difficulty, visual style, metrics
- **product_context.md** — User stories, audience, UX goals, market positioning
- **system_patterns.md** — Architecture, tech stack, design patterns, file structure, testing strategy
