# Deterrence — Implementation Progress

## Phase 1: Project Scaffolding & Build Pipeline ✅

**Status:** Complete
**Date:** 2026-02-06

### Deliverables
- Tauri v2 app scaffolded with Rust 2024 edition backend + Vite/TypeScript/PixiJS frontend
- Bidirectional IPC working: `ping` command returns JSON from Rust to TypeScript
- All module directories created matching planned file structure
- Icons generated for all platforms

### Rust Side
- `src-tauri/Cargo.toml` — tauri v2, serde, serde_json, glam, rand, rand_chacha
- `src-tauri/tauri.conf.json` — 1280×720 window, "Deterrence"
- `src-tauri/src/main.rs` — Tauri entry point
- `src-tauri/src/lib.rs` — module declarations + Tauri builder with ping command
- `src-tauri/src/commands/mod.rs` — `ping` command returning `PingResponse` JSON
- `src-tauri/src/events/game_events.rs` — event types (Detonation, Impact, CityDamaged, WaveComplete)
- `src-tauri/src/state/game_state.rs` — `GamePhase` enum, `GameState` struct

### TypeScript Side
- `package.json` — pixi.js v8, @tauri-apps/api, typescript, vite
- `tsconfig.json`, `vite.config.ts`, `index.html`
- `src/main.ts` — PixiJS Application (black canvas + green ground line + title), IPC ping test
- `src/bridge/commands.ts` — typed invoke wrapper for `ping`
- `src/bridge/events.ts` — typed Tauri event listeners
- `src/types/snapshot.ts`, `src/types/commands.ts`, `src/types/events.ts` — interface definitions

### Milestone Verification
- `cargo test` — ✅ passes
- `cargo clippy` — ✅ 0 warnings
- `npm run build` (tsc + vite) — ✅ passes
- `cargo tauri dev` — ready to open window with PixiJS canvas + IPC round-trip

---

## Phase 2: ECS Core & Physics Simulation ✅

**Status:** Complete
**Date:** 2026-02-06

### Deliverables
- Custom lightweight ECS with generational entity IDs and SoA storage
- 5 physics systems running in correct order via Simulation orchestrator
- 19 automated tests: 6 unit, 10 physics, 3 determinism — all passing
- Deterministic simulation verified: identical inputs → byte-identical snapshots

### ECS Foundation
- `src-tauri/src/ecs/entity.rs` — `EntityId` (index + generation), `EntityAllocator` with alloc/dealloc/reuse
- `src-tauri/src/ecs/components.rs` — 11 component structs: Transform, Velocity, Ballistic, Warhead, Interceptor, Lifetime, Health, ReentryGlow, Shockwave, EntityKind, EntityMarker
- `src-tauri/src/ecs/world.rs` — `World` with `Vec<Option<T>>` SoA storage, spawn/despawn, alive_entities iterator

### Physics Systems
- `systems/gravity.rs` — gravitational acceleration on missiles & interceptors
- `systems/drag.rs` — exponential atmospheric density model, altitude-dependent drag
- `systems/thrust.rs` — interceptor thrust during burn phase, direction toward target
- `systems/movement.rs` — Euler integration (velocity → position), rotation tracking
- `systems/cleanup.rs` — despawn expired (lifetime) and out-of-bounds entities
- `systems/state_snapshot.rs` — build serializable StateSnapshot from world state

### Engine
- `engine/config.rs` — constants: 60Hz tick rate, gravity (9.81), world dimensions, drag model, interceptor/warhead defaults
- `engine/simulation.rs` — `Simulation` struct, `tick()` runs systems in order, returns `StateSnapshot`
- `state/snapshot.rs` — `StateSnapshot`, `EntitySnapshot`, `EntityType`, `EntityExtra`

### Test Results (19/19 passing)
**Unit tests (6):** entity allocator, world spawn/despawn, component storage, air density model
**Physics tests (10):** freefall kinematics, 45° range formula, projectile arcs, drag speed reduction, altitude-dependent drag, thrust acceleration, post-burn ballistic behavior, OOB cleanup, lifetime expiry, snapshot contents
**Determinism tests (3):** identical runs produce identical output, holds over 300 ticks, different tick counts diverge

