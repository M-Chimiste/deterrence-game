# DETERRENCE — Product Context

## The Problem

There is no game that accurately captures the cognitive experience of modern air and missile defense operations. The market has:

- **Arcade missile defense** (Missile Command, Iron Dome mobile games): Fun but mechanically shallow. Direct weapon control, no information management, no resource tradeoffs beyond ammo count.
- **Full-spectrum military sims** (Command: Modern Operations / CMANO): Comprehensive but overwhelming. Spreadsheet-heavy, designed for grognards, minimal dramatic payoff, no progressive onboarding.
- **Naval action games** (World of Warships, Sea Power): Focus on platform maneuvering and offensive combat. Missile defense is either absent or trivially automated.
- **Tower defense**: Superficially similar resource mechanics but entirely wrong frame. Enemies follow paths; the player places static defenses. No information ambiguity, no identification dilemma, no supervisory automation tension.

The gap: **a game that makes the IAMD operator experience — supervisory control of automated systems under information uncertainty — into compelling gameplay.** This is an untapped design space that produces a fundamentally different kind of tension than any existing genre.

---

## Who Is This For

### Primary Audience: The Tactical Thinker

Players who enjoy systems-driven decision-making under pressure. They gravitate toward games where mastery comes from understanding interacting systems rather than reflexes.

**Profile:**
- Plays: Highfleet, CMANO/Command, Into the Breach, FTL, Rimworld, Kerbal Space Program
- Age: 25-45
- Values: Depth over flash, authenticity over spectacle, meaningful decisions over content volume
- Platform: Desktop (PC primary, Mac secondary)
- Engagement: 30-90 minute sessions, high replay value

**What they want:**
- Systems that interact in non-obvious ways (radar budget affecting detection affecting engagement windows)
- Meaningful resource scarcity that forces tradeoffs (not just "run out of ammo")
- Authentic feel without requiring a military manual to understand
- Procedural variety that keeps each session fresh
- That moment where you realize a decision you made 5 minutes ago just created a cascading problem

### Secondary Audience: Military & Defense Enthusiasts

People with professional or hobbyist interest in military systems, defense technology, and naval/air defense operations. Includes veterans, active duty, defense industry professionals, and military history enthusiasts.

**What they want:**
- Recognition: "This is how it actually works" moments
- Authentic terminology, procedures, and system behavior
- Fidelity in the details that matter (symbology, engagement sequence, system constraints)
- A way to explain what they do (or did) to non-military friends through gameplay

### Tertiary Audience: Casual Strategy Gamers

Players who enjoy strategy games but aren't specifically drawn to military simulation. They'll discover DETERRENCE through word-of-mouth, streamers, or the unique aesthetic.

**What they want:**
- Clear onboarding that teaches without overwhelming
- A difficulty curve that lets them engage at their comfort level
- The "cool factor" of operating a military radar console
- Enough depth to reward investment without requiring exhaustive study

---

## User Stories

### Operator Experience (Core Loop)

> **As a player**, I want to see contacts appear on my radar display as ambiguous returns that I must classify and prioritize, **so that** every engagement decision carries the weight of uncertainty.

> **As a player**, I want the system to automatically engage threats unless I intervene, **so that** my critical skill is knowing when NOT to fire — not just choosing targets.

> **As a player**, I want to manage a finite radar energy budget that degrades as I track more contacts, **so that** I must make strategic tradeoffs between searching for new threats and maintaining existing tracks.

> **As a player**, I want limited illuminator channels that bottleneck simultaneous terminal engagements, **so that** I must think about engagement timing and weapon selection, not just "shoot everything."

> **As a player**, I want civilian air traffic and neutral contacts mixed into the tactical picture, **so that** the identification dilemma creates genuine moral and tactical tension — not just target-shooting.

### Ground Operations

> **As a player**, I want to place my Patriot battery on a terrain map before the mission starts and see my radar coverage fan update in real-time, **so that** my emplacement decision feels consequential and strategic.

> **As a player**, I want my ground radar to only cover 120 degrees with terrain masking creating blind spots, **so that** I must think carefully about sector orientation and accept that I cannot see everything.

> **As a player**, I want anti-radiation missiles to target my radar when I'm emitting, **so that** I face a survival dilemma: stay on the air to track threats, or go silent and rely on the network.

> **As a player**, I want to manage launcher reloads during active engagements, **so that** pulling a launcher offline for reload becomes a risk/reward decision — not just a timer.

### Progression & Campaign

> **As a new player**, I want to start with a single ship and one threat type, **so that** I learn one system at a time without being overwhelmed.

> **As an experienced player**, I want theater campaigns where expended missiles don't fully replenish between missions, **so that** resource conservation in early missions pays off during the climactic final engagement.

> **As an advanced player**, I want joint operations where I command both naval and ground assets simultaneously, **so that** cross-domain coordination creates emergent tactical problems.

### Networking (Cooperative)

> **As a player in a multiplayer session**, I want to operate as a forward sensor picket providing targeting data to a rear missile battery, **so that** cooperative play creates role specialization and communication-dependent gameplay.

