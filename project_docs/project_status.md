# DETERRENCE mk2 — Project Status

## Current State

**Branch:** `mk2`
**Phase:** 6 of 10 complete
**Tests:** 78 passing (14 core + 46 sim + 5 app + 13 threat-ai), 0 clippy warnings
**Date:** 2026-02-15

---

## Session Log

### Session 1 — 2026-02-14: Phase 1 Complete (Workspace Foundation)

**What was done:**
- Created 10-phase implementation plan (see `~/.claude/plans/giggly-fluttering-teacup.md`)
- Set up 7-crate Cargo workspace: `deterrence-core`, `deterrence-sim`, `deterrence-threat-ai`, `deterrence-terrain`, `deterrence-procgen`, `deterrence-campaign`, `deterrence-app`
- Implemented `deterrence-core` vocabulary crate with all foundational types:
  - `types.rs` — Position (x/y/z meters), Velocity, SimTime with geometry helpers
  - `enums.rs` — Classification, DoctrineMode, RadarMode, WeaponType, EngagementPhase, ThreatArchetype, ThreatPhase, MissilePhase, IlluminatorStatus, CellStatus, GamePhase, etc.
  - `components.rs` — hecs ECS components: RadarCrossSection, TrackInfo, ThreatProfile, MissileState, RadarSystem, LauncherSystem, Illuminator, OwnShip, Threat, Interceptor, Civilian, PositionHistory
  - `commands.rs` — PlayerCommand enum (HookTrack, VetoEngagement, SetRadarSector, SetDoctrine, SetTimeScale, etc.)
  - `state.rs` — GameStateSnapshot + TrackView, EngagementView, RadarView, VlsView, IlluminatorView, ScoreView
  - `events.rs` — AudioEvent enum, Alert struct
  - `constants.rs` — TICK_RATE=30, radar parameters, tracking thresholds, weapon performance, threat defaults
  - `tests.rs` — 14 tests covering serde round-trips, geometry, time advancement
- Set up `deterrence-app` Tauri 2.x entry point with tauri.conf.json, capabilities, placeholder icons
- Set up frontend (`frontend/`) with Vite + Preact + Three.js + Zustand + Howler.js
  - TypeScript IPC type mirrors (`frontend/src/ipc/state.ts`, `commands.ts`, `bridge.ts`)
  - Zustand store with snapshot interpolation support
  - Landing page with green phosphor CIC aesthetic
- Created CLAUDE.md with build commands and architecture reference

**Key decisions:**
- Chose glam 0.29 over nalgebra (lighter, more game-oriented)
- Chose Preact over React (3KB vs 40KB, API-compatible, sufficient for bespoke CIC panels)
- Coordinate system: x=East, y=North, z=Up (meters), bearing 0=North clockwise
- Tagged JSON for PlayerCommand (`#[serde(tag = "type")]`)

**Verification:**
- `cargo test --workspace` — 14 passing
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo fmt --all --check` — clean
- `npx tsc --noEmit` — clean
- `npx vite build` — 26KB bundle

**Next:** Phase 2 — ECS World + Simulation Engine + Tick Loop (SimulationEngine struct, movement system, snapshot system, determinism tests)

### Session 2 — 2026-02-14: Phase 2 Complete (ECS World + Simulation Engine)

**What was done:**
- Implemented `deterrence-sim` simulation engine with full ECS world:
  - `engine.rs` — `SimulationEngine` struct: owns hecs `World`, `SimTime`, command queue (`VecDeque`), seeded `ChaCha8Rng`. `tick()` → processes commands → runs systems → returns `GameStateSnapshot`
  - `world_setup.rs` — Entity spawn factories: own ship (radar + VLS 64 cells), 3 illuminators, threat waves with archetype-based parameters (speed, altitude, RCS)
  - `systems/movement.rs` — Kinematic integration (`pos += vel * dt`) + position history recording every 15 ticks
  - `systems/cleanup.rs` — Despawns OOB entities (beyond 185km) and destroyed/impacted threats using pre-allocated buffer
  - `systems/snapshot.rs` — Queries entire ECS world to build complete `GameStateSnapshot` (tracks, own ship, radar, VLS, illuminators)
  - `tests.rs` — 11 tests: determinism (same/different seeds), entity lifecycle, snapshot size (<100KB for 100 entities), tick timing, pause/resume, movement integration, hook/unhook, snapshot completeness, phase gating, time scale

**Key decisions:**
- `SimulationEngine` is completely headless (no Tauri dependency) — enables deterministic testing
- Commands batch-processed at tick start (before systems), not mid-tick
- `time_scale` stored but not used to modulate tick rate — Phase 3's game loop controls call frequency
- Threats start pre-classified Hostile with quality 1.0 (radar detection is Phase 4)
- Illuminators are separate entities (not arrays on OwnShip) for clean ECS queries

**Verification:**
- `cargo test --workspace` — 25 passing (14 core + 11 sim)
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo fmt --all --check` — clean
- `npx vite build` — clean