---

## Phase 3: First Playable — Minimal Missile Command Loop ✅

**Status:** Complete
**Date:** 2026-02-06

### Deliverables
- 6 new game systems: wave spawner, collision, detonation, shockwave expansion, damage, input processing
- Game loop on background thread emitting state snapshots at 60Hz via Tauri events
- Full rendering pipeline: PixiJS tactical view with entity trails, shockwave rings, city/battery visuals, HUD
- Player input: click to launch interceptors from nearest battery at target position
- Wave lifecycle: start → spawn missiles → resolve → wave complete with results summary
- Chain reactions: shockwaves from destroyed warheads cascade over subsequent ticks
- 31 automated tests (12 new Phase 3 tests), 0 clippy warnings

### New Rust Systems
- `systems/wave_spawner.rs` — seeded RNG spawns enemy missiles from top edge aimed at random cities, configurable flight time for arc variety
- `systems/collision.rs` — shockwave proximity check against missiles, chain reaction shockwaves from destroyed warheads
- `systems/detonation.rs` — interceptor detonation at target (proximity + overshoot detection), missile ground impact at GROUND_Y
- `systems/shockwave_system.rs` — expand shockwave radius each tick up to max_radius
- `systems/damage.rs` — ground-level shockwaves damage nearby cities with linear distance falloff
- `systems/input_system.rs` — process player launch commands, validate battery ammo, spawn interceptor entities

### Engine Updates
- `engine/simulation.rs` — expanded with world setup, wave management, 10-system execution order, phase transitions, event collection
- `engine/game_loop.rs` — background thread with 60Hz fixed timestep, `GameEngine` handle with mpsc channel for commands
- `engine/config.rs` — 20+ new constants: city/battery positions, wave params, missile/interceptor properties, damage model
- `commands/tactical.rs` — `launch_interceptor` Tauri command
- `commands/campaign.rs` — `start_wave` Tauri command
- `state/wave_state.rs` — `WaveDefinition` (scales with wave number) + `WaveState` tracking
- `events/game_events.rs` — `GameEvent` enum wrapping all discrete events

### ECS Additions
- `BatteryState` component (ammo tracking)
- `Shockwave.damage_applied` flag for one-time ground damage
- `World.battery_states` storage vector

### TypeScript Rendering Pipeline
- `src/renderer/GameRenderer.ts` — PixiJS app orchestrator, event listeners, auto-starts first wave
- `src/renderer/TacticalView.ts` — entity lifecycle management (create/update/destroy), coordinate transform (Y-flip), trails for missiles/interceptors, shockwave rings, city skylines with health bars, battery ammo dots
- `src/renderer/HUD.ts` — wave number, cities remaining, incoming missile count, wave complete summary overlay
- `src/input/InputManager.ts` — click-to-launch (nearest battery), click-to-start between waves
- `src/bridge/commands.ts` — `launchInterceptor()`, `startWave()` invoke wrappers
- `src/main.ts` — bootstraps GameRenderer, connects phase tracking

### System Execution Order (per tick)
1. Input → 2. WaveSpawner → 3. Thrust → 4. Gravity → 5. Drag → 6. Movement → 7. Collision → 8. Detonation → 9. Shockwave → 10. Damage → 11. Cleanup → 12. WaveComplete check → 13. Snapshot

### Test Results (31/31 passing)
**Unit tests (6):** entity allocator, world spawn/despawn, component storage, air density model
**Physics tests (10):** freefall, 45° range, arcs, drag, altitude drag, thrust, post-burn ballistic, OOB cleanup, lifetime, snapshots
**Determinism tests (3):** identical runs, 300-tick stability, divergence detection
**Phase 3 tests (12):** world setup, wave spawning (count + components), interceptor launch + ammo, empty battery rejection, missile ground detonation, shockwave destruction + chain reaction, city damage, wave completion, deterministic waves, scripted intercepts, shockwave expansion