> **As a player**, I want network degradation (from jamming or node destruction) to force my battery into autonomous operation with only its own sensor, **so that** network health is a resource I must protect and plan around.

---

## User Experience Goals

### How It Should Feel

**The CIC at 0300.** Dark room. Screen glow. The hum of electronics. You're alert but steady — scanning, classifying, monitoring. Then multiple contacts appear, closing fast, and the Veto Clocks start ticking. Your pulse rises. You're triaging across three simultaneous engagement decisions. One contact isn't squawking IFF but its flight profile matches a commercial airway. The system says HOSTILE. You have 6 seconds. Now it's a different kind of game.

**Key emotional beats:**
1. **Calm vigilance** — The early watch. Scanning, classifying routine contacts, managing the picture. Low tension, building familiarity.
2. **Rising alertness** — First hostile contacts. System works as designed. Engagements succeed. Confidence builds.
3. **Controlled pressure** — Saturation begins. Multiple threats, resource competition, Veto Clocks overlapping. The player is working hard but managing.
4. **Crisis** — The moment where everything stacks: illuminators full, a new wave inbound, one contact is ambiguous, an ARM is targeting your radar, and you have to decide what to sacrifice.
5. **Resolution** — Either you held the line (catharsis) or something got through (consequence). Both should feel earned.

### Aesthetic Principles

- **The console IS the game.** The dark CIC with glowing screens isn't decoration — it's the entire experience. Every visual choice reinforces that you're operating a real system.
- **Information density, not visual noise.** Every pixel on the tactical display means something. No decorative UI chrome. No unnecessary animation. The beauty is in the data.
- **Sound tells the story.** Audio does the emotional work that the utilitarian visuals don't. The escalation from quiet CIC hum to overlapping alarms and urgent callouts IS the difficulty curve rendered in sound.
- **The 3D view is the reward.** You make decisions in the cave of the CIC. The 3D view shows you what those decisions meant in the physical world. The contrast between the abstract radar display and the visceral 3D intercept is the payoff.

### Onboarding Philosophy

**Progressive disclosure, not front-loaded tutorials.**

The player learns by doing, with complexity introduced through career progression:

| Tier | What's New | What's Hidden |
|---|---|---|
| 1 — Cadet | PPI display, track hooking, basic classification, single threat type, manual engagement | Veto Clock (manual fire only), no EW, no civilians, no environment |
| 2 — Officer | Veto Clock (AUTO mode), multiple threat types, illuminator management, IFF, civilian traffic | No SEAD, no terrain, no networking, standard environment |
| 3 — Commander | Full naval or ground domain, all threats, EW, environment model, CEC/IBCS | No joint ops, no campaign persistence |
| 4 — Flag | Joint operations, campaign mode, full complexity, IAMD commander role | Nothing hidden — full system |

Each tier introduces 2-3 new concepts maximum. The player should feel competent at their current tier before encountering the next.

---

## Market Context

### Competitive Landscape

| Game | Overlap | Key Difference |
|---|---|---|
| Command: Modern Operations | Closest in scope — full naval/air simulation | CMANO is comprehensive but overwhelming; DETERRENCE is focused and visceral |
| Highfleet | Information warfare aesthetic, radar-based gameplay | Highfleet is adventure/RPG hybrid; DETERRENCE is pure tactical simulation |
| DEFCON | Nuclear anxiety aesthetic, real-time strategy | DEFCON is abstract and strategic; DETERRENCE is operational and system-focused |
| Into the Breach | Perfect-information tactical puzzle, resource management | ITB is turn-based and deterministic; DETERRENCE is real-time with uncertainty |
| FTL | Resource management under pressure, procedural missions | FTL is roguelike with RPG progression; DETERRENCE is simulation with career progression |
| Iron Dome (mobile games) | Missile defense theme | Arcade gameplay with no simulation depth |

### Market Position

DETERRENCE occupies the space between CMANO's exhaustive simulation and Missile Command's arcade simplicity. It is:
- **More accessible** than CMANO (progressive onboarding, focused scope, dramatic payoff)
- **More authentic** than any arcade missile defense game (real physics, real procedures, real dilemmas)
- **More focused** than general strategy games (one thing done deeply, not many things done broadly)

### Distribution

- **Primary**: Steam (PC/Mac/Linux)
- **Secondary**: itch.io for early access / community building
- **Tertiary**: Direct distribution via website (Tauri auto-updater)

### Pricing Model

Premium ($19.99-$29.99) with no microtransactions. Potential for paid theater expansion packs post-launch. Modding support extends value without ongoing content investment.

---

## What Success Looks Like

**6 months post-launch:**
- Players describe the Veto Clock as one of the most unique mechanics in strategy gaming
- Military veterans and defense professionals recognize and validate the system behavior
- A modding community has created custom theaters and scenarios
- Streamers find the CIC aesthetic and crisis moments compelling to broadcast
- The game is referenced in conversations about "games that do something different"

**The sentence we want in reviews:**
> "DETERRENCE made me understand what it feels like to be the person behind the radar screen — and why that's terrifying."