**Next:** Phase 3 — Tauri IPC Bridge + Game Loop Thread + Frontend Connection

### Session 3 — 2026-02-14: Phase 3 Complete (Tauri IPC Bridge + Game Loop Thread)

**What was done:**
- Implemented `deterrence-app` game loop thread and IPC bridge:
  - `state.rs` — `AppState` (Tauri managed state with `Mutex`-wrapped channels), `GameLoopCommand` enum for thread communication
  - `game_loop.rs` — `spawn_game_loop()` creates named thread owning `SimulationEngine`, runs at 30Hz with time-scale-adjusted sleep, drains commands via `mpsc`, emits snapshots via `AppHandle.emit()`, caches latest snapshot in `Arc<Mutex<...>>`
  - `ipc.rs` — Three `#[tauri::command]` handlers: `start_simulation` (spawns game loop), `send_command` (forwards via mpsc), `get_snapshot` (reads cached snapshot)
  - `main.rs` — Wired `AppState::new()` into `.manage()`, registered IPC handlers via `generate_handler!`
- Updated frontend IPC and store:
  - `bridge.ts` — Added `getSnapshot()`, convenience wrappers (`pauseSimulation`, `resumeSimulation`, `setTimeScale`, `startMission`)
  - `gameState.ts` — Added snapshot rate tracking (rolling 1-second window), `snapshotReceivedAt` timestamp
  - `interpolation.ts` — Snapshot interpolation utilities: `getInterpolationFactor()`, `lerpPosition()`, `getInterpolatedTracks()` (scaffolding for Phase 4 PPI rendering)
- Created debug overlay:
  - `debug/DebugOverlay.tsx` — Shows PHASE, TICK, TRACKS, SNAP RATE (Hz), RENDER FPS. Controls: START MISSION, PAUSE/RESUME, time scale buttons (0.5x/1x/2x/4x). CIC green-on-black aesthetic.
- Updated `App.tsx` — `useEffect` initializes IPC connection (subscribes to snapshots, starts simulation), mounts DebugOverlay

**Key decisions:**
- Engine created inside game loop thread (clean ownership, avoid Send concerns)
- Time scale modulates sleep duration (1 tick per loop, smoother than multi-tick bursts)
- `GameLoopCommand` wrapper enum keeps `Shutdown` variant separate from `PlayerCommand`
- `AppState` uses `Arc<Mutex<Option<GameStateSnapshot>>>` for snapshot sharing (no extra deps)
- Fire-and-forget `app_handle.emit()` — dropped snapshots are harmless