### Milestone Verification
- `cargo test` — ✅ 31 tests passing
- `cargo clippy` — ✅ 0 warnings
- `npm run build` (tsc + vite) — ✅ passes
- `cargo tauri dev` — ready: missiles arc, click to intercept, explosions cascade, cities take damage

---

## Phase 4: Predicted Arc Overlay & Tactical Polish ✅

**Status:** Complete
**Date:** 2026-02-06

### Deliverables
- Predicted arc overlay: mouse-move shows interceptor trajectory as dashed green line with time-to-intercept label
- Battery selection: Tab to cycle, 1/2 for direct select, visual highlight ring on selected battery
- Phase transitions: WaveActive → WaveResult → Strategic (explicit continue) → WaveActive
- Enhanced wave summary: missiles destroyed, impacted, interceptors used, efficiency %, "ENTER or Click to Continue"
- Interceptor launch tracking per wave for efficiency metrics
- 37 automated tests (6 new arc prediction tests), 0 clippy warnings

### New Rust Code
- `systems/arc_prediction.rs` — pure physics forward simulation for trajectory prediction (no ECS access), replicates thrust/gravity/drag/detonation physics, returns `ArcPrediction { points, time_to_target, reaches_target }`
- `commands/tactical.rs` — added `predict_arc` Tauri command (looks up battery position, runs arc prediction)
- `commands/campaign.rs` — added `continue_to_strategic` Tauri command
- `engine/game_loop.rs` — added `ContinueToStrategic` engine command variant, WaveResult→Strategic transition, StartWave now only accepts from Strategic phase
- `state/wave_state.rs` — added `interceptors_launched` field
- `systems/input_system.rs` — returns `u32` count of interceptors launched per tick
- `engine/simulation.rs` — tracks interceptors launched in wave state, includes in WaveCompleteEvent
- `events/game_events.rs` — added `interceptors_launched` to WaveCompleteEvent

### TypeScript Changes
- `src/input/InputManager.ts` — rewritten: battery selection state, mousemove tracking with ~15Hz throttled arc prediction IPC, keyboard shortcuts (Tab/1/2/Enter), right-click to cancel, phase-aware click handling (WaveActive→launch, WaveResult→continue, Strategic→startWave)
- `src/renderer/TacticalView.ts` — added arc overlay (dashed line + crosshair + time label), battery highlight ring, clearOverlays() method
- `src/renderer/HUD.ts` — added battery selection text (BAT-1/BAT-2 with ammo), enhanced wave summary with efficiency stats, phase-specific prompts
- `src/renderer/GameRenderer.ts` — consolidated snapshot listener, wired arc prediction and battery change callbacks, removed auto-start
- `src/main.ts` — simplified to just bootstrap GameRenderer
- `src/bridge/commands.ts` — added `predictArc()`, `continueToStrategic()` invoke wrappers
- `src/types/commands.ts` — added `ArcPrediction` interface
- `src/types/events.ts` — added `interceptors_launched` to WaveCompleteEvent

### Test Results (37/37 passing)
**Unit tests (6):** entity allocator, world spawn/despawn, component storage, air density model
**Physics tests (10):** freefall, 45° range, arcs, drag, altitude drag, thrust, post-burn ballistic, OOB cleanup, lifetime, snapshots
**Determinism tests (3):** identical runs, 300-tick stability, divergence detection
**Phase 3 tests (12):** world setup, wave spawning, interceptor launch, empty battery, ground detonation, shockwave destruction, city damage, wave completion, deterministic waves, scripted intercepts, shockwave expansion
**Phase 4 tests (6):** arc reaches straight up, arc reaches diagonal, unreachable target detection, arc starts at battery, reasonable point count, right battery arc

### Milestone Verification
- `cargo test` — ✅ 37 tests passing
- `cargo clippy` — ✅ 0 warnings
- `npm run build` (tsc + vite) — ✅ passes
- `cargo tauri dev` — ready: predicted arcs on mouse move, battery selection with Tab/1/2, wave summary with efficiency stats

---

## Phase 4b: Interceptor Physics Refinement ✅

**Status:** Complete
**Date:** 2026-02-06

