<p align="center">
  <img src="deterrence-icon.jpeg" alt="Deterrence" width="200">
</p>

<h1 align="center">Deterrence</h1>

<p align="center">
A physics-based missile defense strategy game with Cold War retro CRT aesthetics.<br>
Command a national defense network, manage radar coverage and interceptor batteries,<br>
and make hard triage decisions against escalating ballistic threats — where every missile follows real physics.
</p>

<p align="center"><em>The soul of the game: you are always losing slowly, and strategy determines how slowly.</em></p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue" alt="Platform">
  <img src="https://img.shields.io/badge/license-Apache%202.0-green" alt="License">
  <img src="https://img.shields.io/badge/version-0.1.0-orange" alt="Version">
  <img src="https://img.shields.io/badge/tests-141%20passing-brightgreen" alt="Tests">
</p>

## Overview

Deterrence reimagines the classic *Missile Command* formula as a physics-driven strategic defense game. Players command a missile defense network across an expanding national territory, managing radar coverage, interceptor batteries, and civilian infrastructure against increasingly complex ballistic threats.

**Key features:**

- **Real ballistic physics** — gravity, atmospheric drag, altitude-dependent blast radii, shockwave propagation, and chain reactions. No shortcuts from Wave 1 onward.
- **Strategic campaign layer** — between tactical wave defense, allocate resources, expand territory, place batteries, upgrade radar, and make hard growth-vs-hardening tradeoffs.
- **4 interceptor types** — Sprint (fast, short-range), Standard (balanced), THAAD (high-altitude), and Patriot (terminal defense), each with distinct flight envelopes.
- **MIRV warheads & chain reactions** — enemy missiles split into multiple warheads mid-flight; skilled players exploit clustered salvos for chain kills.
- **Dynamic weather** — atmospheric conditions introduce wind forces that deflect missiles, alter wave intensity, and change the tactical landscape as campaigns progress.
- **Cold War CRT aesthetic** — phosphor glow, scanlines, film grain, vector-line displays, and teletype readouts evoke NORAD command bunkers.

## Architecture

Deterrence uses an **Authoritative Server / Dumb Client** pattern:

```
Rust Backend (60Hz fixed timestep)  ←— Tauri IPC —→  PixiJS/React Frontend (interpolated rendering)
```

| Layer | Responsibility | Tech |
|-------|---------------|------|
| **Backend** | Physics simulation, ECS world, game state, campaign logic, wave spawning, command validation | Rust (2024 edition), glam, rand_chacha |
| **Frontend** | Rendering, input capture, UI overlays, snapshot interpolation, particle effects, CRT shaders | TypeScript, PixiJS v8, React 18, Zustand |
| **Shell** | Native window, filesystem, IPC bridge | Tauri v2 |

The backend runs a custom lightweight ECS with ~15 component types and 14 systems executing in fixed order each tick. The frontend interpolates between backend state snapshots for smooth rendering at any refresh rate.

## Tech Stack

- **Rust** (2024 edition) — physics, ECS, game logic
- **TypeScript** — frontend rendering and UI
- **PixiJS v8** — WebGL 2 rendering
- **React 18** — UI overlays (HUD, menus, strategic phase)
- **Zustand** — frontend state management
- **Tauri v2** — native desktop shell and IPC
- **Vite** — frontend build tooling
- **glam** — vector/matrix math
- **rand + rand_chacha** — deterministic seeded RNG
- **serde** — serialization for IPC and save files

## Prerequisites

