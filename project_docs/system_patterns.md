# DETERRENCE — System Patterns

## Architecture Overview

DETERRENCE follows a **strict simulation-authority architecture**: the Rust backend owns all game state, the TypeScript frontend is a pure view layer, and Tauri provides the IPC bridge between them. This is not MVC — it's closer to a **command-query separation** where the frontend sends commands and receives state snapshots, never computing game logic.

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri 2.x Shell                       │
│                                                         │
│  ┌──────────────────┐     IPC Bridge     ┌───────────┐  │
│  │   Rust Backend    │◄──── invoke() ────│ TypeScript │  │
│  │   (Simulation     │──── event() ─────►│ Frontend   │  │
│  │    Authority)     │                   │ (View)     │  │
│  │                   │  PlayerCommand ►  │            │  │
│  │  deterrence-core  │  ◄ GameState      │  Three.js  │  │
│  │  deterrence-sim   │  ◄ AudioEvent     │  Canvas    │  │
│  │  deterrence-ai    │  ◄ Alert          │  Preact    │  │
│  │  deterrence-terrain│                  │  Howler    │  │
│  │  deterrence-procgen│                  │            │  │
│  │  deterrence-campaign│                 │            │  │
│  └──────────────────┘                    └───────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Guiding Principle

**Rust computes. TypeScript renders. Tauri connects.**

No game logic in the frontend. No rendering in the backend. The IPC boundary is the only coupling point, defined by a strict typed contract.

---

## Technology Stack

### Backend (Simulation)

| Technology | Version | Purpose |
|---|---|---|
| Rust | 2021 edition (stable) | Simulation engine, game logic, all authoritative state |
| Tauri | 2.x | Application shell, IPC, file system, window management, updates |
| serde + serde_json | latest | IPC serialization (JSON default, MessagePack fallback for perf) |
| nalgebra or glam | latest | Linear algebra: 3D kinematics, intercept geometry, coordinate transforms |
| hecs | latest | Entity-Component-System for entity management (tracks, missiles, batteries) |
| rand + rand_distr | latest | RNG: detection probability, Pk rolls, procedural generation |
| noise / simdnoise | latest | Perlin/simplex noise: atmospheric ducting, clutter generation |
| rayon | latest | Parallel iteration: radar sweep calculations across many contacts |
| redb | latest | Embedded DB: campaign persistence, save games |
| tokio | latest | Async runtime for Tauri event system and file I/O |

### Frontend (Rendering + UI)

| Technology | Version | Purpose |
|---|---|---|
| TypeScript | 5.x | All frontend code |
| Three.js | r160+ | 3D world view: ocean, terrain, missiles, effects, camera |
| Preact or Solid.js | latest | Reactive UI for panel components (lightweight, no React overhead) |
| Zustand | latest | State management: reactive store for GameState snapshots |
| Howler.js | latest | Audio playback: alerts, ambient, spatial, voice callouts |
| Vite | 5.x | Build tool: HMR, TypeScript compilation, asset bundling |
| @tauri-apps/api | 2.x | IPC bridge: invoke commands, listen to events |

### Development & Tooling

| Tool | Purpose |
|---|---|
| Cargo workspace | Multi-crate Rust project management |
| Vite + @tauri-apps/cli | Frontend build + Tauri integration |
| tweakpane | Runtime debug UI for tuning simulation parameters |
| cargo-watch | Auto-rebuild on Rust file changes |
| vitest | Frontend unit testing |
| cargo test | Backend unit + integration testing |

---

## Design Patterns

### 1. Entity-Component-System (Simulation)

All game entities (tracks, missiles, ships, batteries, jammers, civilian aircraft) are managed through an ECS architecture using `hecs`.

**Why ECS:**
- Tracks and missiles share some components (Position, Velocity) but differ in others (RadarCrossSection vs. SeekHead). ECS handles this naturally.
- Radar detection is a system that iterates over `(Position, RCS)` entities against `(Radar, Position)` entities — a classic ECS query.
- Adding new entity types (e.g., a new threat archetype) requires only composing existing components, no inheritance hierarchies.
- Parallel iteration via `rayon` is trivial with ECS — radar sweep over 500 entities parallelizes cleanly.