### Deliverables
- Interceptors now feel like real missiles: rapid acceleration during burn, visible deceleration after burnout
- Burn state exposed to frontend via EntityExtra for visual differentiation
- Burn vs coast visual states: bright/large during thrust, dim/small when coasting

### Physics Tuning (config.rs)
- `INTERCEPTOR_THRUST`: 300 → **500** (66% stronger — rapid acceleration)
- `INTERCEPTOR_BURN_TIME`: 2.0s → **1.0s** (half the duration — punchy burst)
- `INTERCEPTOR_DRAG_COEFF`: 0.2 → **0.35** (75% more drag — visible deceleration after burnout)

### Burn State in Snapshots
- `state/snapshot.rs` — added `Interceptor { burn_remaining, burn_time }` variant to `EntityExtra`
- `systems/state_snapshot.rs` — interceptors now populate extra data from ECS `Interceptor` component
- `types/snapshot.ts` — added `InterceptorExtra` TypeScript interface

### Visual Differentiation (TacticalView.ts)
- **During burn:** larger dot (3-5px scaling with burn ratio), bright green + white core, orange exhaust glow
- **After burnout:** smaller dot (2.5px), dimmer green — clearly coasting
- **Trail:** brighter/wider trail (0x88ff44, 1.5px) during burn, normal trail (0x44ff44, 1px) when coasting

### Arc Prediction
- Automatically uses new constants (reads from same config) — no code changes needed

### Milestone Verification
- `cargo test` — ✅ 37 tests passing (no regressions)
- `cargo clippy` — ✅ 0 warnings
- `npm run build` — ✅ clean

---

## Phase 5: Campaign Foundation — Territory, Economy, Strategic Phase ✅

**Status:** Complete
**Date:** 2026-02-06

### Deliverables
- Territory system: 5 regions with adjacency graph, terrain types, city/battery positions
- Resource economy: income from surviving cities, costs for expansion/placement/restock/repair
- Campaign state persists across waves with sync_to_campaign/spawn_from_campaign cycle
- Wave composer: difficulty scales with wave_number AND owned territory size
- Strategic phase UI: territory map view, action panel, resource display, view switching
- Dynamic battery positions: arc prediction uses (battery_x, battery_y) instead of hardcoded IDs
- 65 automated tests (20 new Phase 5 tests), 0 clippy warnings

### Campaign Data Model (Rust)
- `campaign/territory.rs` — Region, TerrainType, CityDef, BatterySlot, define_regions() with 5 regions
- `campaign/economy.rs` — CostTable (place=100, restock=30, repair=2/hp), calculate_wave_income()
- `campaign/wave_composer.rs` — compose_wave() scales missile_count by territory + wave_number
- `state/campaign_state.rs` — CampaignState (resources, owned_regions, city_healths, battery_ammo), CampaignSnapshot

### Region Layout
| Region | Terrain | Cities | Battery Slots | Expansion Cost |
|--------|---------|--------|---------------|----------------|
| 0 Homeland | Plains | 3 (x=320,640,960) | 2 (x=160*,1120*) | — |
| 1 Western Highlands | Mountains | 1 (x=80) | 2 (x=40,240) | 150 |
| 2 Eastern Seaboard | Coastal | 1 (x=1200) | 1 (x=1060) | 200 |
| 3 Northern Plains | Plains | 2 (x=420,540) | 1 (x=480) | 250 |
| 4 Industrial Core | Urban | 1 (x=800) | 2 (x=720,880) | 300 |
\* = pre-occupied. Adjacency: 0↔1, 0↔2, 1↔3, 2↔4, 3↔4

### Simulation Updates (Rust)
- `engine/simulation.rs` — CampaignState field, setup_world() delegates to spawn_from_campaign(), rebuild_world(), sync_to_campaign(), apply_wave_income(), expand_region(), place_battery(), restock_battery(), repair_city(), build_campaign_snapshot()
- `engine/game_loop.rs` — New EngineCommand variants (ExpandRegion, PlaceBattery, RestockBattery, RepairCity, GetCampaignState), emits campaign:state_update events, ContinueToStrategic handler syncs campaign + calculates income
- `commands/campaign.rs` — 5 new Tauri commands: expand_region, place_battery, restock_battery, repair_city, get_campaign_state
- `commands/tactical.rs` — predict_arc signature changed to (battery_x, battery_y, target_x, target_y)

