# Deterrence — Phased Implementation Plan

## Context

Deterrence is a physics-based missile defense game (Tauri v2 + Rust backend / PixiJS v8 frontend) with comprehensive design documentation but zero implementation code. This plan breaks the full build into 9 phases, each producing a testable milestone. The guiding principle: build from the inside out — Rust ECS core first, then IPC bridge, then rendering, then gameplay, then depth. **Phase 3 delivers the first playable** — a minimal Missile Command loop with real ballistic physics.

Design docs: [project_brief.md](project_brief.md), [product_context.md](product_context.md), [system_patterns.md](system_patterns.md)

---

## Phase 1: Project Scaffolding & Build Pipeline (Week 1)

**Goal:** Tauri v2 app compiles, opens a window, and proves bidirectional IPC works.

### Rust side
- `src-tauri/Cargo.toml` — tauri v2, serde, serde_json, glam, rand, rand_chacha
- `src-tauri/tauri.conf.json` — window config (1280×720, "Deterrence")
- `src-tauri/src/main.rs` — Tauri entry point
- `src-tauri/src/lib.rs` — module declarations (empty stubs for commands, engine, ecs, systems, state, events)
- `src-tauri/src/commands/mod.rs` — a "ping" command returning JSON
- `src-tauri/src/events/game_events.rs` — stub event types with serde derives
- `src-tauri/src/state/game_state.rs` — `GamePhase` enum, `GameState` struct with phase + tick counter

### TypeScript side
- `package.json` — pixi.js v8, @tauri-apps/api, typescript, vite
- `tsconfig.json`, `vite.config.ts`, `index.html`
- `src/main.ts` — bootstrap PixiJS Application (black canvas), register Tauri event listener, invoke ping
- `src/bridge/commands.ts` — typed invoke wrapper
- `src/bridge/events.ts` — typed event listener
- `src/types/snapshot.ts`, `src/types/commands.ts`, `src/types/events.ts` — placeholder interfaces

### Milestone
- `cargo test && cargo clippy && npm run build` all pass
- `cargo tauri dev` opens window with black PixiJS canvas
- Console shows successful IPC round-trip

---

## Phase 2: ECS Core & Physics Simulation (Weeks 2–3)

**Goal:** Functioning ECS with core physics systems running in a fixed-timestep loop. No rendering — validated entirely through Rust tests.

### ECS foundation
- `src-tauri/src/ecs/entity.rs` — `EntityId` (index + generation), `EntityAllocator`
- `src-tauri/src/ecs/components.rs` — component structs with serde derives: `Transform`, `Velocity`, `Ballistic`, `Warhead`, `Interceptor`, `Lifetime`, `Health`, `ReentryGlow`
- `src-tauri/src/ecs/world.rs` — `World` with `Vec<Option<T>>` SoA storage, spawn/despawn/add_component/get/get_mut/query methods

### Physics systems (one file each in `src-tauri/src/systems/`)
- `gravity.rs` — gravitational acceleration on all ballistic entities
- `drag.rs` — altitude-dependent atmospheric drag (exponential density falloff)
- `thrust.rs` — apply thrust to interceptors during burn phase
- `movement.rs` — Euler integration (velocity → position)
- `cleanup.rs` — despawn expired/out-of-bounds entities
- `state_snapshot.rs` — build serializable `StateSnapshot`

### Engine
- `src-tauri/src/engine/config.rs` — constants: tick rate (60Hz), gravity, world dimensions, drag tables
- `src-tauri/src/engine/simulation.rs` — `Simulation` struct, `tick()` runs systems in order, returns `StateSnapshot`
- `src-tauri/src/state/snapshot.rs` — `StateSnapshot`, `EntitySnapshot`, `EntityType`

### Tests (`src-tauri/tests/`)
- `physics_tests.rs` — freefall matches kinematic equations, drag produces terminal velocity, thrust accelerates then goes ballistic, 45° launch lands at expected range, cleanup removes expired entities
- `determinism_tests.rs` — identical initial conditions → byte-identical snapshots

### Milestone
Correct, deterministic ballistic trajectories validated by automated tests. Spawn a missile, watch it arc under gravity and drag — all in Rust tests, no UI needed.

---

## Phase 3: First Playable — Minimal Missile Command Loop (Weeks 4–5)

**Goal:** Enemy missiles arc down toward cities. Player clicks to launch interceptors. Detonations destroy nearby missiles. Cities take damage. This is the "Missile Command in a Tauri window" milestone.