**Core Components:**
```
Position { lat, lon, alt }          — All entities
Velocity { speed, heading, climb }  — Moving entities
RadarCrossSection { base_rcs }      — Detectable entities
TrackInfo { track_number, quality, classification, iff_status }
ThreatProfile { archetype, phase, behavior_state }
MissileState { phase, target_id, fuel, seeker_status }
RadarSystem { type, energy_budget, sector, mode }
LauncherState { cells, ready_count, reload_timer }
Illuminator { channel_id, status, assigned_target }
NetworkNode { link_quality, connected_units }
```

**Core Systems (run each tick in order):**
1. `ThreatAISystem` — Advance threat state machines, pathfinding, swarm coordination
2. `RadarDetectionSystem` — Calculate Pd for each contact vs. each radar, manage track table
3. `IdentificationSystem` — IFF interrogation, kinematic profiling, correlation
4. `FireControlSystem` — DTE state machine, solution calculation, Pk evaluation
5. `EngagementSystem` — Illuminator scheduling, missile guidance, midcourse updates
6. `MissileKinematicsSystem` — Advance all in-flight missiles (friendly and threat)
7. `InterceptSystem` — Evaluate intercept events, Pk rolls, BDA
8. `EnvironmentSystem` — Update atmospheric conditions, ducting, clutter
9. `NetworkSystem` — CEC/IBCS state, composite track fusion, link quality
10. `AlertSystem` — Evaluate alert conditions, generate AudioEvents
11. `ScoringSystem` — Track mission metrics

### 2. Command Pattern (Player Input)

Player actions are encoded as a `PlayerCommand` enum, serialized, and dispatched to Rust via Tauri `invoke()`. Commands are validated and queued for the next simulation tick. This ensures:
- All input is processed at simulation time, not render time
- Input is replayable (for after-action review)
- Invalid commands are rejected at the boundary

```rust
enum PlayerCommand {
    HookTrack { track_id: u32 },
    ClassifyTrack { track_id: u32, classification: Classification },
    VetoEngagement { engagement_id: u32 },
    ConfirmEngagement { engagement_id: u32 },
    RedirectEngagement { engagement_id: u32, weapon_type: WeaponType },
    SetRadarSector { sector_center: f32, sector_width: f32 },
    SetRadarPriority { sector_id: u32, priority: Priority },
    SetDoctrine { mode: DoctrineMode },
    SetEmcon { mode: EmconMode },           // Ground only
    InitiateDisplacement {},                 // Ground only
    SetCourse { heading: f32, speed: f32 },  // Naval only
    ToggleView { view: ViewMode },
    SetTimeScale { scale: f32 },
    // ...
}
```

### 3. State Snapshot Broadcasting (Rust → Frontend)

The simulation runs at a fixed tick rate (default 30Hz). After each tick, the engine serializes the visible game state into a `GameStateSnapshot` and broadcasts it to the frontend via Tauri events. The frontend interpolates between consecutive snapshots for smooth rendering.

**Why snapshots, not incremental updates:**
- Simpler to reason about — every frame the frontend has complete state
- No desync bugs from missed deltas
- Replay is trivial — just replay the snapshot sequence
- Serialization cost is manageable at 30Hz for the expected state size (~50-100KB per snapshot)

**Snapshot structure (simplified):**
```rust
struct GameStateSnapshot {
    tick: u64,
    time: SimTime,
    tracks: Vec<TrackView>,           // All visible tracks with display data
    engagements: Vec<EngagementView>, // Active engagements with timers
    own_ship: OwnShipView,            // Or own_battery for ground
    systems: SystemsView,             // Radar, VLS/launchers, illuminators, network
    alerts: Vec<Alert>,               // Pending alerts since last tick
    environment: EnvironmentView,     // Current conditions
    score: ScoreView,                 // Running mission score
}
```

### 4. Observer Pattern (Audio Events)

Audio events are emitted as a separate channel from game state. This allows the audio system to react to discrete events (missile launch, intercept, new contact) without polling the full game state.