### TypeScript Frontend
- `src/types/campaign.ts` — CampaignSnapshot, RegionSnapshot, AvailableAction types
- `src/bridge/commands.ts` — expandRegion(), placeBattery(), restockBattery(), repairCity(), getCampaignState(), updated predictArc()
- `src/bridge/events.ts` — onCampaignUpdate() listener
- `src/renderer/StrategicView.ts` — Territory map with region circles, adjacency lines, action panel, resource/wave display, intel briefing
- `src/renderer/GameRenderer.ts` — View switching (Strategic→StrategicView, WaveActive/WaveResult→TacticalView), campaign event listener
- `src/renderer/HUD.ts` — Added resources display, updateResources() method
- `src/renderer/TacticalView.ts` — Added visible property, updated battery highlight for dynamic positions
- `src/input/InputManager.ts` — Dynamic battery positions from campaign, handleStrategicAction(), strategic phase click handling via StrategicView actions

### Test Results (65/65 passing)
**Unit tests (30):** entity, world, components, air density, territory (4), economy (4), wave_composer (3), campaign_state (7), arc_prediction (6)
**Determinism tests (3):** identical runs, 300-tick stability, divergence detection
**Phase 3 tests (12):** world setup, wave spawning, interceptor launch, empty battery, ground detonation, shockwave destruction, city damage, wave completion, deterministic waves, scripted intercepts, shockwave expansion
**Phase 5 tests (20):** campaign defaults, wave income (full + damaged), expand region (success + fail: resources/adjacency/owned), place battery (success + fail: occupied/unowned), restock battery (success + fail: full/resources), repair city (success + fail: full), campaign snapshot structure, world rebuild preserves state, full cycle, wave composer territory scaling, backward compatibility

### Milestone Verification
- `cargo test` — ✅ 65 tests passing
- `cargo clippy` — ✅ 0 warnings
- `npm run build` (tsc + vite) — ✅ passes
- `cargo tauri dev` — ready: strategic view with territory map, expand/place/restock/repair actions, wave income, view switching

---

## Phase 6: Advanced Combat — Interceptor Types, MIRVs, Chain Reactions, Tech Tree ✅

**Status:** Complete
**Date:** 2026-02-06

### Deliverables
- 4 interceptor types with distinct physics profiles (Standard, Sprint, Exoatmospheric, AreaDenial)
- MIRV warheads that split into multiple child missiles at configurable altitude
- Enhanced chain reactions with dual-zone shockwaves (destroy inner + deflect outer)
- Tech tree with unlock gates (wave-gated + resource cost) and per-type upgrades (thrust/yield/guidance)
- Full frontend integration: type-colored interceptors, MIRV visuals, tech tree in strategic view
- 111 automated tests (46 new Phase 6 tests), 0 clippy warnings

### Sub-phase 6A: Interceptor Type System
- `InterceptorType` enum: Standard, Sprint, Exoatmospheric, AreaDenial
- `InterceptorProfile` per-type physics: thrust, burn_time, ceiling, mass, drag_coeff, cross_section, yield_force, blast_radius
- **Standard:** 500 thrust, 1.0s burn, 700 ceiling, 40 blast — balanced
- **Sprint:** 900 thrust, 0.5s burn, 350 ceiling, 25 blast — fast/short range
- **Exoatmospheric:** 300 thrust, 2.5s burn, 900 ceiling, 70 blast — slow/high altitude
- **AreaDenial:** 400 thrust, 1.2s burn, 600 ceiling, 55 blast, 180-tick lingering shockwave
- Interceptor type flows through: launch command → ECS → physics → snapshot → frontend

### Sub-phase 6B: MIRV System
- `MirvCarrier` component: child_count, split_altitude, spread_angle
- `mirv_split.rs` system: detects descending carriers below split altitude, despawns carrier, spawns child missiles with fan-spread velocities
- Wave composer: MIRVs appear at wave 26+, scaling from 3 to 5 children at wave 35+
- `MirvSplitEvent` emitted for frontend visual effects
- System order: Movement → **MirvSplit** → Collision

