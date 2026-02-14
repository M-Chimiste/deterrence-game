# DETERRENCE — Project Brief

## Project Overview

**DETERRENCE** is a real-time tactical simulation of Integrated Air and Missile Defense (IAMD). The player operates as a battle manager supervising automated defense systems across naval (AEGIS) and ground-based (Patriot/THAAD) platforms, making high-stakes identification and engagement decisions through authentic CIC/ECS radar console interfaces.

Built as a cross-platform desktop application using Tauri 2.x (Rust backend) + Three.js/TypeScript frontend.

---

## North Star

**Create the definitive IAMD simulation — a game where the core tension is information management, not twitch reflexes. The player's skill is cognitive: interpreting ambiguous data, managing finite resources, and deciding when to trust or override automation under extreme time pressure.**

The game succeeds when a player, staring at a yellow quatrefoil on a dark screen with a Veto Clock ticking down, feels the weight of that decision in their chest.

---

## High-Level Goals

1. **Authenticity over realism**: Model the *experience* of operating AEGIS/Patriot systems faithfully, not every technical detail. The physics must produce correct behavioral outcomes. The UI must feel like a real console. The decisions must map to real operator dilemmas.

2. **Cognitive gameplay**: The core skill is information processing and resource optimization under time pressure. No twitch mechanics. No direct weapon control. The player supervises, interprets, and decides.

3. **Dual-domain IAMD**: Naval and ground-based missile defense as two distinct but mechanically unified experiences. Each domain has unique constraints that create different tactical problems from the same core systems.

4. **Procedural replayability**: Every mission is generated from real-world theaters and plausible conflict parameters. No two sessions are identical. The generator produces solvable scenarios that feel handcrafted.

5. **Progressive mastery**: A clear skill progression from single-battery operator to theater IAMD commander. Each tier introduces new systems and complexity without invalidating previous knowledge.

---

## Success Criteria

### MVP (Phase 1 — 12-16 weeks)

- [ ] Core DCIE loop (Detect → Classify → Identify → Engage) feels tense and rewarding in playtesting
- [ ] Veto Clock creates genuine dilemma moments — players hesitate, second-guess, and feel consequences
- [ ] Radar energy budget tradeoff is intuitively understood within 2-3 missions
- [ ] Illuminator bottleneck creates meaningful resource tension when 4+ threats are terminal
- [ ] 3D cinematic view provides satisfying payoff for decisions made on the radar console
- [ ] Simulation sustains 30Hz tick rate with 100+ simultaneous entities on mid-range hardware
- [ ] Frontend sustains 60fps with full PPI display and basic 3D PIP view
- [ ] "One more mission" compulsion validated through external playtesting

### Full Release (Phase 4)

- [ ] Naval and ground domains feel mechanically distinct but unified by shared design language
- [ ] Emplacement planning produces meaningful strategic decisions with lasting consequences
- [ ] Joint operations create emergent cross-domain tactical problems not present in either domain alone
- [ ] Campaign progression drives long-term engagement through resource persistence and escalation
- [ ] Procedural generation produces varied, replayable missions across 15+ theaters
- [ ] Modding community can create custom theaters, scenarios, and threat profiles

### Technical Benchmarks

| Metric | Target |
|---|---|
| Simulation tick rate | 30Hz sustained, 500+ entities |
| Frontend FPS | 60fps, full PPI + 3D PIP, integrated GPU |
| IPC latency | < 5ms per game state broadcast |
| Application size | < 50 MB base (excl. terrain data) |
| Startup to main menu | < 3 seconds |
| Mission load (incl. terrain) | < 5 seconds |
| Memory (complex joint scenario) | < 500 MB |

---

## Core Requirements

### Must Have (MVP)

- Rust simulation engine with radar detection model, fire control loop, and engagement state machine
- Tauri 2.x application shell with typed IPC bridge (Rust ↔ TypeScript)
- PPI tactical display with NTDS symbology, track hooking, velocity leaders, history trails
- Veto Clock mechanic with AUTO-SPECIAL doctrine mode
- Radar energy budget with visible search/track tradeoff
- Illuminator channel management (3-4 channels, time-share scheduling)
- At least one threat archetype (subsonic sea-skimming ASCM)
- At least one interceptor archetype (SM-2 equivalent with semi-active terminal guidance)
- Basic 3D world view (ocean, skybox, missile trails, intercept effects)
- VLS status panel and threat evaluation table
- Mission scoring (threats neutralized, assets protected, interceptors expended)