**Verification:**
- `cargo test --workspace` — 30 passing (14 core + 11 sim + 5 app)
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo fmt --all --check` — clean
- `npx tsc --noEmit` — clean
- `npx vite build` — 30KB bundle

**Next:** Phase 4 — Radar Detection Model + PPI Tactical Display

### Session 4 — 2026-02-14: Phase 4 Complete (Radar Detection Model + PPI Tactical Display)

**What was done:**
- Implemented realistic radar detection pipeline (Rust):
  - `components.rs` — Added `DetectionCounter` component (pre-track hit/miss accumulator)
  - `constants.rs` — Added `RADAR_BEAM_HALF_WIDTH_TICKS`, `RADAR_K`, `RADAR_MIN_RANGE`, `TRACK_INITIAL_QUALITY`
  - `systems/radar/energy.rs` — Energy budget system: allocates search/track energy based on active track count, advances sweep angle
  - `systems/radar/detection.rs` — Sweep-based probabilistic detection: `SNR = K * (search_energy/total) * (rcs/range^4)`, `Pd = 1 - exp(-SNR)`. Beam width ~4.5°, sector filtering, deterministic RNG rolls. 5 inline unit tests.
  - `systems/radar/tracking.rs` — Track lifecycle: promote after 3 consecutive hits (→ TrackInfo), drop after 5 misses or quality ≤ 0. Generates AudioEvent::NewContact/ContactLost.
  - `engine.rs` — Wired radar systems (energy → detection → tracking) before movement/cleanup. Added audio event buffer. Implemented SetRadarSector, SetRadarMode, ClassifyTrack, SetDoctrine commands.
  - `world_setup.rs` — Threats now spawn with `DetectionCounter::default()` (undetected). Added `spawn_tracked_threat` test helper.
  - `snapshot.rs` — Wired `active_track_count` from TrackInfo query, plumbed audio events through snapshot.
  - `tests.rs` — Updated 6 existing tests for new detection model, added 6 new tests: undetected threats, track initiation pipeline, energy budget, sector narrowing, classify track, set radar mode.
- Built PPI tactical display (frontend):
  - `tactical/symbology.ts` — NTDS symbol geometries (circle/diamond/semicircle/square/quatrefoil) + classification colors
  - `tactical/tracks.ts` — `TrackRenderer` class: object pooling, NTDS symbols, velocity leaders, history trails, hook highlights, track labels
  - `tactical/PPI.tsx` — Three.js PPI radar scope: orthographic camera, range rings (4 at 25nm), bearing ticks, cardinal labels, rotating sweep line with phosphor decay trail, click-to-hook interaction
  - `panels/TrackBlock.tsx` — Hooked track detail readout (BRG/RNG/ALT/SPD/HDG/CLASS/IFF/QUAL)
  - `panels/RadarStatus.tsx` — Radar status panel with energy bar (search green, track amber), mode, sector, track count
  - `styles.css` — CIC theme CSS (green-on-black, panel overlays, energy bar)
  - `App.tsx` — Conditional layout: PPI scope with overlaid panels when Active/Paused, splash screen for MainMenu
  - `bridge.ts` — Added `hookTrack`, `unhookTrack`, `classifyTrack`, `setRadarSector`, `setRadarMode` wrappers

**Key decisions:**
- Detection model uses fourth-root law (SNR ∝ rcs/range⁴) — doubling RCS extends detection range by factor 2^(1/4) ≈ 1.19x
- Sweep-based detection: only entities within beam width (~4.5°) checked per tick, preventing continuous-time cheating
- `last_sweep_tick` on DetectionCounter prevents double-counting in a single sweep pass
- Dropped tracks get DetectionCounter re-added (entity still flies, can be re-detected)
- PPI uses direct coordinate mapping (world meters → screen), not bearing/range projection
- Click-to-hook uses world-coordinate distance threshold, not raycasting

**Verification:**
- `cargo test --workspace` — 41 passing (14 core + 22 sim + 5 app)
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo fmt --all` — clean
- `npx tsc --noEmit` — clean
- `npx vite build` — 517KB bundle (Three.js)

**Next:** Phase 5 — Threat AI FSM + Threat Wave Spawning

### Session 5 — 2026-02-14: Phase 5 Complete (Threat AI FSM + Fire Control + Engagement Pipeline)

**What was done:**
- Implemented threat AI FSM in `deterrence-threat-ai` crate:
  - `profiles.rs` — Archetype behavior profiles (cruise_speed, terminal_range, popup_range, can_evade, terminal_dive flags)
  - `fsm.rs` — Core FSM: `evaluate()` dispatches to phase-specific handlers (cruise, popup, terminal, evasive). Velocity adjustments on transition (popup climb, terminal acceleration ×1.2, Mk2 lateral jink, ballistic dive)
  - `tests.rs` — 13 unit tests covering all archetype transitions (SeaSkimmerMk1/Mk2, SupersonicCruiser, SubsonicDrone, TacticalBallistic, destroyed/impact terminal states)
- Implemented wave spawning system:
  - `systems/wave_spawner.rs` — `WaveSchedule` + `WaveEntry` structs, 3-wave default mission (tick 0: 3× Mk1, tick 300: 2× Mk1 + 1× Supersonic, tick 600: 2× Mk2 + 1× Supersonic + 1× Drone)
  - `world_setup.rs` — `setup_mission()` now only spawns own ship + illuminators; threats delegated to wave scheduler
- Implemented threat AI ECS system wrapper:
  - `systems/threat_ai.rs` — Queries ECS, calls FSM `evaluate()`, applies updates in second pass (hecs borrow pattern), emits `VampireImpact` audio on impact