### Sub-phase 6C: Chain Reactions Enhancement
- Dual-zone shockwave collision:
  - **Destroy zone** (dist < radius × 0.7): entity destroyed; missiles chain-react at 0.7× power, interceptors do NOT
  - **Deflect zone** (0.7 × radius ≤ dist < radius): push velocity away from center
- Deflection force: `sw.force × (1 - dist/radius) × 0.1 × DT`
- Aggregates deflections from multiple shockwaves per entity per tick

### Sub-phase 6D: Tech Tree & Upgrade System
- `TechTree` struct: unlocked_types + per-type upgrades (thrust/yield/guidance, max level 3)
- Unlock gates: Standard=wave1(free), Sprint=wave8($200), Exo=wave15($300), AreaDenial=wave22($400)
- Upgrade multipliers: +15% thrust/level, +20% blast radius/level
- `effective_profile()` applies upgrade multipliers to base InterceptorProfile
- `unlock_interceptor` and `upgrade_interceptor` Tauri commands
- Tech tree actions appear in strategic phase available actions

### Sub-phase 6E: Frontend Integration
- **Interceptor colors by type:** Standard=green, Sprint=cyan, Exo=magenta, AreaDenial=amber
- **MIRV carriers:** Larger 5px red dot with white core, pinkish trails
- **MIRV split effect:** Expanding red ring animation on split event
- **Type selection:** Q/W/E/R keys (only unlocked types), shown in HUD as BAT-1 [8/10] STD
- **Tech tree in strategic view:** Unlock and upgrade actions displayed with costs
- **Type-colored trails:** Each interceptor type has matching trail color

### New/Modified Files (Rust)
- **New:** `systems/mirv_split.rs`, `campaign/upgrades.rs`
- **Modified:** components, world, config, input_system, collision, detonation, arc_prediction, state_snapshot, snapshot, wave_state, wave_spawner, wave_composer, simulation, game_loop, game_events, tactical commands, campaign commands, lib

### New/Modified Files (TypeScript)
- **Modified:** snapshot types, event types, campaign types, commands bridge, events bridge, InputManager, TacticalView, StrategicView, HUD, GameRenderer

### Test Results (111/111 passing)
**Unit tests (43):** entity, world, components, air density, territory, economy, wave_composer, campaign_state, arc_prediction, upgrades (7 unit tests in upgrades.rs)
**Determinism tests (3):** identical runs, 300-tick stability, divergence detection
**Phase 3 tests (12):** world setup, wave spawning, interceptor launch, empty battery, ground detonation, shockwave destruction, city damage, wave completion, deterministic waves, scripted intercepts, shockwave expansion
**Phase 5 tests (20):** campaign defaults, wave income, expand/place/restock/repair operations, snapshot structure, world rebuild, full cycle, wave composer scaling, backward compatibility
**Phase 6 tests (23):** standard profile, sprint profile, exo altitude, area denial linger, launch with type, predict_arc type differentiation, MIRV carrier spawning, MIRV split (count, components, velocity spread, ascending check, event), compose_wave MIRV thresholds, shockwave vs MIRV children, chain reaction destroy/deflect zones, interceptor destruction (no chain), multi-step cascade, tech tree unlock/upgrade, effective_profile, campaign snapshot tech tree
**Physics tests (10):** freefall, 45° range, arcs, drag, altitude drag, thrust, post-burn, OOB cleanup, lifetime, snapshots

### Milestone Verification
- `cargo test` — ✅ 111 tests passing
- `cargo clippy` — ✅ 0 warnings
- `npm run build` (tsc + vite) — ✅ passes
- `cargo tauri dev` — ready: Q/W/E/R type selection, colored interceptors, MIRV splits, chain reactions with deflection, tech tree unlocks/upgrades

---

## Phase 7: Save/Load, Weather, Radar/Detection, CRT Shaders, Audio ✅

**Status:** Complete
**Date:** 2026-02-06

