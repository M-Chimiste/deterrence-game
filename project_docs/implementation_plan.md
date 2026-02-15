DETERRENCE mk2 — Implementation Plan
Context
The mk2 branch is a clean reimplementation of DETERRENCE, an IAMD simulation game. The previous implementation (on main) used a simpler architecture (single Tauri crate, PixiJS, custom ECS). This time we're building the full architecture from the start: 7-crate Cargo workspace, hecs ECS, Three.js + Preact frontend, targeting a CIC Console as the first playable milestone.

The design docs in project_docs/ define the complete vision. This plan turns that vision into 10 buildable phases.

Decisions
Architecture: Full workspace up front (7 crates)
ECS: hecs
Frontend: Three.js + Preact + Zustand + Howler.js + Vite
First milestone: Full CIC console (Phase 6)
Full Phase 1 MVP: Phase 10
Phase 1: Workspace Foundation + Core Types + Build Pipeline
Goal: Everything compiles, cargo tauri dev launches a window.

Rust workspace (/Cargo.toml):

All 7 crates: deterrence-core, deterrence-sim, deterrence-threat-ai, deterrence-terrain, deterrence-procgen, deterrence-campaign, deterrence-app
Workspace-level deps: serde, serde_json, glam/nalgebra, rand, hecs
deterrence-core — the vocabulary crate (no Tauri dep):

types.rs — Position, Velocity, Bearing, Range, Altitude, SimTime
components.rs — All hecs components: RadarCrossSection, TrackInfo, ThreatProfile, MissileState, RadarSystem, LauncherState, Illuminator
commands.rs — PlayerCommand enum (HookTrack, VetoEngagement, SetRadarSector, SetDoctrine, SetTimeScale, etc.)
state.rs — GameStateSnapshot + all *View sub-structs (TrackView, EngagementView, SystemsView, etc.)
events.rs — AudioEvent enum, Alert struct
enums.rs — Classification, DoctrineMode, RadarMode, EngagementPhase, WeaponType, etc.
constants.rs — TICK_RATE, radar equation constants, detection thresholds
Other crates: Stubs (lib.rs with doc comment + re-export of core)

deterrence-app: Tauri entry point, tauri.conf.json, icons, capabilities

Frontend (/frontend/):

package.json — Three.js, Preact, Zustand, Howler.js, @tauri-apps/api, Vite
vite.config.ts — Preact plugin, GLSL raw import, Tauri env prefix
src/ipc/ — TypeScript mirrors of PlayerCommand, GameStateSnapshot
src/store/gameState.ts — Zustand store skeleton
Tests: serde round-trip for all enums, cargo tauri dev opens window with "DETERRENCE" text

Phase 2: ECS World + Simulation Engine + Tick Loop
Goal: Headless simulation runs at 30Hz, spawns entities, produces serialized snapshots.

deterrence-sim:

engine.rs — SimulationEngine: owns hecs::World, SimTime, command queue. tick(commands) -> GameStateSnapshot
world_setup.rs — Entity spawn factories (own ship, batteries, threats)
systems/movement.rs — Kinematic integration: position += velocity * dt
systems/cleanup.rs — Remove OOB/dead entities
systems/snapshot.rs — Query ECS, build GameStateSnapshot
Tests: Determinism (same seed = same output), entity lifecycle, snapshot serialization (<100KB for 100 entities), tick timing (30 ticks = 1 sec)

Phase 3: Tauri IPC Bridge + Game Loop Thread + Frontend Connection
Goal: Frontend receives live snapshots. Pause/unpause works.

deterrence-app:

game_loop.rs — Background thread: runs sim at 30Hz, emits game:state_snapshot via AppHandle, processes commands via mpsc
ipc.rs — Tauri commands: start_simulation, send_command, get_snapshot
state.rs — AppState with sync primitives
Frontend:

src/ipc/bridge.ts — startSimulation(), sendCommand(), onSnapshot() wrappers
src/store/interpolation.ts — Lerp between consecutive snapshots
src/debug/DebugOverlay.tsx — Tick counter, entity count, FPS, IPC latency
Tests: IPC round-trip, snapshot rate ~30Hz, pause/resume, serialization <3ms

Looks like: Black window showing debug text — tick counter, entity count, FPS. Pause/resume/time-scale buttons work.