```rust
enum AudioEvent {
    NewContact { bearing: f32, priority: ThreatPriority },
    ContactLost { track_id: u32 },
    ThreatEvaluated { track_id: u32, threat_level: ThreatLevel },
    VetoClockStart { engagement_id: u32, duration_secs: f32 },
    BirdAway { weapon_type: WeaponType },
    Splash { result: InterceptResult },
    VampireVampire { bearing: f32, count: u32 },
    ArmInbound {},          // Ground only
    LauncherReloading {},   // Ground only
    DisplacementStarted {}, // Ground only
    NetworkDegraded { severity: f32 },
}
```

### 5. Data-Driven Configuration

All game content is defined in data files, not hardcoded. This enables rapid balancing, modding, and community content.

**Data file types:**
- `threats/*.yaml` — Threat archetype definitions (speed, RCS, behavior, phases)
- `interceptors/*.yaml` — Interceptor definitions (range, Pk curves, guidance type)
- `theaters/*.yaml` — Theater definitions (geography, environment profile, threat palette, civilian density)
- `scenarios/*.yaml` — Curated scenario templates for procedural generation
- `doctrines/*.yaml` — Engagement doctrine presets
- `symbology/*.yaml` — NTDS and MIL-STD-2525 symbol definitions
- `audio/*.yaml` — Audio event mappings and ambient layer definitions

Loaded by Rust at startup via `serde`, validated against schemas.

### 6. State Machine (Engagement Lifecycle)

Each engagement is modeled as an independent state machine that progresses through defined phases. State machines are explicit (enum-based), not implicit (boolean flags).

```rust
enum EngagementPhase {
    SolutionCalc { quality: f32, timer: f32 },
    Ready { veto_timer: f32, weapon: WeaponType, target: TrackId },
    Launched { missile_id: EntityId },
    Boost { capture_quality: f32 },
    Midcourse { error_signal: Vec3, uplink_quality: f32 },
    Terminal { guidance_mode: GuidanceMode, time_to_intercept: f32 },
    Intercept { result: Option<InterceptResult> },
    Complete { result: InterceptResult },
    Aborted { reason: AbortReason },
}
```

This pattern is used for all lifecycle entities: threats (cruise → terminal → impact), missiles (boost → midcourse → terminal), launchers (ready → firing → reloading), and the mission itself.

---

## File & Folder Structure