### Deliverables
- Save/load system with quicksave (F5/F9), autosave between waves, and slot-based persistence
- Weather and wind system: per-wave weather conditions (Clear/Overcast/Storm/Severe), altitude-dependent lateral wind forces, weather affects wave difficulty
- Radar/detection system: battery-based radar range, weather degrades detection, reentry glow provides visual-only contacts
- CRT post-processing shader: scanlines, chromatic aberration, barrel distortion, vignette, film grain (toggle with C key)
- Web Audio synthesized sound: launch sweep, detonation boom, city damage rumble, wave chimes, MIRV split crack, spatial panning, weather-modulated ambient (toggle with M key)
- 141 automated tests (30 new Phase 7 tests), 0 clippy warnings

### Sub-phase 7A: Save/Load System
- `persistence/save_load.rs` — SaveData/SaveMetadata structs, save_to_file, load_from_file, list_saves, delete_save
- `commands/persistence.rs` — Tauri commands: save_game, load_game, list_saves, delete_save
- Auto-save between waves (slot "autosave"), quicksave/load in Strategic phase (F5/F9)
- Game loop handles SaveGame/LoadGame via EngineCommand variants
- 8 persistence tests

### Sub-phase 7B: Weather + Wind System
- `state/weather.rs` — WeatherCondition enum, WeatherState, generate_weather(), radar_multiplier(), glow_visibility()
- `systems/wind.rs` — Lateral wind force system, altitude-dependent scaling (WIND_ALTITUDE_FACTOR=0.003)
- Waves 1-15 always Clear; wave 16+ introduces Overcast/Storm/Severe with increasing probability
- Storm/Severe multiply missile count (1.15×/1.3×)
- Arc prediction updated to account for wind
- Weather and wind_x included in StateSnapshot
- HUD weather indicator with wind direction/speed
- 12 weather/wind tests

### Sub-phase 7C: Radar/Detection System
- `systems/detection.rs` — Detection system: radar range check from batteries, reentry glow visual detection
- `Detected { by_radar, by_glow }` component added to ECS
- RADAR_BASE_RANGE=500.0, degraded by weather multipliers (Clear=1.0, Overcast=0.85, Storm=0.6, Severe=0.4)
- Glow visibility: Clear=1.0, Overcast=0.3, Storm/Severe=0.0
- Undetected missiles filtered from state snapshot — fog of war mechanic
- Glow-only contacts rendered as dim pulsing orange dots
- HUD shows "CONTACTS: 5 (3R/2G)" breakdown
- System order: ... → Damage → **Detection** → Cleanup → Snapshot
- 10 detection tests

### Sub-phase 7D: CRT Shader Effects
- `src/renderer/shaders/crt.frag.ts` — GLSL fragment shader: scanlines, chromatic aberration, barrel distortion, vignette, film grain, phosphor glow
- `src/renderer/shaders/CRTFilter.ts` — PixiJS v8 Filter with time-animated uniforms
- Applied to stage, toggle with C key
- No Rust changes required

### Sub-phase 7E: Audio System
- `src/audio/SoundSynth.ts` — Oscillator/noise synthesizers: launch sweep (sawtooth 200→800Hz), detonation boom (sine 60Hz + noise burst), city damage rumble (filtered noise), wave chimes (3-note triangle), MIRV split crack (square 1200→200Hz)
- `src/audio/AudioManager.ts` — Web Audio lifecycle, gain chain (effects + ambient → master), spatial StereoPannerNode, weather-modulated ambient hum, mute toggle (M key)
- Sounds triggered from: click-to-launch, detonation events, MIRV split events, wave start/complete phase transitions
- No Rust changes required

### System Execution Order (Final)
Input → WaveSpawner → Thrust → Gravity → Drag → **Wind** → Movement → MirvSplit → Collision → Detonation → Shockwave → Damage → **Detection** → Cleanup → Snapshot