- Implemented engagement data model:
  - `engagement.rs` — `Engagement` struct (target entity, phase, weapon type, Pk, VLS cell, veto timer, PIP, interceptor entity, result) + `ScoreState` (kills, total, fired, impacted)
- Implemented fire control system:
  - `systems/fire_control.rs` — 4-step per-tick: update threat engagement status, cleanup old Complete/Aborted engagements, create new engagements for eligible hostiles (doctrine check, quality >= 0.6, weapon/cell selection), advance engagement state machine (SolutionCalc → Ready → Launched)
  - Weapon selection by range (>100km ER, 20-100km Standard, <20km PD) with fallback
  - PIP calculation (pursuit geometry: `tti = range / closing_speed`)
  - Pk model (`base × range_factor × quality / rcs_factor`, clamped [0.1, 0.95])
  - Interceptor spawning: expend VLS cell, spawn entity with TrackInfo(Friend), emit BirdAway
- Implemented intercept system:
  - `systems/intercept.rs` — Proximity check (20m lethal radius), Pk roll, fuel exhaustion. Marks threat Destroyed on hit, emits Splash events
- Updated engine, snapshot, cleanup:
  - `engine.rs` — New system order (wave_spawner → threat_ai → radar → fire_control → intercept → movement → cleanup). VetoEngagement/ConfirmEngagement commands. Engagement/score state on engine.
  - `snapshot.rs` — Builds `EngagementView` list + populates `ScoreView` from engine state
  - `cleanup.rs` — Handles interceptor OOB/complete despawn
- Frontend updates:
  - `bridge.ts` — Added `vetoEngagement()`, `confirmEngagement()`, `setDoctrine()` IPC wrappers
  - `gameState.ts` — Added `focusedEngagementId`, `setFocusedEngagement()`, `cycleFocusedEngagement()`. Auto-clears stale focused engagement.
  - `panels/VetoClock.tsx` — Engagement cards: SolutionCalc ("COMPUTING..."), Ready (countdown timer green→amber→red + progress bar + VETO/CONFIRM buttons), Launched ("BIRD AWAY — TTI"), Complete (KILL/MISS result)
  - `panels/ThreatTable.tsx` — Threat board table: TRK/BRG/RNG/SPD/CLS/ENG/WPN/Pk columns, sorted by range, hooked row highlight, score footer
  - `App.tsx` — Added VetoClock (right side), ThreatTable (bottom center), Tab/V/C keyboard shortcuts
  - `styles.css` — New overlay positions, veto card styles, threat table styles
- Added 16 new sim tests covering: wave spawning schedule, score totals, threat terminal transition, threat impact, engagement creation (AutoSpecial), no engagement (Manual), no duplicates, veto clock countdown, veto abort, confirm launch, interceptor spawning, Friend classification, VLS cell consumption, audio events, doctrine command, engagement snapshot

**Key decisions:**
- Engagements stored as `HashMap<u32, Engagement>` on engine (not ECS entities) — cross-cutting state linking target, weapon, interceptor
- Wave schedule on engine, first wave at tick 0 for backward compatibility
- Interceptors get `TrackInfo(Classification::Friend)` to reuse existing track rendering pipeline
- Focused engagement is frontend-only state (Zustand store), not sent to backend
- Illuminator scheduling deferred to Phase 6