Phase 4: Radar Detection Model + PPI Tactical Display
Goal: Contacts appear on a PPI radar display with NTDS symbology. Radar energy budget is visible.

deterrence-sim/src/radar/:

detection.rs — RadarDetectionSystem: Pd from simplified radar equation (range, RCS, energy, environment)
tracking.rs — Track lifecycle: initiate → promote → drop. Quality degradation over time.
energy.rs — RadarEnergySystem: total budget split between search volume and tracked contacts
Frontend (src/tactical/):

PPI.ts — Three.js OrthographicCamera: dark green circle, range rings, bearing lines, rotating sweep line, phosphor glow shader
symbology.ts — NTDS symbols: Unknown (circle), Hostile (diamond), Friendly (semicircle), Neutral (square), Suspect (quatrefoil)
tracks.ts — Symbol + velocity leader + history trail + track number label + hook highlight
Frontend panels:

TrackBlock.tsx — Hooked track data: bearing, range, alt, speed, heading, classification, quality
RadarStatus.tsx — Energy budget bar (search vs track), sector indicator
Tests: Detection Pd vs range/RCS, fourth-root law, track lifecycle, energy budget vs track count, sector narrowing

Looks like: A CIC radar scope — green phosphor PPI with rotating sweep, contacts as NTDS symbols with velocity leaders. Click to hook, see data. Radar energy bar shifts as tracks accumulate.

Phase 5: Fire Control + Veto Clock + Engagement State Machine
Goal: Core DCIE loop works. Veto Clock ticks down. Player vetoes or confirms.

deterrence-sim/src/fire_control/:

dte.rs — DTE state machine: SolutionCalc → Ready (Veto Clock starts) → Launched/Cancelled
solution.rs — PIP calculation (where/when intercept occurs), quality from track quality
pk.rs — Probability of kill: Pk = f(weapon, range, aspect, RCS, ECM, track_quality)
deterrence-sim/src/systems/:

fire_control.rs — FireControlSystem: evaluate hostiles, manage engagement FSMs, advance Veto Clocks
engagement.rs — EngagementSystem: on Veto expiry → launch interceptor entity
Frontend panels:

VetoClock.tsx — Circular countdown timer per engagement. Green → amber → red. [VETO] [CONFIRM] buttons. Ticking audio accelerates near expiry. Multiple clocks stack vertically.
ThreatTable.tsx — All hostiles/suspects: bearing, range, speed, engagement status, Veto remaining, weapon, Pk. Sortable.
Keybinds: V=veto, C=confirm, Tab=cycle Veto Clocks

Tests: DTE progression, veto cancels, confirm skips timer, PIP geometry, Pk boundaries, doctrine modes, multiple simultaneous engagements

Looks like: Contacts classified hostile trigger Veto Clocks counting down on the right. Threat table shows engagement schedule. Player vetoes or confirms. On expiry, missile launches and appears on PPI.

Phase 6: Illuminators + VLS Panel + CIC Console Assembly ★ FIRST PLAYABLE
Goal: Illuminator bottleneck creates resource tension. Full CIC console assembled.

deterrence-sim/src/fire_control/illuminator.rs:

IlluminatorScheduler: 3-4 channels, terminal engagements require one, time-sharing when saturated, queuing when full
deterrence-sim/src/systems/:

missile_kinematics.rs — Interceptor flight: boost → midcourse (command guided) → terminal (semi-active, needs illuminator)
intercept.rs — Evaluate intercept: lethal radius check, Pk roll, BDA delay
Frontend panels:

VLSStatus.tsx — Cell grid (64 Mk 41 cells), color-coded: green=ready, amber=assigned, red=expended. Ready count by type.
IlluminatorStatus.tsx — 3-4 channel bars: assigned engagement, time remaining, queue depth indicator
Frontend HUD (src/hud/):

WindowManager.ts — Panel layout: PPI (center-left), Veto Clocks (right), threat table (bottom), VLS (top-right), illuminators (mid-right), radar status (top-left), track block (bottom-left)
Theme.ts — CIC dark theme, JetBrains Mono font, green-on-dark colors
Tests: Illuminator saturation (5 engagements, 3 channels), time-sharing, freed on completion, full end-to-end DCIE test

