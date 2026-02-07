# Deterrence — System Patterns

**Technology stack, architecture, design patterns, and key technical decisions.**

---

## Technology Stack

| Layer | Technology | Version Target | Purpose |
|---|---|---|---|
| Application Shell | Tauri | v2.x | Native window management, filesystem access, IPC bridge between Rust and webview |
| Backend / Game Engine | Rust | 2024 edition | Physics simulation, game state management, campaign logic, save/load |
| Frontend / Renderer | PixiJS | v8.x | WebGL 2 rendering, particle systems, CRT post-processing shaders |
| Frontend Framework | TypeScript | 5.x | Type-safe frontend logic, UI state, input handling |
| UI Layer | HTML/CSS | — | Strategic phase menus, HUD overlays, teletype readouts |
| Build System | Cargo + Vite | — | Rust compilation via Cargo, frontend bundling via Vite (Tauri's default frontend tooling) |
| Serialization | serde + serde_json | — | Game state serialization for save/load and IPC |
| RNG | rand + rand_chacha | — | Deterministic seeded RNG for wave generation and reproducibility |
| Physics Math | glam | — | Fast vector/matrix math for trajectory calculations (f32, no-std compatible) |

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri Shell (v2)                      │
│                                                         │
│  ┌───────────────────┐       ┌───────────────────────┐  │
│  │   Rust Backend     │ IPC  │   Webview Frontend     │  │
│  │                   │◄─────►│                       │  │
│  │  ┌─────────────┐ │       │  ┌─────────────────┐  │  │
│  │  │ Physics Sim  │ │       │  │ PixiJS Renderer  │  │  │
│  │  │ (fixed tick) │ │       │  │ (vsync render)   │  │  │
│  │  └─────────────┘ │       │  └─────────────────┘  │  │
│  │  ┌─────────────┐ │       │  ┌─────────────────┐  │  │
│  │  │ Game State   │ │       │  │ Input Manager    │  │  │
│  │  │ (authority)  │ │       │  │ (mouse/keyboard) │  │  │
│  │  └─────────────┘ │       │  └─────────────────┘  │  │
│  │  ┌─────────────┐ │       │  ┌─────────────────┐  │  │
│  │  │ Wave Engine  │ │       │  │ UI / HUD Layer   │  │  │
│  │  │ (spawning)   │ │       │  │ (HTML/CSS + JS)  │  │  │
│  │  └─────────────┘ │       │  └─────────────────┘  │  │
│  │  ┌─────────────┐ │       │  ┌─────────────────┐  │  │
│  │  │ Campaign Mgr │ │       │  │ Post-Processing  │  │  │
│  │  │ (save/load)  │ │       │  │ (CRT shaders)    │  │  │
│  │  └─────────────┘ │       │  └─────────────────┘  │  │
│  └───────────────────┘       └───────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Core Architectural Pattern: Authoritative Server / Dumb Client

The Rust backend operates as the single source of truth for all game state, functioning like a local game server. The frontend is a stateless renderer that receives snapshots and sends input commands. This pattern was chosen because:

- Physics determinism requires a single authoritative simulation with no competing state
- Save/load becomes trivial — serialize the Rust state, done
- No desync bugs between "what the game thinks" and "what the player sees"
- If multiplayer is ever added, the architecture already separates simulation from rendering
- Frontend can be swapped or reskinned without touching game logic

---

## Design Patterns

### 1. Entity-Component System (ECS) — Physics Simulation

The Rust backend uses a lightweight ECS pattern for all simulated entities (enemy missiles, interceptors, detonations, shockwaves). This is not a full ECS framework like Bevy — it's a simple, purpose-built implementation using Rust structs and Vec storage.

**Why ECS over OOP hierarchy:**
- Missiles, interceptors, and shockwaves share physics components (position, velocity, drag coefficient) but differ in behavior components (guided vs. ballistic, detonation trigger, MIRV split logic)
- ECS allows composing entity types from reusable components without deep inheritance trees
- Cache-friendly iteration over component arrays matters when simulating 100+ entities per tick during saturation attacks
- Easy to add new entity types (new interceptor variants, new warhead types) without refactoring existing code

**Core Components:**
```
Transform       { position: Vec2, rotation: f32 }
Velocity        { linear: Vec2, angular: f32 }
Ballistic       { drag_coefficient: f32, mass: f32, cross_section: f32 }
Warhead         { yield_force: f32, blast_radius_base: f32, warhead_type: WarheadType }
MirvCarrier     { child_count: u8, split_altitude: f32, spread_angle: f32 }
Interceptor     { thrust: f32, burn_time: f32, ceiling: f32, battery_id: u32 }
Radar           { range: f32, resolution: f32, weather_resistance: f32 }
Health          { current: f32, max: f32 }
Lifetime        { remaining_ticks: u32 }
ReentryGlow     { intensity: f32, altitude_threshold: f32 }
```

**Core Systems (executed in order each tick):**
```
1. InputSystem          — Process player commands from frontend (launch interceptor, etc.)
2. WaveSpawnerSystem    — Spawn new enemy missiles per wave schedule
3. ThrustSystem         — Apply thrust to active interceptors during burn phase
4. GravitySystem        — Apply gravitational acceleration to all ballistic entities
5. DragSystem           — Apply altitude-dependent atmospheric drag
6. MovementSystem       — Integrate velocity into position
7. MirvSplitSystem      — Check MIRV carriers against split altitude, spawn child warheads
8. CollisionSystem      — Detect proximity between shockwaves and entities (destroy/deflect)
9. DetonationSystem     — Trigger interceptor detonations, spawn shockwave entities
10. ShockwaveSystem     — Expand shockwave radius, apply force falloff, expire
11. DamageSystem        — Apply damage to cities/infrastructure from warhead impacts
12. CleanupSystem       — Remove expired entities (detonated warheads, dissipated shockwaves)
13. DetectionSystem     — Update radar contacts based on radar range, weather, re-entry glow
14. StateSnapshotSystem — Package current state for frontend transmission
```

### 2. Event-Driven Communication — IPC Bridge

Communication between Rust backend and TypeScript frontend uses Tauri's event system and invoke commands, following an event-driven pattern.

**Backend → Frontend (Events — push model):**
Events are emitted by the backend whenever state changes that the frontend needs to render. The frontend subscribes to event channels and updates its render state accordingly.

```
Event: "game:state_snapshot"     — Full entity positions/states, emitted every physics tick
Event: "game:detonation"         — Detonation occurred at position, yield, altitude (triggers VFX)
Event: "game:impact"             — Warhead hit ground at position (triggers impact VFX + audio)
Event: "game:chain_reaction"     — Chain reaction occurred (triggers chain VFX sequence)
Event: "game:city_damaged"       — City took damage (triggers UI update + audio)
Event: "game:city_destroyed"     — City lost (triggers major UI event)
Event: "game:wave_start"         — New wave beginning (triggers wave counter, weather update)
Event: "game:wave_complete"      — Wave ended (transition to strategic phase)
Event: "game:detection"          — New radar contact or re-entry glow spotted
Event: "campaign:state_update"   — Campaign state changed (territory, resources, upgrades)
```

**Frontend → Backend (Commands — pull model):**
The frontend invokes Rust commands when the player takes an action. Commands are validated by the backend before execution.

```
Command: "launch_interceptor"    — { battery_id, target_x, target_y }
Command: "select_battery"        — { battery_id }
Command: "place_battery"         — { region_id, position_x, position_y }
Command: "upgrade_interceptor"   — { interceptor_type, upgrade_path }
Command: "upgrade_radar"         — { radar_id, upgrade_type }
Command: "expand_territory"      — { target_region_id }
Command: "repair_city"           — { city_id }
Command: "build_shelter"         — { city_id }
Command: "start_wave"            — { }
Command: "save_game"             — { slot_id }
Command: "load_game"             — { slot_id }
```

**Why event-driven over request/response:**
- The physics sim runs at a fixed tick rate independent of the frontend — it pushes state, it doesn't wait to be asked
- Events naturally decouple the simulation from rendering; the frontend processes whatever events are available each render frame
- VFX events (detonation, impact) need to arrive at the moment they happen, not when the frontend next polls
- The command pattern for player input gives the backend authority to validate and reject invalid actions (e.g., launching from an empty battery)

### 3. State Machine — Game Phase Management

The game transitions through distinct phases, managed by a state machine in the Rust backend.

```
┌──────────┐    start     ┌──────────┐   all enemies   ┌──────────────┐
│ STRATEGIC │───────────►│  WAVE    │──────────────►│ WAVE_RESULT  │
│  PHASE    │◄───────────│  ACTIVE  │               │  (summary)   │
└──────────┘  continue   └──────────┘               └──────┬───────┘
     │                        │                            │
     │                        │ all cities lost            │ acknowledge
     │                        ▼                            ▼
     │                   ┌──────────┐              ┌──────────────┐
     │                   │ REGION   │              │  STRATEGIC   │
     │                   │  LOST    │              │   PHASE      │
     │                   └────┬─────┘              └──────────────┘
     │                        │
     │            homeland?   │   frontier?
     │               ▼        │      ▼
     │         ┌──────────┐   │ ┌───────────┐
     │         │ CAMPAIGN  │   └►│ CONTRACTION│──► STRATEGIC PHASE
     │         │  OVER     │     └───────────┘
     │         └──────────┘
     │
     ▼
┌──────────┐
│  PAUSED   │ (spacebar during strategic phase only)
└──────────┘
```

**Phase responsibilities:**

| Phase | Backend | Frontend |
|---|---|---|
| STRATEGIC | Validates placements/upgrades, manages economy | Renders campaign map, upgrade UI, intel briefing |
| WAVE_ACTIVE | Runs physics sim at fixed tick, processes input commands | Renders tactical view, handles mouse/keyboard input |
| WAVE_RESULT | Calculates damage totals, updates campaign state | Shows wave summary (missiles destroyed, cities hit, resources earned) |
| REGION_LOST | Removes region from player territory, adjusts enemy patterns | Shows region loss event, map contraction |
| CAMPAIGN_OVER | Calculates final statistics | Shows campaign end screen, offers new campaign |
| PAUSED | Simulation frozen, state preserved | Dim overlay, pause menu |

### 4. Observer Pattern — Detection System

The radar and re-entry glow detection system uses an observer pattern where detection sources (radar installations, visual scanning) observe the simulation and produce contacts for the frontend to display.

**Why this matters as a separate pattern:** The player does not see raw simulation state. They see *detected* simulation state, filtered through their radar network and weather conditions. An enemy missile exists in the physics simulation from the moment it spawns, but it only appears on the player's screen when a detection source picks it up. This separation is critical to the radar-as-a-resource mechanic.

```
DetectionManager
  ├── RadarSource[] (one per radar installation)
  │     └── Observes: all entities within range × weather_multiplier
  │         Produces: RadarContact { entity_id, position, velocity, confidence }
  │
  └── VisualSource (single, global)
        └── Observes: entities below re-entry glow altitude threshold
            Filtered by: weather_visibility (clear = 1.0, overcast = 0.3, storm = 0.0)
            Produces: GlowContact { position, intensity } (no velocity — player must infer)
```

The frontend receives merged contacts from all detection sources. Radar contacts include velocity vectors (shown as trajectory predictions on the HUD). Glow contacts are position-only — the player sees a streak but must infer trajectory from watching it move across frames. This asymmetry between detection methods is a deliberate design choice that rewards skilled players who invest attention rather than just resources.

### 5. Command Pattern — Player Actions

All player actions during both tactical and strategic phases are encapsulated as command objects. This provides:

- **Validation:** The backend checks every command before execution (sufficient ammo? battery in range? enough resources for upgrade?)
- **Replay potential:** If replay functionality is ever added, the command log perfectly reproduces a session when fed into the deterministic simulation
- **Undo (strategic phase):** During the between-wave phase, placement and upgrade commands can be undone before confirming and starting the next wave

---

## File & Folder Structure

```
deterrence/
├── src-tauri/                          # Rust backend (Tauri + game engine)
│   ├── Cargo.toml
│   ├── tauri.conf.json                 # Tauri configuration (window, permissions, bundler)
│   ├── src/
│   │   ├── main.rs                     # Tauri app entry point, plugin registration
│   │   ├── lib.rs                      # Module declarations
│   │   ├── commands/                   # Tauri invoke command handlers
│   │   │   ├── mod.rs
│   │   │   ├── tactical.rs             # launch_interceptor, select_battery
│   │   │   ├── strategic.rs            # place_battery, upgrade_radar, expand_territory
│   │   │   └── campaign.rs             # save_game, load_game, start_wave
│   │   ├── engine/                     # Core game engine
│   │   │   ├── mod.rs
│   │   │   ├── game_loop.rs            # Fixed-timestep loop, phase state machine
│   │   │   ├── simulation.rs           # Top-level simulation orchestrator (runs systems in order)
│   │   │   └── config.rs               # Simulation constants (gravity, tick rate, drag tables)
│   │   ├── ecs/                        # Entity-Component System
│   │   │   ├── mod.rs
│   │   │   ├── world.rs                # Entity storage, component arrays, entity creation/destruction
│   │   │   ├── components.rs           # All component struct definitions
│   │   │   └── entity.rs               # Entity ID type, generation tracking
│   │   ├── systems/                    # ECS systems (one file per system)
│   │   │   ├── mod.rs
│   │   │   ├── gravity.rs
│   │   │   ├── drag.rs
│   │   │   ├── thrust.rs
│   │   │   ├── movement.rs
│   │   │   ├── collision.rs
│   │   │   ├── detonation.rs
│   │   │   ├── shockwave.rs
│   │   │   ├── mirv_split.rs
│   │   │   ├── damage.rs
│   │   │   ├── detection.rs
│   │   │   ├── wave_spawner.rs
│   │   │   ├── cleanup.rs
│   │   │   └── state_snapshot.rs
│   │   ├── campaign/                   # Strategic campaign layer
│   │   │   ├── mod.rs
│   │   │   ├── territory.rs            # Region definitions, expansion logic, terrain effects
│   │   │   ├── economy.rs              # Resource generation, spending, cost tables
│   │   │   ├── upgrades.rs             # Tech tree, interceptor types, radar upgrades
│   │   │   ├── wave_composer.rs        # Wave difficulty scaling, enemy composition per wave
│   │   │   └── weather.rs              # Weather generation, radar/visibility effects
│   │   ├── state/                      # Game state types
│   │   │   ├── mod.rs
│   │   │   ├── game_state.rs           # Top-level game state (current phase, tick count, etc.)
│   │   │   ├── campaign_state.rs       # Territory, resources, upgrades, population
│   │   │   ├── wave_state.rs           # Current wave number, active entities, score
│   │   │   └── snapshot.rs             # Serializable state snapshot for frontend transmission
│   │   ├── events/                     # Backend event definitions
│   │   │   ├── mod.rs
│   │   │   └── game_events.rs          # All event types (detonation, impact, detection, etc.)
│   │   └── persistence/                # Save/load
│   │       ├── mod.rs
│   │       └── save_manager.rs         # Serialize/deserialize campaign state, save slots
│   └── tests/                          # Rust integration tests
│       ├── physics_tests.rs            # Trajectory accuracy, blast propagation validation
│       ├── determinism_tests.rs        # Same seed → same outcome verification
│       └── campaign_tests.rs           # Economy balance, wave composition tests
│
├── src/                                # TypeScript frontend (PixiJS + UI)
│   ├── main.ts                         # Frontend entry point, Tauri event listeners, app bootstrap
│   ├── renderer/                       # PixiJS rendering layer
│   │   ├── GameRenderer.ts             # Top-level PixiJS Application, stage management
│   │   ├── TacticalView.ts             # Wave gameplay rendering (missiles, arcs, detonations)
│   │   ├── StrategicView.ts            # Campaign map rendering (regions, batteries, radar coverage)
│   │   ├── HUD.ts                      # Heads-up display (ammo counts, radar status, wave info)
│   │   ├── Interpolator.ts             # Smooths between physics snapshots for render frames
│   │   ├── entities/                   # Renderable entity classes
│   │   │   ├── MissileRenderer.ts      # Trail drawing, re-entry glow effect
│   │   │   ├── InterceptorRenderer.ts  # Predicted arc overlay, active flight trail
│   │   │   ├── DetonationRenderer.ts   # Expanding shockwave ring, bloom trigger
│   │   │   ├── CityRenderer.ts         # City skyline, damage states, population indicator
│   │   │   ├── BatteryRenderer.ts      # Battery position marker, ammo gauge
│   │   │   └── RadarRenderer.ts        # Radar sweep effect, detection range circle
│   │   ├── effects/                    # Visual effects
│   │   │   ├── ParticleManager.ts      # PixiJS particle emitter pool for explosions, weather
│   │   │   ├── WeatherOverlay.ts       # Cloud layers, rain static, wind streaks
│   │   │   └── ImpactEffect.ts         # Ground impact flash, city damage transition
│   │   └── shaders/                    # Custom WebGL shaders
│   │       ├── crt.frag                # CRT curvature distortion, scanlines, vignetting
│   │       ├── phosphor.frag           # Phosphor glow / bloom pass
│   │       └── filmgrain.frag          # Film grain noise overlay
│   ├── input/                          # Player input handling
│   │   ├── InputManager.ts             # Mouse + keyboard event capture, keybinding config
│   │   ├── TacticalInput.ts            # Click-to-target, drag-to-adjust, battery selection
│   │   └── StrategicInput.ts           # Map interaction, placement mode, upgrade menus
│   ├── ui/                             # HTML/CSS UI panels (strategic phase)
│   │   ├── UpgradePanel.ts             # Tech tree / upgrade selection interface
│   │   ├── TerritoryPanel.ts           # Expansion options, region info
│   │   ├── IntelBriefing.ts            # Pre-wave intel readout (weather, threat assessment)
│   │   ├── WaveSummary.ts              # Post-wave results screen
│   │   └── MainMenu.ts                 # Title screen, save slot selection, settings
│   ├── state/                          # Frontend state management
│   │   ├── RenderState.ts              # Current entity positions, interpolated for rendering
│   │   ├── UIState.ts                  # Menu state, selected battery, zoom level
│   │   └── AudioState.ts               # Current audio context, active sounds
│   ├── audio/                          # Audio management
│   │   ├── AudioManager.ts             # Web Audio API wrapper, spatial audio
│   │   ├── SoundBank.ts                # Sound effect registry and loading
│   │   └── MusicController.ts          # Soundtrack playback, dynamic intensity
│   ├── bridge/                         # Tauri IPC bridge
│   │   ├── commands.ts                 # Typed wrappers around Tauri invoke() calls
│   │   └── events.ts                   # Typed Tauri event listeners and handlers
│   └── types/                          # Shared TypeScript type definitions
│       ├── snapshot.ts                 # Game state snapshot types (mirrors Rust snapshot.rs)
│       ├── commands.ts                 # Command payload types (mirrors Rust command handlers)
│       └── events.ts                   # Event payload types (mirrors Rust game_events.rs)
│
├── assets/                             # Game assets
│   ├── audio/
│   │   ├── sfx/                        # Sound effects (launch, detonation, impact, siren)
│   │   └── music/                      # Soundtrack tracks
│   ├── fonts/                          # Military-stencil typeface, teletype font
│   └── data/
│       ├── terrain_maps/               # Region terrain definitions (JSON or binary)
│       ├── wave_tables/                # Wave composition data (enemy counts, types, timings)
│       └── upgrade_trees/              # Interceptor and radar upgrade definitions
│
├── docs/                               # Project documentation
│   ├── deterrence-prd.md
│   ├── product_context.md
│   └── system_patterns.md
│
├── package.json                        # Frontend dependencies (PixiJS, TypeScript, Vite)
├── tsconfig.json                       # TypeScript configuration
├── vite.config.ts                      # Vite bundler configuration (Tauri plugin)
└── README.md
```

---

## Component Relationships

### Data Flow During Wave (Tactical Phase)

```
Player clicks sky
       │
       ▼
InputManager.ts ──► commands.ts ──► [Tauri IPC] ──► commands/tactical.rs
                                                          │
                                                          ▼
                                                    Validates:
                                                    - Battery has ammo?
                                                    - Target in range?
                                                    - Battery not reloading?
                                                          │
                                                          ▼
                                                    Creates Interceptor entity
                                                    (Transform + Velocity + Ballistic
                                                     + Interceptor + Warhead components)
                                                          │
                                                          ▼
                                               ┌─── Simulation Tick ───┐
                                               │                       │
                                               │  ThrustSystem         │
                                               │  GravitySystem        │
                                               │  DragSystem           │
                                               │  MovementSystem       │
                                               │  MirvSplitSystem      │
                                               │  CollisionSystem      │
                                               │  DetonationSystem ────┼──► Emits "game:detonation" event
                                               │  ShockwaveSystem      │
                                               │  DamageSystem ────────┼──► Emits "game:impact" / "game:city_damaged"
                                               │  DetectionSystem      │
                                               │  CleanupSystem        │
                                               │  StateSnapshotSystem ─┼──► Emits "game:state_snapshot"
                                               │                       │
                                               └───────────────────────┘
                                                          │
                                                   [Tauri Events]
                                                          │
                                                          ▼
events.ts ──► Interpolator.ts ──► TacticalView.ts ──► PixiJS Stage ──► Screen
                                        │
                                        ├──► DetonationRenderer (on detonation event)
                                        ├──► ParticleManager (explosions, shockwaves)
                                        └──► HUD update (ammo, radar contacts)
```

### Data Flow During Strategic Phase

```
Player clicks "Expand to Region 4"
       │
       ▼
StrategicInput.ts ──► commands.ts ──► [Tauri IPC] ──► commands/strategic.rs
                                                             │
                                                             ▼
                                                       Validates:
                                                       - Region adjacent?
                                                       - Sufficient resources?
                                                       - Region not already owned?
                                                             │
                                                             ▼
                                                       campaign/territory.rs
                                                       - Adds region to player territory
                                                       - Reveals new battery positions
                                                       - Registers new cities + population
                                                             │
                                                             ▼
                                                       campaign/economy.rs
                                                       - Deducts expansion cost
                                                       - Recalculates resource generation
                                                             │
                                                             ▼
                                                       campaign/wave_composer.rs
                                                       - Adjusts future wave patterns
                                                         for new territory shape
                                                             │
                                                             ▼
                                                       Emits "campaign:state_update"
                                                             │
                                                      [Tauri Event]
                                                             │
                                                             ▼
                                    events.ts ──► StrategicView.ts ──► Map re-render
                                                        │
                                                        ├──► TerritoryPanel update
                                                        └──► IntelBriefing update
```

### Component Ownership Summary

| Component | Owns | Communicates With |
|---|---|---|
| `game_loop.rs` | Phase state machine, tick scheduling | All systems, command handlers, event emitters |
| `simulation.rs` | System execution order | All ECS systems |
| `world.rs` | Entity storage, component arrays | All ECS systems (read/write) |
| `campaign/territory.rs` | Region map, terrain data | economy.rs, wave_composer.rs, weather.rs |
| `campaign/economy.rs` | Resource pool, cost tables | territory.rs, upgrades.rs |
| `campaign/wave_composer.rs` | Enemy wave definitions | territory.rs (for attack vectors), weather.rs |
| `GameRenderer.ts` | PixiJS Application, render loop | TacticalView, StrategicView, HUD |
| `Interpolator.ts` | Snapshot buffer, lerp state | GameRenderer (provides interpolated positions) |
| `InputManager.ts` | Raw input events, keybinds | TacticalInput, StrategicInput |
| `bridge/commands.ts` | IPC command dispatch | Rust command handlers (via Tauri invoke) |
| `bridge/events.ts` | IPC event subscriptions | Rust event emitters (via Tauri listen) |

---

## Key Architectural Decisions

### 1. Physics in Rust, Rendering in JS

**Decision:** The physics simulation runs entirely in the Rust backend. The PixiJS frontend never calculates physics — it only renders state it receives.

**Rationale:**
- Determinism is a hard requirement (seeded waves must produce identical outcomes). JavaScript's floating-point behavior, garbage collection pauses, and event loop timing make deterministic simulation unreliable. Rust's predictable execution model and lack of GC eliminates this class of bugs entirely.
- Performance headroom for saturation attacks. Late-game waves can have 100+ simultaneous entities with pairwise shockwave interaction checks. Rust handles this without breaking a sweat; JS would require careful optimization and might still stutter.
- Clean separation of concerns. Game logic bugs can be tested and debugged in Rust without a browser. Rendering bugs can be debugged in the browser without touching game logic.

**Tradeoff:** IPC overhead. Every physics tick requires serializing state and sending it across the Tauri bridge. Mitigation: snapshots are compact (entity positions + velocities, not full component data), and Tauri's IPC is fast enough for 60 Hz state transfer of this payload size.

### 2. Fixed Timestep Simulation with Render Interpolation

**Decision:** The physics simulation runs at a fixed tick rate (60 Hz target) independent of the display refresh rate. The frontend interpolates between the two most recent snapshots for smooth rendering.

**Rationale:**
- Fixed timestep ensures identical simulation results regardless of the player's hardware. A player running at 144 Hz sees smoother animation but the missiles follow the exact same paths as a player at 30 Hz.
- Interpolation prevents visual stutter when the render rate and physics rate don't align. The frontend holds two snapshots and lerps entity positions based on how far between ticks the current render frame falls.
- Decouples simulation correctness from rendering performance. If the renderer hitches (shader compilation, particle burst), the simulation continues unaffected.

**Tradeoff:** Adds one tick of visual latency (entities are always rendered slightly behind their "true" position). At 60 Hz physics this is ~16ms — imperceptible for a game where intercepts are planned, not twitch-reacted.

### 3. Lightweight Custom ECS over Framework (Bevy, etc.)

**Decision:** Build a simple, purpose-specific ECS in Rust rather than using an existing game framework.

**Rationale:**
- Deterrence is a 2D physics simulation with ~15 component types and ~14 systems. Bevy, Amethyst, or similar frameworks bring massive dependency trees, steep learning curves, and architectural opinions that would fight the Tauri shell integration.
- The ECS needs to be serialization-friendly (for save/load and IPC snapshots). A hand-rolled ECS with serde derives on all components makes this trivial. Framework ECS storage formats are often opaque.
- Full control over system execution order is critical — the order in which physics systems run (gravity before drag, detonation before shockwave propagation) affects correctness. Frameworks that parallelize or reorder systems would require careful constraint specification.
- Fewer dependencies = faster compile times, smaller binary, simpler debugging.

**Tradeoff:** No free rendering, no built-in asset pipeline, no editor. These are non-issues because rendering is handled by PixiJS, assets are simple (JSON data + audio files), and the game doesn't need a visual editor.

### 4. Tauri v2 over Electron

**Decision:** Use Tauri as the application shell instead of Electron.

**Rationale:**
- Tauri uses the OS native webview (WebView2 on Windows, WebKit on macOS/Linux) instead of bundling Chromium. Binary size is ~5-10 MB vs. ~150+ MB for Electron.
- Rust backend is a first-class citizen in Tauri, not a bolted-on native module. The physics engine lives naturally in the same Rust process as the Tauri shell.
- Lower memory footprint — important for a game that should run comfortably alongside other applications.
- Tauri v2's mobile support (iOS/Android) provides a future porting path without re-architecting.

**Tradeoff:** WebView rendering inconsistencies across platforms (WebKit on Linux can behave differently from WebView2 on Windows). Mitigation: PixiJS abstracts over WebGL and handles cross-browser differences. CRT shaders will need testing across all three platform webviews.

### 5. Event-Driven IPC over Polling

**Decision:** The backend pushes state snapshots and game events to the frontend via Tauri's event system, rather than the frontend polling for state.

**Rationale:**
- The simulation runs at its own tick rate — the frontend shouldn't drive when state updates happen.
- Discrete events (detonation, impact, city destroyed) need to arrive at the exact moment they occur for VFX and audio to sync properly. Polling would miss or batch events.
- Tauri's event system is asynchronous and non-blocking on both sides. The frontend's render loop and the backend's physics loop run independently.

**Tradeoff:** Event ordering must be carefully managed. If a "detonation" event and a "state_snapshot" event arrive out of order, the frontend might try to render an explosion for an entity that doesn't exist in the snapshot yet. Mitigation: include a tick counter in all events so the frontend can sequence them correctly.

### 6. JSON Serialization for IPC and Save Data

**Decision:** Use serde_json for both IPC state transfer and save file serialization, at least initially.

**Rationale:**
- JSON is human-readable, which is invaluable for debugging during development. You can inspect save files and IPC payloads directly.
- serde_json is the most battle-tested serialization crate in the Rust ecosystem.
- Tauri's IPC natively supports JSON payloads — no custom serialization layer needed.

**Tradeoff:** JSON is larger and slower to parse than binary formats (MessagePack, bincode). If 60 Hz state snapshots with 100+ entities cause measurable IPC latency, switch to MessagePack (serde-compatible, drop-in replacement, ~2x faster serialization). This is an optimization to make later if profiling shows it's needed, not an upfront decision.

---

## Testing Strategy

### Rust Backend

- **Unit tests per system:** Each ECS system has isolated tests verifying correct physics behavior (e.g., gravity system produces expected velocity after N ticks, drag system correctly scales with altitude)
- **Determinism tests:** Run identical wave seeds twice, assert byte-identical final state. This is the most critical test category — if determinism breaks, seeded waves and replay become unreliable.
- **Integration tests:** Full simulation runs of predefined wave scenarios, asserting expected outcomes (e.g., "Wave 5 with seed X, no player input → missiles impact cities A, B, C at ticks T1, T2, T3")
- **Balance tests:** Automated playthroughs with simple AI strategies to validate difficulty curves, resource pacing, and expansion timing targets

### TypeScript Frontend

- **Interpolation tests:** Verify smooth position interpolation between snapshots, handle edge cases (entity created, entity destroyed between snapshots)
- **Input mapping tests:** Verify correct command generation from mouse/keyboard input sequences
- **Visual regression:** Screenshot-based tests for CRT shader output across different viewport sizes

### Cross-Layer

- **Round-trip IPC tests:** Send a command from frontend, verify backend processes it and emits correct events, verify frontend receives and handles events
- **Performance benchmarks:** Measure IPC latency and render frame time during worst-case scenarios (100+ entities, multiple simultaneous detonations, full CRT shader pipeline)

---

*"Two systems, one truth, zero desync."*
