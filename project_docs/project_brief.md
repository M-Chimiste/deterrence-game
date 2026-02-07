# Deterrence — Product Requirements Document

**Working Title:** Deterrence
**Platform:** Windows, macOS, Linux (via Tauri)
**Tech Stack:** Tauri (Rust backend) + PixiJS (WebGL rendering)
**Genre:** Strategic Missile Defense / Physics-Based Tactics
**Aesthetic:** Cold War Retro-Strategic
**Version:** 0.1 — Initial Design
**Author:** Christian & Claude
**Date:** February 2026

---

## Vision Statement

Deterrence reimagines the classic Missile Command formula as a physics-driven strategic defense game set against a Cold War backdrop. Players command a missile defense network across an expanding national territory, managing radar coverage, interceptor batteries, and civilian infrastructure against escalating ballistic threats. Every missile — incoming and outgoing — follows real ballistic physics. Victory comes not from reflexes alone but from understanding trajectories, managing resources, and making hard triage decisions about what to save and what to sacrifice.

The soul of the game: *you are always losing slowly, and strategy determines how slowly.*

---

## Design Pillars

### 1. Physics Are Real, Always
All missile behavior — enemy and friendly — is governed by a consistent physics simulation from Wave 1 onward. Gravity, drag, and blast propagation are never simplified or toggled. Difficulty scales through scenario complexity (more missiles, faster missiles, MIRVs) rather than system simplification. Players develop genuine physical intuition that rewards them throughout the entire campaign.

### 2. Every Interception Is a Decision
Where and when you intercept matters as much as whether you intercept. High-altitude intercepts are easier (slower targets at apogee) but require radar investment. Low-altitude intercepts are desperate and dangerous — terminal velocity targets, shockwaves risking collateral damage. Players must read trajectories and choose their moment.

### 3. Strategic Depth Between Waves
The tactical wave defense sits inside a larger strategic campaign. Between waves, players allocate resources, expand territory, place batteries, upgrade radar, and make hard choices about growth versus hardening. The campaign map is the long game; each wave is a crisis within it.

### 4. Cold War Anxiety
The aesthetic evokes NORAD command bunkers, civil defense posters, and early-warning radar screens. Vector-line displays, CRT phosphor glow, teletype readouts, and civil defense sirens. The tone is tense and procedural — you're an operator in a bunker making life-and-death calls with incomplete information.

---

## Core Gameplay

### Wave Defense (Tactical Layer)

#### Player Interaction Model
- **Click to target:** Player clicks a point in the sky to designate an intercept point
- **Predicted arc overlay:** The game immediately shows the interceptor's predicted ballistic arc from the nearest (or selected) battery to the target point, including estimated time-to-intercept
- **Commit or adjust:** Player can drag to adjust the target point (arc updates in real-time) or click to confirm launch
- **Battery selection:** If multiple batteries can reach the target, the game highlights the optimal battery but the player can cycle between options using Tab or number keys to choose a different one (different arc, different angle, different time-to-intercept)

This model preserves the original Missile Command's "click the sky" simplicity while surfacing the physics so players make informed decisions. The predicted arc is the primary teaching tool — players learn trajectory mechanics by watching their own interceptors.

#### Physics Systems

**Gravity & Ballistic Trajectories**
- All missiles (enemy and interceptor) follow parabolic/elliptical trajectories under gravity
- Enemy missiles are launched from off-screen origins at varying angles and velocities, producing different arc profiles — high lobs, shallow fast attacks, steep near-vertical drops
- Interceptor trajectories are determined by battery position, launch angle (derived from the click target), and interceptor thrust characteristics
- Missiles are fastest at the lowest point of their trajectory (terminal phase) and slowest at apogee
- Intercept difficulty is directly tied to the phase of the incoming trajectory — apogee intercepts are the easiest but require the longest radar warning