### Must Have (Full Release)

- Full threat archetype roster (12+ types across naval and ground domains)
- Full interceptor roster (9+ types)
- Ground domain: Patriot battery simulation with 120° sector radar, launcher management, reload mechanic
- Ground domain: THAAD simulation with TPY-2 radar modes
- Emplacement planning phase with terrain-aware coverage visualization
- SEAD mechanic: ARM threats, EMCON management, displacement
- Terrain system: heightmap-based LOS calculation and terrain masking
- Environmental model: ducting, sea state, weather, clutter
- Electronic warfare: noise/deceptive/DRFM jamming, burn-through
- IFF system and civilian traffic generation for identification dilemma
- CEC (naval) and IBCS (ground) networking with composite tracks and engage-on-remote
- Procedural mission generator across all theaters
- Campaign mode with persistent consequences
- A-Scope display for manual signal analysis
- Full audio: CIC ambient, alert tones, voice callouts, tension layers

### Should Have

- Joint operations (simultaneous naval + ground defense)
- IAMD Commander role (theater-level view)
- Career progression (4-tier operator → commander)
- MIL-STD-2525 symbology option alongside NTDS
- Ship maneuvering (naval course/speed control)
- Modding support (exposed data files for community content)
- Replay/after-action review with full revealed ground truth
- Cinematic auto-camera for 3D intercept moments

### Nice to Have

- Multiplayer cooperative (CEC/IBCS roles)
- Streaming/spectator mode
- Historical scenario recreations (curated missions based on real incidents)
- Ballistic missile defense discrimination (warhead vs. decoy vs. debris)
- Advanced campaign branching with political consequences
- Custom UI layout sharing

### Out of Scope

- First-person or third-person gameplay perspectives
- Direct missile control or flight simulation mechanics
- Submarine warfare or ASW mechanics
- Land combat / ground force simulation
- Mobile platform (iOS/Android) — desktop only
- VR/AR support
- Online competitive multiplayer (cooperative only if multiplayer is implemented)
- Classified or restricted information — all reference material is public domain

---

## Project Scope

### Domain Boundaries

The simulation models **air and missile defense** only. This includes:
- Radar-based detection and tracking of airborne threats
- Fire control and interceptor engagement
- Ballistic missile defense
- Electronic warfare (as it affects detection and engagement)
- Cooperative sensor networking

It explicitly **does not** model:
- Offensive strike operations (no Tomahawk mission planning)
- Anti-submarine warfare
- Surface warfare (ship-to-ship combat)
- Ground combat or infantry operations
- Logistics beyond interceptor inventory and reload

### Fidelity Boundaries

The simulation targets **behavioral fidelity**, not engineering fidelity:
- Radar detection is statistical, not ray-traced
- Missile flight is state-machine-based, not 6DOF
- Engagement probability uses weighted Pk models, not physics-based terminal dynamics
- Environmental effects are parameterized, not fluid-dynamics simulated

The standard: *would a naval officer or air defender recognize the behavior as correct?* If yes, the fidelity is sufficient.

### Timeline

| Phase | Milestone | Target Duration |
|---|---|---|
| 1 | Core Loop MVP | 12-16 weeks |
| 2 | Naval Complete | 10-14 weeks |
| 3 | Ground Expansion | 10-14 weeks |
| 4 | Joint & Campaign | 8-12 weeks |

Total estimated: 40-56 weeks to full release.

---

## Risks

| Risk | Impact | Likelihood | Mitigation |
|---|---|---|---|
| Simulation perf bottleneck at IPC boundary | High | Medium | Profile early. MessagePack fallback. State diffs. |
| Radar model too complex to balance | High | Medium | Simplified MVP model. All params data-driven. Iterate from playtesting. |
| Terrain LOS calculation perf (ground) | Medium | Medium | Pre-compute masking tables at mission load. Hierarchical queries. |
| Scope creep from ground expansion | High | High | Strict phase gating. Naval must be complete + fun first. |
| Player onboarding too steep | High | High | Progressive disclosure via career tiers. Interactive tutorials. |
| Tauri 2.x ecosystem maturity | Medium | Low | Pin deps. Minimal plugin reliance. Standard web APIs where possible. |