### Test Results (141/141 passing)
**Unit tests (53):** entity, world, components, air density, territory, economy, wave_composer (8), campaign_state, arc_prediction (9), upgrades (7), weather (5), wind (4), detection (10)
**Determinism tests (3):** identical runs, 300-tick stability, divergence detection
**Phase 3 tests (12):** world setup, wave spawning, interceptor launch, empty battery, ground detonation, shockwave destruction, city damage, wave completion, deterministic waves, scripted intercepts, shockwave expansion
**Phase 5 tests (20):** campaign defaults, wave income, expand/place/restock/repair, snapshot structure, world rebuild, full cycle, wave composer scaling, backward compatibility
**Phase 6 tests (23):** interceptor profiles, launch with type, MIRV system, chain reactions, tech tree
**Phase 7 persistence tests (8):** roundtrip, file I/O, listing, delete, simulation-to-save, save-to-simulation
**Physics tests (10):** freefall, 45° range, arcs, drag, altitude drag, thrust, post-burn, OOB cleanup, lifetime, snapshots

### Milestone Verification
- `cargo test` — ✅ 141 tests passing
- `cargo clippy` — ✅ 0 warnings
- `npm run build` (tsc + vite) — ✅ passes
- `cargo tauri dev` — ready: F5/F9 save/load, weather varies wave 16+, wind curves arcs, radar detection with fog of war, CRT shader (C toggle), synthesized audio (M mute)

---

## Phase 8A: Main Menu / Landing Page ✅

**Status:** Complete
**Date:** 2026-02-06

### Deliverables
- Main menu landing page with animated missile/explosion background
- Menu options: New Game, Load Game, Settings
- Game starts in MainMenu phase instead of Strategic — proper entry point
- Load game panel fetches saved games list from backend, displays slot info
- Settings panel with CRT and Audio toggles
- Decorative MenuBackground particle animation (missiles, interceptors, explosions)
- 141 automated tests (no regressions), 0 clippy warnings

### Backend Changes
- `engine/game_loop.rs` — Added `NewGame` EngineCommand variant; `run_loop` starts in `GamePhase::MainMenu` (Simulation default unchanged for test compatibility); `NewGame` handler creates fresh Simulation in Strategic phase
- `commands/campaign.rs` — Added `new_game` Tauri command
- `lib.rs` — Registered `new_game` in generate_handler

### Frontend New Files
- `src/renderer/effects/MenuBackground.ts` — Particle animation on PixiJS Graphics: missiles descend from top with red trails, interceptors launch from bottom targeting missiles, explosions expand and fade. ~1 missile/1.5s, ~1 interceptor/2s spawn rate
- `src/renderer/MainMenuView.ts` — Container with title ("DETERRENCE"), subtitle, 3 menu buttons (NEW GAME, LOAD GAME, SETTINGS), load panel (fetches saves, shows slot name/wave/resources/date), settings panel (CRT ON/OFF, AUDIO ON/OFF), hover effects on buttons, version/keybind hints

### Frontend Modified Files
- `src/renderer/GameRenderer.ts` — Imports MainMenuView + MenuBackground, creates both in init(), wires callbacks (onNewGame→newGame(), onLoadGame→loadGame(), onCRTToggle, onMuteToggle), updated setViewForPhase() for MainMenu/Strategic/WaveActive phases, MenuBackground ticks alongside CRT shader, lastPhase starts as "MainMenu"
- `src/renderer/HUD.ts` — Added visible getter/setter (controls container + waveComplete overlay)
- `src/bridge/commands.ts` — Added newGame() invoke wrapper

### Milestone Verification
- `cargo test` — ✅ 141 tests passing (no regressions — Simulation default unchanged)
- `cargo clippy` — ✅ 0 warnings
- `npm run build` (tsc + vite) — ✅ passes
- `cargo tauri dev` — ready: app opens to animated main menu, NEW GAME transitions to Strategic, LOAD GAME shows saves, SETTINGS toggles CRT/audio

---

## Phase 8B: Next Steps
**Status:** Not started
**Potential:** Difficulty tuning, more visual polish, particle effects, score tracking, game over screen, tutorial

---

## Build Commands
```bash
cargo test --manifest-path src-tauri/Cargo.toml    # 141 tests, all passing
cargo clippy --manifest-path src-tauri/Cargo.toml   # 0 warnings
npm run build                                        # tsc + vite, clean
cargo tauri dev                                      # launch app
```