**Atmosphere & Drag**
- The atmosphere is modeled as increasing density at lower altitudes
- Missiles experience drag that increases as they descend, affecting their speed curve (they don't accelerate linearly — they hit a terminal velocity)
- Drag also affects interceptors, giving different interceptor types different effective ceilings and range envelopes
- Re-entry heating creates a visible glow on incoming warheads as they enter dense atmosphere — this is a **visual detection method independent of radar**
  - In clear weather, attentive players can spot re-entry glow before radar detects the missile (rewarding vigilance)
  - In poor weather (clouds, storms), re-entry glow is obscured or invisible, making radar the only detection method
  - This creates a dynamic interplay: clear weather + weak radar is survivable through skill; bad weather + weak radar is genuinely dangerous

**Blast Radius & Shockwaves**
- Interceptor detonations produce a shockwave that expands outward with diminishing force
- **Altitude-dependent blast behavior:**
  - High-altitude detonations: wider blast radius but lower peak force (thin atmosphere, shockwave dissipates quickly). Good for area denial against spreads of missiles.
  - Low-altitude detonations: tight, powerful blast radius. Effective against single targets but risks collateral damage to ground infrastructure and can damage your own structures if too close.
- Near-misses can **deflect** warheads rather than destroy them, altering their trajectory. A deflected warhead is still dangerous — it just lands somewhere unplanned. This can be beneficial (pushed into empty terrain) or catastrophic (pushed toward a city).
- Shockwaves interact with other nearby missiles, enabling chain reactions (see below).

**Chain Reactions**
- When a detonation's shockwave reaches other nearby missiles, it can destroy, deflect, or destabilize them depending on proximity and force
- Skilled players can identify clustered incoming salvos and time a single interception to trigger chain destruction of multiple warheads
- Chain reactions are emergent — the game doesn't script them. They arise naturally from the physics when missiles happen to be close together
- Risk factor: chain reactions can deflect warheads in unpredictable directions, so a "successful" chain kill might push a surviving warhead toward a city you weren't expecting to defend

**MIRVs (Late-Game Mechanic)**
- Introduced in mid-to-late campaign as an escalation event
- A single re-entry vehicle follows a standard ballistic arc until it reaches a split altitude, then deploys multiple independent warheads on slightly divergent trajectories
- Pre-split interception: difficult (high altitude, need excellent radar range and fast interceptors) but eliminates all warheads in one shot. High risk — if you miss, you've wasted an interceptor and the MIRV still splits.
- Post-split interception: each warhead must be engaged individually. Warheads spread over time, so earlier post-split engagement means they're still clustered (chain reaction opportunity) but hard to distinguish. Later means they're spread out and easier to track individually but require more interceptors.
- The MIRV forces a genuine strategic dilemma every time one appears on radar

---

### Strategic Campaign (Meta Layer)

#### Territory & Expansion

The campaign takes place on a stylized Cold War-era national map divided into regions. Each region has a distinct terrain profile that affects gameplay.

**Starting Position:**
- Player begins with a small homeland territory: 3–4 cities, 1–2 battery positions, basic radar coverage
- This opening territory serves as the tutorial space — simple terrain, manageable wave sizes, room to learn the physics

**Expansion Mechanics:**
- Between wave sets, the player can choose to expand into adjacent regions
- Each new region offers:
  - Additional cities (population = resources, but also more to defend)
  - New battery placement positions (some with terrain advantages)
  - Terrain features that alter offensive and defensive geometry
  - Potential resource bonuses (industrial regions produce more upgrade currency, etc.)
- Expanding increases your resource base but stretches defenses thinner — the fundamental strategic tension
- Enemy attack patterns adapt to your territory: more spread-out territory means attacks come from more directions and target more locations simultaneously

**Terrain Types & Tactical Effects:**
- **Mountains:** Block low-angle missile trajectories from certain directions, creating natural defensive walls. Batteries placed on high ground have extended range. However, valleys between mountains funnel attacks into predictable corridors — which is an advantage if you defend them and a disaster if you don't.
- **Coastal regions:** Open to sea-launched missile attacks (different trajectory profiles — lower, faster, shorter warning time). Provide access to naval radar installations with excellent over-water detection range.
- **Plains/Flatlands:** No terrain advantages or disadvantages. Maximum battery placement flexibility but no natural cover. Ideal for radar installations (no terrain occlusion). Vulnerable from all directions.
- **Urban/Industrial zones:** High population density and resource output. High-value, high-risk territories.

#### Between-Wave Management

After each wave (or wave set), the player enters a strategic planning phase:

**Resource Allocation:**
- Resources are generated based on surviving population and infrastructure
- Spent on: new batteries, interceptor stockpiles, radar upgrades, infrastructure repair, territory expansion, new interceptor types
- There is never enough. The core strategic tension is always "invest in defense of what I have" vs. "expand to grow my resource base"

**Battery Placement:**
- New batteries can be placed at available positions within controlled territory
- Position selection is critical: elevation, sight lines, proximity to cities, overlapping fields of fire
- Batteries can be relocated between waves at a cost
- Each battery has a limited interceptor stockpile that must be replenished between waves

**Radar Network:**
- Radar installations provide detection range — further range means earlier warning, which means higher-altitude (easier) intercepts
- Multiple radar installations provide overlapping coverage and redundancy
- Radar can be upgraded for range, resolution (distinguishing MIRVs from decoys earlier), and weather resistance
- Radar installations are targetable by enemy attacks — losing radar coverage mid-wave is a crisis event

**Interceptor Upgrades:**
- Unlock and upgrade different interceptor types over the campaign:
  - **Standard interceptor:** Balanced speed, range, blast radius. The workhorse.
  - **Sprint interceptor:** Fast burn, short range, small blast radius. Last-ditch terminal defense. Effective against fast low-angle attacks.
  - **Exoatmospheric interceptor:** Slow launch, very high ceiling, wide blast radius at altitude. Designed for apogee intercepts and pre-split MIRV kills.
  - **Area denial interceptor:** Produces a lingering shockwave zone that damages anything passing through it. Excellent for corridor defense in mountainous terrain.
- Upgrades improve specific characteristics: thrust (faster arc, higher ceiling), warhead yield (larger blast radius), guidance (tighter predicted arc, less deviation)

**Infrastructure & Repair:**
- Damaged cities can be repaired between waves, but repair is expensive
- Destroyed cities can be rebuilt at extreme cost — or abandoned
- Civilian shelters can be built to reduce population loss from hits (population survives but infrastructure is damaged)
- Industrial facilities boost resource generation but are high-value targets

---

### Weather System

Weather varies between waves and affects multiple gameplay systems:

- **Clear skies:** Full re-entry glow visibility, maximum radar effectiveness, baseline physics
- **Overcast/Clouds:** Re-entry glow partially obscured (visible only at very low altitude), radar slightly degraded, no other effects
- **Storms:** Re-entry glow invisible, radar significantly degraded, wind at altitude can subtly push interceptor arcs off-target (visible in the predicted arc overlay so the player can compensate)
- **Severe storms (rare):** Extreme radar degradation, significant wind effects, interceptor accuracy reduced. These waves are survival scenarios — you're defending nearly blind

Weather is announced before each wave in the intel briefing, allowing strategic preparation (e.g., launching extra interceptors preemptively, relying on overlapping radar coverage, pulling back to defend only critical assets).

---

## Difficulty & Progression

### Wave Escalation (Not Physics Escalation)

The physics simulation is identical on Wave 1 and Wave 50. What changes:

| Wave Range | Enemy Capabilities | Player Capabilities |
|---|---|---|
| 1–5 | Few missiles, slow, high arcs, single direction | 1–2 batteries, standard interceptors, basic radar |
| 6–15 | More missiles, varied speeds and angles, multiple directions | Additional batteries, first upgrades, territory expansion begins |
| 16–25 | Fast low-angle missiles, clustered salvos (chain reaction opportunities), first decoys | Sprint interceptors, radar upgrades, weather variation introduced |
| 26–40 | MIRVs introduced, mixed warhead types, radar-targeted strikes | Exoatmospheric interceptors, area denial, shelter construction |
| 40+ | Full escalation — MIRVs, saturation attacks, electronic warfare, severe weather events | Full tech tree, large territory, multiple radar networks |

### Failure States & Recovery

- **Wave failure:** Losing all cities in a region means losing that region. Territory contracts.
- **Campaign failure:** Losing your homeland (original starting territory) ends the campaign.
- **Recovery:** Losing outlying regions is painful but survivable. The game is designed so that strategic retreat and consolidation is a valid (sometimes optimal) strategy. You might deliberately let a frontier region fall to concentrate defenses.
- **No single wave is unwinnable.** Even late-game saturation attacks can be survived through smart triage — you might lose cities but preserve enough to continue.

---

## Architecture & Technology

### Tauri Application Structure

Deterrence is a Tauri desktop application with a clear separation between the physics simulation (Rust backend) and the rendering/UI layer (PixiJS frontend).

**Rust Backend (tauri core + game engine):**
- Owns the authoritative physics simulation: gravity, drag, ballistic trajectories, blast propagation, shockwave interactions, chain reactions
- Runs the simulation at a fixed timestep (e.g., 60 Hz physics tick) independent of rendering frame rate, ensuring deterministic and reproducible behavior
- Manages all game state: wave composition, entity positions/velocities, battery ammo, campaign progress, resource economy
- Exposes game state to the frontend via Tauri's IPC (invoke commands and event system)
- Handles save/load (local filesystem via Tauri's path API), wave seeding, and campaign progression logic
- Benefits: Rust's performance guarantees mean the physics simulation can handle saturation attacks (100+ simultaneous entities) without frame drops affecting simulation accuracy

**PixiJS Frontend (WebGL rendering):**
- Receives game state snapshots from the Rust backend each frame and renders the scene
- Handles all visual presentation: missile trails, trajectory overlays, detonation effects, shockwave visualization, CRT post-processing, weather overlays, re-entry glow
- Manages player input (mouse, keyboard) and sends commands back to the Rust backend (e.g., "launch interceptor from battery 2 targeting coordinates X,Y")
- Owns the UI layer: strategic map, upgrade menus, intel briefings, resource displays
- PixiJS's WebGL particle systems handle explosions, shockwave rings, and weather effects
- Custom WebGL shaders for the CRT aesthetic: phosphor glow, scanlines, film grain, bloom on detonations, vignetting

**IPC Flow (per frame during wave):**
1. Frontend sends player input events → Rust backend
2. Rust backend advances physics simulation by one tick, processes any new interceptor launches
3. Rust backend emits updated game state (entity positions, new detonations, damage events) → Frontend
4. Frontend interpolates between state snapshots for smooth rendering and triggers visual/audio effects

**IPC Flow (strategic phase):**
1. Frontend sends player decisions (place battery, upgrade radar, expand territory) → Rust backend
2. Rust backend validates and applies changes to campaign state
3. Rust backend emits updated campaign state → Frontend re-renders strategic map

### Key Technical Decisions

- **Physics in Rust, not JS:** The physics simulation must be deterministic and frame-rate independent. Rust's fixed-point-friendly numerics and lack of GC pauses make it the right place for this. PixiJS should never run physics — it only renders.
- **State authority:** Rust backend is the single source of truth for all game state. The frontend is a "dumb" renderer that displays what it's told. This makes save/load trivial and prevents desync bugs.
- **Interpolation for smoothness:** The frontend renders at monitor refresh rate (typically 60–144 Hz) and interpolates between physics ticks for smooth motion, even if the physics tick rate differs from the display rate.

---

## Controls & Input

### Mouse & Keyboard

- **Click:** Set intercept target point
- **Right-click:** Show detailed info on incoming missile (trajectory, estimated impact point, speed, type)
- **Click + Drag:** Adjust intercept target before confirming
- **Scroll wheel:** Zoom between tactical and strategic views
- **Number keys (1–9):** Direct battery selection — press a battery's number then click to launch from that specific battery
- **Tab:** Cycle between batteries
- **Spacebar:** Pause (strategy phase only) for unhurried planning
- **Keyboard shortcuts for strategic phase:** Hotkeys for repair, upgrade, and placement menus
- **Escape:** Open settings / menu

### Desktop Layout Advantages

- Large screen enables a persistent strategic mini-map alongside the tactical view
- Mouse precision allows for tight interceptor placement under pressure
- Keyboard hotkeys enable faster battery switching during saturation attacks
- Wide viewport can show more of the territory simultaneously, rewarding peripheral awareness

---

## Session & Save Design

- Each wave is 60–120 seconds of active play
- Between-wave strategy phase is self-paced
- A typical session (3–5 waves + strategy) runs 10–15 minutes
- Sessions can trend longer (20–30 minutes) for players who want sustained immersion — the game accommodates this without penalizing shorter sessions
- Campaign progress auto-saves to local filesystem between waves via Tauri's path API
- Multiple save slots for different campaigns

### Visual Design (Cold War Retro-Strategic)
- Primary display: Dark background with vector-line radar aesthetic (green/amber phosphor CRT look)
- Incoming missiles: Bright trails with color-coding by type (standard, MIRV, fast-attack)
- Interceptor arcs: Dotted predicted path, solid actual path
- Detonations: Expanding circle with brightness falloff (shockwave visualization) using PixiJS particle emitters
- Cities: Simplified geometric skylines with population indicators
- Terrain: Topographic contour lines, elevation shading
- UI elements: Military-stencil typography, teletype-style text readouts, analog gauge-style indicators for ammo and radar status
- Weather: Atmospheric overlays — cloud layers that visually occlude the upper screen, rain static on the radar display, wind streaks
- Re-entry glow: Bright orange-white streak against the dark sky, visible above cloud layers in clear weather, masked by cloud cover
- **CRT post-processing pipeline (custom WebGL shaders via PixiJS):** Phosphor glow, scanline overlay, subtle screen curvature distortion, bloom on detonations, film grain, vignetting. These shaders are lightweight and run as a final compositing pass over the rendered scene.

---

## Key Metrics & Balancing Targets

- **Average wave survival rate:** 70–80% of cities surviving in early waves, declining to 40–60% in late waves for an average player. The game should feel like you're always *barely* holding on.
- **Interceptor efficiency:** Players should use 1.5–2.5 interceptors per destroyed warhead on average. Chain reactions should bring this below 1.0 in optimal scenarios, rewarding skill.
- **Session length:** Target 10–15 minutes for a satisfying play session (3–5 waves).
- **Campaign length:** 40–60 waves for a full campaign playthrough, approximately 8–12 hours total.
- **Expansion pacing:** Players should control 2–3 regions by Wave 10, 4–6 by Wave 25, and face contraction decisions by Wave 35+.

---

## Open Questions & Future Considerations

1. **Retaliation mechanic (future iteration):** Should the player have the option to launch retaliatory strikes between waves? Could reduce future wave intensity but escalate long-term difficulty. Fits the Cold War theme of mutually assured destruction. Deferred to a future version.
2. **Intel & espionage layer:** Pre-wave intelligence briefings that give partial information about the incoming attack. Invest in intelligence to get better briefings. Possible disinformation from the enemy. This should be explored during design iteration — it deepens the strategic phase and fits the Cold War theme naturally.
3. **Soundtrack:** Custom original soundtrack to be created. Target aesthetic: Cold War-era emergency broadcast tones, shortwave radio static, period-appropriate tension music. Audio design is critical for the bunker atmosphere.
4. **Mobile port:** Tauri v2 supports iOS and Android targets. A future mobile version could adapt controls to touch (tap-to-target, pinch-to-zoom) and add haptic feedback for detonations and impacts. The Rust physics backend would port unchanged.

---

## Technical Notes

- **Tauri v2** as the application shell — provides native window management, filesystem access, and IPC between Rust backend and webview frontend
- **Rust physics engine** runs at fixed timestep (e.g., 60 Hz) independent of frame rate for deterministic consistency
- Ballistic trajectory calculations use standard projectile motion with altitude-dependent drag coefficient
- Blast propagation modeled as expanding wavefront with inverse-square falloff, modified by atmospheric density
- Weather effects on radar modeled as range multiplier (0.3x in severe storms to 1.0x in clear conditions)
- All random elements (weather, attack patterns) seeded per-wave for reproducibility and fairness — Rust's deterministic RNG makes this trivial
- **PixiJS v8** for WebGL 2 rendering — particle systems for explosions/weather, custom GLSL shaders for CRT post-processing pipeline
- Frontend interpolates between physics state snapshots for smooth rendering at monitor refresh rate
- Save data stored locally via Tauri's path API (app data directory), serialized as JSON or MessagePack
- **Build targets:** `.msi` / `.exe` (Windows), `.dmg` / `.app` (macOS), `.AppImage` / `.deb` (Linux) — all from a single codebase via Tauri's bundler
- Minimum spec target: integrated GPU from ~2018 (Intel UHD 620 equivalent) — PixiJS 2D rendering is lightweight; the CRT shader pipeline is the heaviest visual cost

---

*"The only winning move is to keep playing."*