```
deterrence/
├── Cargo.toml                      # Workspace root
├── Cargo.lock
├── tauri.conf.json                  # Tauri configuration
├── .cargo/
│   └── config.toml                  # Cargo workspace settings
│
├── crates/
│   ├── deterrence-core/             # Core types, no dependencies on Tauri
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs             # Position, Velocity, Classification, etc.
│   │       ├── components.rs        # ECS components
│   │       ├── commands.rs          # PlayerCommand enum
│   │       ├── state.rs             # GameStateSnapshot, views
│   │       ├── events.rs            # AudioEvent, Alert
│   │       └── constants.rs         # Physical constants, defaults
│   │
│   ├── deterrence-sim/              # Simulation engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs            # Main simulation loop, tick management
│   │       ├── radar/
│   │       │   ├── mod.rs
│   │       │   ├── detection.rs     # Detection probability model
│   │       │   ├── tracking.rs      # Track management lifecycle
│   │       │   ├── energy.rs        # Radar energy budget
│   │       │   └── environment.rs   # Ducting, clutter, weather effects
│   │       ├── fire_control/
│   │       │   ├── mod.rs
│   │       │   ├── dte.rs           # Detect-to-Engage state machine
│   │       │   ├── solution.rs      # PIP calculation, solution quality
│   │       │   ├── illuminator.rs   # Channel scheduling
│   │       │   └── pk.rs            # Probability of kill model
│   │       ├── kinematics/
│   │       │   ├── mod.rs
│   │       │   ├── missile.rs       # Interceptor flight models
│   │       │   ├── threat.rs        # Threat flight profiles
│   │       │   └── geometry.rs      # Intercept geometry, engagement envelopes
│   │       ├── identification/
│   │       │   ├── mod.rs
│   │       │   ├── iff.rs           # IFF interrogation model
│   │       │   └── profiling.rs     # Kinematic profiling, correlation
│   │       └── network/
│   │           ├── mod.rs
│   │           ├── cec.rs           # Cooperative Engagement Capability
│   │           ├── ibcs.rs          # Integrated Battle Command System
│   │           └── fusion.rs        # Composite track fusion
│   │
│   ├── deterrence-threat-ai/        # Threat behavior
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── state_machine.rs     # Per-threat behavior FSM
│   │       ├── pathfinding.rs       # Terrain-following, waypoint routing
│   │       ├── swarm.rs             # Coordinated attack logic
│   │       ├── sead.rs              # SEAD/ARM targeting logic
│   │       ├── decoy.rs             # Decoy and jammer behavior
│   │       └── coordinator.rs       # Wave sequencing, time-on-top, multi-axis
│   │
│   ├── deterrence-terrain/          # Terrain system
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── heightmap.rs         # Heightmap loading and querying
│   │       ├── los.rs               # Line-of-sight calculation
│   │       ├── masking.rs           # Radar terrain masking (precomputed tables)
│   │       └── coverage.rs          # Coverage fan visualization data
│   │
│   ├── deterrence-procgen/          # Mission generation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── generator.rs         # Mission generator core
│   │       ├── theater.rs           # Theater parameter loading
│   │       ├── difficulty.rs        # Difficulty scaling curves
│   │       ├── validation.rs        # Solvability verification
│   │       └── civilian.rs          # Civilian traffic generation
│   │
│   ├── deterrence-campaign/         # Campaign & progression
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── campaign.rs          # Campaign state machine
│   │       ├── progression.rs       # Career tier unlocks
│   │       ├── scoring.rs           # Scoring calculations
│   │       ├── persistence.rs       # Save/load via redb
│   │       └── branching.rs         # Mission branching logic
│   │
│   └── deterrence-app/              # Tauri application entry
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs              # Tauri entry point
│           ├── ipc.rs               # Command handlers, event emitters
│           ├── state.rs             # App state management
│           └── config.rs            # Settings, user preferences
│
├── frontend/                        # TypeScript / Three.js frontend
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── index.html
│   └── src/
│       ├── main.ts                  # Application entry
│       ├── ipc/
│       │   ├── bridge.ts            # Tauri invoke/event wrappers
│       │   ├── commands.ts          # PlayerCommand type definitions (mirrors Rust)
│       │   └── state.ts             # GameStateSnapshot type definitions (mirrors Rust)
│       ├── store/
│       │   ├── gameState.ts         # Zustand store for GameState
│       │   └── interpolation.ts     # Snapshot interpolation for smooth rendering
│       ├── tactical/                # 2D tactical displays
│       │   ├── PPI.ts               # Plan Position Indicator (main radar display)
│       │   ├── symbology.ts         # NTDS / MIL-STD-2525 symbol rendering
│       │   ├── tracks.ts            # Track rendering: velocity leaders, history dots
│       │   ├── overlays.ts          # Range rings, bearing lines, coverage fans
│       │   ├── ascope.ts            # A-Scope display
│       │   └── shaders/
│       │       ├── phosphor.glsl    # Phosphor glow / CRT effect
│       │       └── bloom.glsl       # Symbol bloom
│       ├── panels/                  # UI panels (Preact/Solid components)
│       │   ├── VLSStatus.tsx        # VLS / launcher status panel
│       │   ├── IlluminatorStatus.tsx # Illuminator / guidance channel panel
│       │   ├── ThreatTable.tsx      # Threat evaluation / engagement schedule
│       │   ├── VetoClock.tsx        # Veto Clock display with countdown timers
│       │   ├── EnvironmentPanel.tsx # SEAWASP / radar coverage
│       │   ├── NetworkPanel.tsx     # CEC / IBCS status
│       │   ├── EmconPanel.tsx       # EMCON controls (ground)
│       │   ├── ReloadQueue.tsx      # Launcher reload management (ground)
│       │   ├── PEPButtons.tsx       # Programmable Entry Panel buttons
│       │   └── TrackBlock.tsx       # Hooked track data readout
│       ├── world/                   # 3D world view (Three.js)
│       │   ├── Scene.ts             # Scene setup, render loop
│       │   ├── Ocean.ts             # Ocean shader and mesh
│       │   ├── Terrain.ts           # Heightmap-based terrain
│       │   ├── Sky.ts               # Skybox, time-of-day lighting
│       │   ├── MissileTrails.ts     # Particle system for missile/threat trails
│       │   ├── Intercepts.ts        # Intercept visual effects
│       │   ├── Ships.ts             # Ship models
│       │   ├── Batteries.ts         # Ground battery models
│       │   ├── Camera.ts            # Camera controller (free, follow, cinematic)
│       │   └── shaders/
│       │       ├── ocean.glsl       # Water surface shader
│       │       └── trail.glsl       # Missile trail shader
│       ├── audio/
│       │   ├── AudioManager.ts      # Sound management, event routing
│       │   ├── ambient.ts           # CIC ambient layers
│       │   ├── alerts.ts            # Alert tone definitions
│       │   ├── tension.ts           # Tension escalation system
│       │   └── voice.ts             # Voice callout playback
│       ├── hud/
│       │   ├── WindowManager.ts     # Draggable/resizable panel layout
│       │   ├── layouts.ts           # Preset layout configurations
│       │   └── Theme.ts             # CIC dark theme, fonts, colors
│       └── input/
│           ├── InputManager.ts      # Keyboard/mouse event routing
│           ├── hotkeys.ts           # Hotkey definitions and remapping
│           └── trackball.ts         # Trackball cursor simulation (optional)
│
├── data/                            # Game content (data-driven)
│   ├── threats/
│   │   ├── sea_skimmer_mk1.yaml
│   │   ├── supersonic_cruiser.yaml
│   │   ├── hypersonic_glider.yaml
│   │   ├── tactical_ballistic.yaml
│   │   ├── subsonic_drone.yaml
│   │   └── ...
│   ├── interceptors/
│   │   ├── standard_naval.yaml
│   │   ├── extended_range.yaml
│   │   ├── mse.yaml
│   │   ├── high_altitude.yaml
│   │   └── ...
│   ├── theaters/
│   │   ├── strait_of_hormuz.yaml
│   │   ├── south_china_sea.yaml
│   │   ├── korean_peninsula.yaml
│   │   ├── central_europe.yaml
│   │   └── ...
│   ├── scenarios/                   # Curated scenario templates
│   ├── doctrines/                   # Engagement doctrine presets
│   ├── symbology/                   # Symbol definitions
│   └── audio/                       # Audio event mappings
│
├── assets/                          # Binary assets
│   ├── terrain/                     # Heightmap data per theater
│   ├── models/                      # 3D models (ships, batteries, missiles)
│   ├── textures/                    # Terrain textures, skyboxes
│   ├── sounds/                      # Audio files
│   └── fonts/                       # Monospace fonts (JetBrains Mono, IBM Plex Mono)
│
├── docs/                            # Project documentation
│   ├── project_brief.md
│   ├── product_context.md
│   ├── system_patterns.md
│   ├── tech_context.md
│   └── reference/                   # AEGIS technical reference, radar physics notes
│
└── tests/
    ├── sim/                         # Rust integration tests
    │   ├── radar_detection.rs
    │   ├── fire_control.rs
    │   ├── engagement_flow.rs
    │   └── mission_generation.rs
    └── frontend/                    # Frontend tests
        ├── ipc.test.ts
        └── symbology.test.ts
```