**Verification:**
- `cargo test --workspace` — 70 passing (14 core + 38 sim + 5 app + 13 threat-ai)
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo fmt --all --check` — clean
- `npx tsc --noEmit` — clean
- `npx vite build` — 523KB bundle

**Next:** Phase 6 — CIC Console Milestone (First Playable)

### Session 6 — 2026-02-15: Phase 6 Complete (CIC Console Milestone — First Playable)

**What was done:**
- Implemented missile kinematics system:
  - `systems/missile_kinematics.rs` — Interceptor flight phase transitions: Boost (5s fixed velocity) → Midcourse (retargets velocity toward current target position each tick) → Terminal (gated on illuminator assignment or ER active seeker). Uses collect-then-mutate pattern for hecs borrow safety.
  - `constants.rs` — Added `MISSILE_BOOST_DURATION_SECS` (5.0s) and `TERMINAL_GUIDANCE_RANGE` (20km)
  - `components.rs` — Added `phase_start_tick: u64` to MissileState for timing phase transitions
- Implemented illuminator scheduler system (the core resource tension mechanic):
  - `systems/illuminator.rs` — 4-step per-tick: release completed/orphaned channels, identify candidates (midcourse engagements within 1.5× terminal range, Standard/PD only), assign idle channels (lowest TTI priority), update time-sharing (cycle illuminators when queue has waiters, Pk penalty proportional to share count)
  - `components.rs` — Added `dwell_remaining_secs: f64` to Illuminator for time-sharing rotation
  - `engagement.rs` — Added `illuminator_channel: Option<u8>` to Engagement struct
  - `engine.rs` — Added `illuminator_queue: Vec<u32>` for cross-cutting illuminator waiting list
- Updated fire control phase sync:
  - `systems/fire_control.rs` — Launched arm now matches `Launched | Midcourse | Terminal`, reads interceptor's MissileState and syncs engagement phase to match
- Updated intercept system with terminal phase gate:
  - `systems/intercept.rs` — Gates proximity + Pk check on `MissilePhase::Terminal` (or Midcourse for ER missiles). Time-sharing Pk penalty: `effective_pk = base_pk / total_needing_illumination` when queue is non-empty.
- Updated engine system execution order:
  - `engine.rs` — fire_control → illuminator (NEW) → missile_kinematics (NEW) → intercept. Wired illuminator_queue through snapshot.
- Updated snapshot system:
  - `systems/snapshot.rs` — Wired `illuminator_channel` from engagement, `queue_depth` from illuminator queue length
- Added 8 new Rust tests:
  - `test_missile_boost_to_midcourse_transition` — Verifies phase transition after 5s boost
  - `test_missile_velocity_retargeting_in_midcourse` — Verifies velocity tracks toward target
  - `test_illuminator_assigned_for_terminal` — Verifies illuminator activation during terminal phase
  - `test_illuminator_freed_on_completion` — Verifies channels return to Idle after engagement
  - `test_illuminator_saturation_5_engagements_3_channels` — Verifies at most 3 active channels
  - `test_intercept_requires_terminal_phase` — Verifies no intercept during Boost/early Midcourse
  - `test_er_missile_no_illuminator_needed` — Verifies ER missile reaches Terminal without illuminator
  - `test_full_dcie_end_to_end` — Complete detect → engage → launch → terminal → intercept cycle
  - `world_setup.rs` — Added `spawn_tracked_threat_at()` test helper for precise range/bearing positioning
- Built VLS Magazine panel (frontend):
  - `panels/VLSStatus.tsx` — 8×8 CSS grid of color-coded cells (green=Standard, blue=ER, cyan=PD, amber=Assigned, red=Expended). Summary line: SM/ER/PD/RDY counts.
- Built Illuminator Status panel (frontend):
  - `panels/IlluminatorStatus.tsx` — 3 channel rows with status dots (dim green=Idle, bright green=Active, amber=TimeSharing), engagement assignment info, queue depth indicator.
- Updated VetoClock engagement cards:
  - `panels/VetoClock.tsx` — Midcourse ("MIDCOURSE — TTI Xs"), Terminal ("TERMINAL — CH N — TTI Xs" or "AWAITING ILLUM")
- Assembled CIC console layout:
  - `App.tsx` — Right-column flex container stacking DebugOverlay + VLSStatus + VetoClock + IlluminatorStatus
  - `styles.css` — VLS grid, illuminator channel, terminal status styles, right-column layout

**Key decisions:**
- Illuminator system is separate from fire_control (clean separation, one-system-per-file pattern)
- ER missiles skip illuminator (SM-6 equivalent with active seeker) — meaningful tradeoff: fewer ER cells but no illuminator dependency
- Standard/PD require illuminator — the core bottleneck: 3 channels shared across all terminal-phase engagements
- Time-sharing reduces Pk proportionally to share count (saturation has real gameplay cost)
- MissileState.phase is source of truth; fire_control reads it and syncs EngagementPhase
- Orphaned illuminator detection handles cleanup timing gap (fire_control removes engagements before illuminator runs)

**Verification:**
- `cargo test --workspace` — 78 passing (14 core + 46 sim + 5 app + 13 threat-ai)
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo fmt --all --check` — clean
- `npx tsc --noEmit` — clean
- `npx vite build` — 525KB bundle

**Next:** Phase 7 — Audio & Polish