### Rust — new systems & game loop
- `src-tauri/src/engine/game_loop.rs` — fixed-timestep loop on background thread, accumulator-based, emits `StateSnapshot` via Tauri events each tick, emits discrete events (detonation, impact, city_damaged)
- `src-tauri/src/systems/wave_spawner.rs` — spawns enemy missiles from top edge aimed at random cities, seeded ChaChaRng, simple wave definition (count, speed range, delay)
- `src-tauri/src/systems/collision.rs` — proximity check between shockwaves and entities
- `src-tauri/src/systems/detonation.rs` — interceptors detonate at target coords, warheads detonate on ground impact, spawns shockwave entities
- `src-tauri/src/systems/shockwave.rs` — expand radius, inverse-square falloff, expire at max radius
- `src-tauri/src/systems/damage.rs` — check ground detonations against city positions, reduce city Health
- `src-tauri/src/systems/input_system.rs` — process queued player commands, spawn interceptor entities

### Rust — commands & state
- `src-tauri/src/commands/tactical.rs` — `launch_interceptor { battery_id, target_x, target_y }`
- `src-tauri/src/commands/campaign.rs` — `start_wave` (simple version)
- `src-tauri/src/state/wave_state.rs` — wave number, missiles remaining/destroyed, cities alive
- Hardcoded initial world: 3 cities, 2 batteries with 10 interceptors each

### TypeScript — rendering pipeline
- `src/renderer/GameRenderer.ts` — PixiJS Application, stage management, render loop
- `src/renderer/TacticalView.ts` — entity map (EntityId → DisplayObject), create/destroy visuals from snapshots, ground line
- `src/renderer/Interpolator.ts` — holds two snapshots, lerps positions by alpha factor, handles entity creation/destruction
- `src/renderer/entities/MissileRenderer.ts` — bright dot + trailing line (amber/red)
- `src/renderer/entities/InterceptorRenderer.ts` — dot + trail (green) + target crosshair
- `src/renderer/entities/DetonationRenderer.ts` — expanding circle, brightness falloff, fade on expire
- `src/renderer/entities/CityRenderer.ts` — geometric skyline, health indicator, damage states
- `src/renderer/entities/BatteryRenderer.ts` — icon + ammo count
- `src/renderer/HUD.ts` — wave number, cities remaining, ammo per battery

### TypeScript — input & IPC
- `src/input/InputManager.ts` — mouse click → world coords → nearest battery → invoke launch
- `src/input/TacticalInput.ts` — click = launch from nearest battery
- `src/bridge/commands.ts` — `launchInterceptor()`, `startWave()`
- `src/bridge/events.ts` — `onStateSnapshot()`, `onDetonation()`, `onImpact()`, `onCityDamaged()`, `onWaveComplete()`
- `src/types/` — full snapshot, command, event types mirroring Rust

### Tests
- Rust: wave spawner produces correct missile count, collision detects proximity, detonation triggers at target, damage applies to cities
- Integration: full wave with no input → all missiles impact; full wave with scripted launches → expected kills
- Determinism: same seed + same inputs = identical outcome
- **Manual playtest: open app, see missiles arcing down, click to intercept, watch explosions, see cities get hit**

### Milestone
**A playable Missile Command clone with real ballistic physics.** The entire Rust→IPC→PixiJS pipeline works end-to-end.

---

## Phase 4: Predicted Arc Overlay & Tactical Polish (Week 6)

**Goal:** The predicted arc overlay — the signature UX element that teaches physics — plus battery selection, drag-to-adjust targeting, and wave results.

### Rust
- `commands/tactical.rs` — `predict_arc` command: runs lightweight forward simulation, returns `Vec<(f32, f32)>` points + time-to-target. Read-only, doesn't modify state.
- `commands/tactical.rs` — `select_battery` command, battery range validation
- `engine/game_loop.rs` — full phase transitions: WaveActive → WaveResult → Strategic (loops back to WaveActive for now)
- Emit `game:wave_complete` with summary data

### TypeScript
- `InterceptorRenderer.ts` — predicted arc (dotted line from battery to target), updates on mouse move, time-to-intercept label
- `TacticalInput.ts` — mouse move = throttled arc prediction (~15Hz), click = commit, click+drag = adjust target, Tab = cycle batteries, 1-9 = direct battery select, right-click = cancel
- `ui/WaveSummary.ts` — post-wave results panel (HTML/CSS): missiles destroyed, cities hit, interceptors used, efficiency, "Continue" button
- `HUD.ts` — selected battery indicator, ammo depletion visual, active missile count

### Milestone
Player sees predicted arcs before committing. Can select batteries, drag to adjust. Wave results provide feedback. The game teaches its own physics.

---

## Phase 5: Campaign Foundation — Territory, Economy, Strategic Phase (Weeks 7–8)

**Goal:** Between waves, the player sees a territory map, earns resources, places batteries, upgrades radar, expands territory.