---

## Key Architectural Decisions

### Decision 1: Rust Owns All Game State

**Context:** The simulation involves complex interacting systems (radar, fire control, AI, environment) that must produce deterministic, consistent results.

**Decision:** All game logic lives in Rust. The TypeScript frontend receives serialized state snapshots and renders them. The frontend never computes game logic, even for seemingly simple things like "is this track in range."

**Rationale:** Determinism for replay. Single source of truth prevents desync. Performance for radar calculations. Enables headless testing of the entire simulation.

**Tradeoff:** Higher IPC overhead. Frontend cannot respond to hover/selection interactions without a round-trip to Rust for game-state-dependent calculations. Mitigated by including precomputed display data in snapshots (e.g., track blocks include pre-formatted strings, engagement status includes display-ready timers).

### Decision 2: ECS Over Object-Oriented Entities

**Context:** The game has many entity types (tracks, missiles, ships, batteries, jammers) with overlapping but not identical component sets.

**Decision:** Use `hecs` ECS rather than inheritance-based entity hierarchies.

**Rationale:** Component composition is natural for this domain. A "tracked contact" might be a threat missile, a civilian aircraft, a friendly ship, or a decoy — same Position and RCS components, different behavior components. Radar detection doesn't care what kind of entity it's detecting, only that it has Position and RCS. ECS queries model this perfectly.