Looks like: The full CIC console. PPI with contacts and NTDS symbols. Veto Clocks ticking. VLS showing depleting magazine. Illuminator panel cycling assignments. Threat table with full engagement schedule. When 5+ threats go terminal, illuminator saturation forces hard choices about priority.

Phase 7: Threat AI + Multi-Wave Scenarios + Proper Kinematics
Goal: Threats behave like real ASCMs. Scenarios escalate across waves.

deterrence-threat-ai:

state_machine.rs — Threat FSM: Cruise → PopUp → Terminal → Evasive/Destroyed
coordinator.rs — Wave composition, multi-axis attacks, time-on-top coordination
archetypes.rs — Data-driven threat definitions (YAML)
deterrence-sim/src/kinematics/:

missile.rs — Interceptor: boost → midcourse → terminal with proportional navigation
threat.rs — Threat flight: profile-based speed/altitude curves, sea-skimming, terminal dive
geometry.rs — Lead/pure pursuit, time-to-intercept, engagement envelopes
Data: /data/threats/ — sea_skimmer_mk1.yaml, sea_skimmer_mk2.yaml, supersonic_cruiser.yaml

Scenario system: scenario.rs — Wave sequences, 3 hardcoded scenarios (easy/medium/hard)

Phase 8: 3D World View + Intercept Visualization
Goal: PIP 3D view showing ocean, missile trails, intercept effects.

Frontend (src/world/):

Scene.ts, Ocean.ts (animated shader), Sky.ts (gradient skybox)
Camera.ts — Overhead / follow / cinematic modes
MissileTrails.ts — Particle trails (blue=friendly, red=hostile)
Intercepts.ts — Hit flash + debris, miss divergence, impact effect
Ships.ts, EntityMarkers.ts — Simple 3D markers
Integration: PIP window on CIC, toggleable (W key), follow mode (F key)

Phase 9: Environment + IFF/Civilians + Audio
Goal: Information ambiguity. Civilian traffic creates identification dilemmas. Audio builds tension.

deterrence-sim:

radar/environment.rs — Sea clutter, atmospheric ducting, weather degradation
identification/iff.rs — IFF interrogation (modes, reliability, false codes)
identification/profiling.rs — Kinematic profiling: match flight profile against archetypes
systems/civilian.rs — Civilian traffic on airways, correct IFF (usually), edge cases
Frontend audio (src/audio/):

AudioManager.ts — Howler.js event routing, spatial panning
ambient.ts — CIC hum layers, tension escalation crossfades
alerts.ts — Contact beeps, hostile tones, Veto warning, missile launch, intercept, vampire alarm
tension.ts — Dynamic layer system driven by threat count + engagement status
Phase 10: Scoring + Campaign + Menus + Polish ★ PHASE 1 MVP
Goal: Complete MVP loop: menu → briefing → mission → score → progress.

deterrence-sim/src/systems/scoring.rs — Threats killed, assets protected, interceptors used, civilian casualties, grade (S-F)

deterrence-campaign:

campaign.rs — Mission sequence, persistent interceptor inventory
persistence.rs — Save/load via redb
scoring.rs — Cumulative campaign score
deterrence-procgen (basic):

generator.rs — Difficulty-scaled wave generation, solvability validation
Frontend: MainMenu, MissionSelect, ScorePanel, SettingsPanel, mission briefing text

Phase Summary
Phase	Deliverable	Milestone
1	Workspace compiles, Tauri window opens	Foundation
2	Headless sim runs 30Hz, produces snapshots	Engine
3	Frontend receives live snapshots, pause/resume	Bridge
4	PPI radar display with NTDS symbols, energy budget	Radar
5	Veto Clock, DTE state machine, engagements	Core Mechanic
6	Illuminators, VLS, full CIC console	★ CIC Console
7	Threat AI, multi-wave scenarios, kinematics	Threats
8	3D ocean view, missile trails, intercepts	3D View
9	Environment, IFF, civilians, audio	Ambiguity
10	Scoring, campaign, menus, polish	★ Phase 1 MVP
Verification
After each phase:

cargo clippy --workspace -- -D warnings passes
cargo test --workspace passes (new tests per phase)
cargo fmt --all --check passes
npm run build (in frontend/) succeeds
cargo tauri dev launches and demonstrates the phase's deliverable
Update CLAUDE.md with new build notes and architecture changes