### Rust — campaign systems
- `campaign/territory.rs` — `Region` (terrain type, cities, battery slots, adjacency), `TerritoryMap`, expansion logic
- `campaign/economy.rs` — resource generation from surviving cities, cost tables (batteries, restocking, expansion, repairs)
- `campaign/upgrades.rs` — basic version: Standard interceptor only, radar range levels, ammo capacity
- `campaign/wave_composer.rs` — difficulty curve from wave number, territory-aware attack directions, seeded RNG
- `state/campaign_state.rs` — territory, economy, upgrades, wave number

### Rust — commands & detection
- `commands/strategic.rs` — `place_battery`, `upgrade_radar`, `expand_territory`, `repair_city`, `start_wave`
- `systems/detection.rs` — basic radar: entities within range appear in snapshot, entities outside are invisible

### TypeScript — strategic view & UI
- `renderer/StrategicView.ts` — campaign map with regions (colored by control state), battery/city/radar icons
- `renderer/GameRenderer.ts` — switches between TacticalView and StrategicView by phase
- `input/StrategicInput.ts` — click regions, place batteries, trigger upgrades
- `ui/UpgradePanel.ts` — radar/ammo upgrades with costs
- `ui/TerritoryPanel.ts` — region info, expansion button
- `ui/IntelBriefing.ts` — wave preview, "Begin Wave" button

### Milestone
Multi-wave campaign with strategic decisions between waves. Resources matter. Territory decisions shape future tactical situations. **The game has a loop.**

---

## Phase 6: Advanced Combat — Chain Reactions, MIRVs, Interceptor Types (Weeks 9–10)

**Goal:** Emergent tactical depth via chain reactions, MIRV warheads, and 4 interceptor types.

### Chain reactions
- `systems/shockwave.rs` — shockwaves check against ALL entities (including other warheads), destroy or deflect based on proximity/force
- `systems/collision.rs` — deflection physics (modify velocity by shockwave force direction)
- Destroyed warheads trigger their own detonation → cascading chains

### MIRVs
- `ecs/components.rs` — `MirvCarrier { child_count, split_altitude, spread_angle }`
- `systems/mirv_split.rs` — check altitude, spawn N child warheads on divergent trajectories, despawn carrier
- `campaign/wave_composer.rs` — MIRVs appear wave 26+

### Interceptor types
- `InterceptorType` enum: Standard, Sprint (fast/short), Exoatmospheric (slow/high ceiling/wide blast), AreaDenial (lingering zone)
- `systems/thrust.rs` — different thrust profiles per type
- `systems/detonation.rs` — different detonation behavior per type
- `campaign/upgrades.rs` — full tech tree: unlock types by wave tier, per-type upgrades (thrust, yield, guidance)

### TypeScript
- MIRV split animation, chain reaction cascade visuals, interceptor type trail styles
- `ui/UpgradePanel.ts` — interceptor unlock/upgrade tree

### Milestone
Chain reactions produce "read the cluster" moments. MIRVs force pre-split vs. post-split dilemmas. Four interceptor types give meaningful tactical choice. Emergent physics creates surprising, learnable moments.

---

## Phase 7: Detection, Weather & Fog of War (Week 11)

**Goal:** The player sees only what radar and visual observation detect. Weather degrades both.

### Detection system
- `systems/detection.rs` — full rewrite: `RadarSource` (range × weather_resistance), `VisualSource` (reentry glow below altitude threshold × weather visibility), `DetectionManager` merging contacts
- `RadarContact { entity_id, position, velocity, confidence }` vs `GlowContact { position, intensity }` (no velocity)
- `systems/state_snapshot.rs` — snapshot now contains only detected entities

### Weather
- `campaign/weather.rs` — `WeatherCondition` enum (Clear/Overcast/Storm/Severe), per-wave RNG generation
- Effects: radar_multiplier (1.0/0.7/0.4/0.3), glow_visibility (1.0/0.3/0.0/0.0), wind vector
- `systems/drag.rs` — wind force during storms affects interceptor trajectories (visible in predicted arc)
- Weather introduced wave ~16

### TypeScript
- Radar contacts show full trail + velocity vector; glow contacts show only a bright point
- `renderer/entities/RadarRenderer.ts` — sweep effect, range circle
- `renderer/effects/WeatherOverlay.ts` — cloud layers, rain particles, reduced visibility
- `ui/IntelBriefing.ts` — weather forecast

### Milestone
Information uncertainty as a game mechanic. Radar is a resource. Weather creates varied wave experiences. Clear weather rewards vigilance; storms create genuine tension.

---

## Phase 8: Persistence, Save/Load & Campaign Completion (Weeks 12–13)

**Goal:** Complete, finishable campaign with save/load, full 40+ wave progression, territory contraction, and end states.