**Tradeoff:** ECS has a learning curve. State machine logic for engagement phases may feel awkward in pure ECS. Hybrid approach: ECS for entity data, explicit state machines for lifecycle logic that references ECS entities.

### Decision 3: Fixed-Rate Simulation with Snapshot Broadcasting

**Context:** The simulation must run at consistent speed regardless of rendering frame rate, and must support replay.

**Decision:** Simulation runs at a fixed 30Hz tick rate in Rust. After each tick, a complete GameStateSnapshot is serialized and broadcast to the frontend. The frontend interpolates between snapshots for smooth rendering at display refresh rate.

**Rationale:** Decouples simulation from rendering. Consistent physics regardless of frame rate. Snapshot sequence IS the replay — no separate recording needed. 30Hz is sufficient for the decision timescales in the game (fastest Veto Clock is ~5 seconds).

**Tradeoff:** 30Hz introduces up to 33ms of display latency. Acceptable for this game's pace. Interpolation adds frontend complexity. Snapshot serialization is ~50-100KB per tick — manageable but must be profiled.

### Decision 4: JSON for IPC (MessagePack Fallback)

**Context:** Game state must cross the Rust-TypeScript boundary every tick.

**Decision:** Default to JSON via serde_json. If profiling shows serialization is a bottleneck, switch to MessagePack (rmp-serde) which is binary and ~2-5x faster.

**Rationale:** JSON is human-readable and debuggable during development. The expected payload size (50-100KB) should serialize in well under 5ms. MessagePack is a drop-in replacement via serde — same derive macros, different serializer.

**Tradeoff:** JSON is larger and slower than binary formats. Accepted for development velocity with a clear migration path if needed.

### Decision 5: Preact/Solid Over React for Panel UI

**Context:** Panel UI components (VLS status, threat table, Veto Clock display) need reactive rendering but are not complex enough to justify React's bundle size.

**Decision:** Use Preact or Solid.js for panel UI components. Final choice during Phase 1 scaffolding based on Three.js integration smoothness.

**Rationale:** Panels are relatively simple reactive components — data in, DOM out. React's virtual DOM diffing is unnecessary overhead. Preact is API-compatible with React at 3KB. Solid uses fine-grained reactivity with no VDOM at 7KB. Either is sufficient.

**Tradeoff:** Smaller ecosystem than React. Fewer ready-made component libraries. Acceptable since the UI is bespoke (CIC aesthetic, not Material Design).

### Decision 6: Heightmap-Based Terrain (Not Voxel or Mesh)

**Context:** Ground operations require terrain for LOS calculation, radar masking, and 3D visualization.

**Decision:** Use SRTM-derived or similar heightmap data (regular grid of elevation values). Pre-process into efficient query structures at mission load.

**Rationale:** Heightmaps are simple to query (O(1) for a point elevation), efficient for LOS ray-marching, and directly convertible to Three.js terrain meshes. Pre-computing terrain masking tables at mission load amortizes the cost.

**Tradeoff:** Limited vertical features (overhangs, tunnels — not relevant for radar). Resolution limited by heightmap grid spacing. Acceptable for the simulation fidelity target.