- [Rust](https://rustup.rs/) (stable, 2024 edition)
- [Node.js](https://nodejs.org/) (v18+)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform

## Getting Started

```bash
# Clone the repository
git clone https://github.com/your-username/deterrence-game.git
cd deterrence-game

# Install frontend dependencies
npm install

# Run in development mode (launches Tauri window with hot-reload)
cargo tauri dev
```

## Build & Development

```bash
# Development (opens native window with hot-reload)
cargo tauri dev

# Production build (creates distributable binary)
cargo tauri build

# Run Rust tests (141 tests — unit, physics, determinism, integration)
cargo test

# Rust linting
cargo clippy
cargo fmt --check

# Frontend type-checking
npm run lint

# Frontend build only
npm run build
```

### Windows-specific scripts

```powershell
npm run win:dev      # Development via PowerShell
npm run win:build    # Production build via PowerShell
```

## Project Structure

```
src-tauri/src/
├── commands/        # Tauri invoke handlers (tactical, campaign, persistence)
├── engine/          # Game loop, simulation orchestrator, config
├── ecs/             # Entity-Component-System (entity IDs, components, world)
├── systems/         # 14+ systems (gravity, drag, thrust, collision, damage, ...)
├── campaign/        # Territory, economy, upgrades, wave composition
├── state/           # Game state, campaign state, snapshots, weather
├── events/          # Event definitions for frontend communication
├── persistence/     # Save/load logic
├── lib.rs           # Module declarations + Tauri builder
└── main.rs          # Entry point

src/
├── renderer/        # PixiJS (GameRenderer, TacticalView, StrategicView)
│   ├── effects/     # MenuBackground, ParticleManager
│   └── shaders/     # CRT filter, phosphor glow (GLSL)
├── audio/           # AudioManager, MusicManager, SoundSynth
├── input/           # InputManager (keyboard + mouse)
├── ui/              # React components (MainMenu, HUD, StrategicOverlay, ...)
│   ├── components/  # UI panels and overlays
│   └── styles/      # CSS modules
├── bridge/          # Tauri IPC (commands + event listeners)
├── types/           # TypeScript type definitions
└── main.tsx         # Application entry point

project_docs/        # Design documentation and progress tracking
public/music/        # Audio assets (WAV)
```

## Design Pillars

1. **Physics Are Real, Always** — consistent simulation from Wave 1. Difficulty scales through scenario complexity, not system simplification.
2. **Every Interception Is a Decision** — altitude, timing, and blast geometry all matter. High intercepts are easier but need radar; low intercepts risk collateral damage.
3. **Strategic Depth Between Waves** — resource allocation, territory expansion, battery placement, and upgrade paths create a metagame around each tactical crisis.
4. **Cold War Anxiety** — tense, procedural atmosphere. You're an operator in a bunker making life-and-death calls with incomplete information.

## Current Status

The project is in active development. Phases 1 through 8A of the [implementation plan](project_docs/implementation_plan.md) are complete:

- Custom ECS with generational entity IDs
- Full physics simulation (gravity, drag, thrust, shockwaves, chain reactions)
- Wave defense gameplay loop with 4 interceptor types and MIRVs
- Campaign system (territory, economy, upgrades, wave composition)
- Strategic phase UI with territory map
- Save/load system with campaign persistence
- Weather system (wind, wave intensity modifiers)
- CRT shader pipeline (scanlines, phosphor glow, film grain)
- Procedural audio system with synthesized sound effects
- React UI with main menu, settings, HUD overlays
- 141 automated tests passing

See [progress.md](project_docs/progress.md) for detailed implementation history.

## Documentation

Detailed design specifications live in `project_docs/`:

| Document | Contents |
|----------|----------|
| [project_brief.md](project_docs/project_brief.md) | Full PRD — gameplay, physics, controls, difficulty, visual style |
| [product_context.md](project_docs/product_context.md) | User stories, audience, UX goals, market positioning |
| [system_patterns.md](project_docs/system_patterns.md) | Architecture, tech stack, design patterns, testing strategy |
| [implementation_plan.md](project_docs/implementation_plan.md) | 9-phase implementation roadmap |
| [progress.md](project_docs/progress.md) | Phase-by-phase implementation progress |

## License

This project is licensed under the [Apache License 2.0](LICENSE).