### Save/load
- `persistence/save_manager.rs` — serialize `CampaignState` to JSON in Tauri app data dir, 3-5 save slots, auto-save between waves, version number for migration

### Campaign completion
- `engine/game_loop.rs` — RegionLost → Contraction → Strategic; homeland lost → CampaignOver
- `campaign/territory.rs` — region loss when all cities destroyed, territory contraction
- `campaign/wave_composer.rs` — full 40+ wave escalation through all difficulty tiers

### TypeScript
- `ui/MainMenu.ts` — title screen, new/continue/load/settings, save slot display
- `ui/WaveSummary.ts` — region loss notification, campaign stats, campaign over screen
- `renderer/StrategicView.ts` — region loss/expansion animations

### Milestone
A complete, finishable campaign. Start → 40+ waves → conclusion. Save/load works. Strategic retreat is viable. The game has a beginning, middle, and end.

---

## Phase 9: CRT Aesthetic, Audio & Visual Polish (Weeks 14–16)

**Goal:** Transform the functional game into the atmospheric Cold War bunker experience.

### CRT shader pipeline
- `src/renderer/shaders/crt.frag` — scanlines, barrel distortion, vignetting, chromatic aberration
- `src/renderer/shaders/phosphor.frag` — phosphor glow/bloom on bright objects, detonation bloom pulse
- `src/renderer/shaders/filmgrain.frag` — animated noise, intensity increases in storms
- Applied as PixiJS post-processing filters: scene → phosphor → CRT → filmgrain → output

### Particle effects
- `src/renderer/effects/ParticleManager.ts` — emitter pool: explosion rings, launch smoke, missile trails, impact flash, chain reaction cascades, screen shake
- `src/renderer/effects/ImpactEffect.ts` — ground impact vis, city damage transition (skyline crumbles, lights out)

### Audio
- `src/audio/AudioManager.ts` — Web Audio API, spatial panning, volume/mute
- `src/audio/SoundBank.ts` — SFX: launch, detonation, impact, city hit/destroyed, radar ping, siren, MIRV split, wave klaxon, UI clicks
- `src/audio/MusicController.ts` — bunker ambience, dynamic intensity (ambient → tense → urgent), phase-based transitions

### UI polish
- Military-stencil typography, teletype text effect for briefings
- Analog gauge indicators for ammo/radar
- Smooth view transitions, loading screen
- Settings: volume, display, CRT toggle (accessibility)
- Green/amber phosphor color palette

### Milestone
The game looks and sounds like the design vision. CRT aesthetic, Cold War audio atmosphere, weight of detonations and city losses. Functional game becomes evocative experience.

---

## Phase Summary

| Phase | Weeks | Deliverable |
|-------|-------|-------------|
| 1 | 1 | Compiling Tauri+PixiJS app with working IPC |
| 2 | 2–3 | ECS + physics simulation (test-driven, no UI) |
| 3 | 4–5 | **First playable** — Missile Command loop with real physics |
| 4 | 6 | Predicted arc overlay, battery selection, wave results |
| 5 | 7–8 | Campaign: territory, economy, strategic phase |
| 6 | 9–10 | Chain reactions, MIRVs, 4 interceptor types |
| 7 | 11 | Detection system, weather, fog of war |
| 8 | 12–13 | Save/load, full campaign, end states |
| 9 | 14–16 | CRT shaders, particles, audio, UI polish |

## Dependency Graph

```
Phase 1 (Scaffolding)
  └→ Phase 2 (ECS + Physics)
       └→ Phase 3 (First Playable) ← bridges backend to frontend
            └→ Phase 4 (Arc Overlay + Polish)
                 └→ Phase 5 (Campaign Foundation)
                      ├→ Phase 6 (Advanced Combat) ─┐
                      └→ Phase 7 (Detection+Weather) ┤  ← can be parallel
                                                      └→ Phase 8 (Persistence + Campaign Completion)
                                                           └→ Phase 9 (Visual Polish + Audio)
```

Phases 6 and 7 are independent of each other and can be developed in either order or in parallel.

## Key Risks

| Risk | Mitigation |
|------|------------|
| IPC performance at 60Hz with 100+ entities | Phase 1 proves bridge; if JSON is slow, swap to MessagePack (drop-in serde replacement) |
| Physics feel — correct math ≠ good game feel | Phase 2 is test-driven; Phase 3 gives immediate visual feedback; constants in config.rs are tunable |
| CRT shaders across platform WebViews | Deferred to Phase 9; PixiJS handles WebGL differences; shaders are optional |
| Campaign balance over 40+ waves | Data-driven wave tables + automated balance tests in Phases 6/8; deterministic RNG makes issues reproducible |
| Arc prediction IPC latency on mouse move | Throttle to ~15Hz; consider frontend-side fast approximation with backend correction |
