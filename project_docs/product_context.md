# Deterrence — Product Context

**Who is this for and why does it matter?**

---

## The Problem Being Solved

The strategy gaming space on desktop has a gap between two extremes. On one side, there are arcade action games with simple mechanics and no lasting depth — satisfying for minutes, forgotten within days. On the other side, there are sprawling grand strategy titles that demand dozens of hours before they become interesting and hundreds more before they feel mastered.

Deterrence sits in the space between them. It addresses three specific unmet needs:

**Nobody has made Missile Command grow up.** The original Missile Command (1980) is one of the most mechanically elegant arcade games ever designed. Its core loop — triage under pressure, deciding what to save and what to sacrifice — is inherently strategic. But the arcade format never developed that potential. Every modern homage either preserves the arcade simplicity without adding depth, or abandons the core loop entirely in favor of tower defense conventions. No one has taken the Missile Command premise and asked: what if the physics were real, the consequences persisted, and there was a strategic layer on top?

**Physics-based games rarely trust their own physics.** Many games advertise "realistic physics" but simplify or override the simulation to keep things accessible. Deterrence takes the opposite approach: the physics are consistent from the first wave to the last. Players develop genuine physical intuition — understanding trajectories, blast behavior, and intercept geometry — that compounds into mastery over the full campaign. The game teaches through scenario simplicity (fewer, slower missiles early on), never through system simplification.

**Short-session strategy games are underserved on desktop.** The desktop strategy market skews heavily toward long-session experiences (Civilization, EU4, Total War). Players who want meaningful strategic decisions in 10–15 minute sessions — the kind of play that fits into a lunch break or a gap between meetings — have limited options that don't feel shallow. Deterrence offers a campaign with genuine strategic weight (territory management, resource allocation, tech progression) structured around short tactical engagements.

---

## Market Context

**Competitive landscape:**

The missile defense subgenre is remarkably thin. Most entries are direct Missile Command clones with cosmetic updates, mobile free-to-play adaptations, or VR novelty experiences. None combine physics simulation with strategic campaign depth. The closest analogues come from adjacent genres: Into the Breach offers turn-based tactical defense with persistent consequences, while Kingdom: Two Crowns layers strategy onto a simple real-time defense loop. Both demonstrate appetite for "defense games with brains."

**Audience positioning:**

Deterrence targets strategy gamers who enjoy systems-heavy games (Factorio, Into the Breach, FTL, Rimworld) but also appreciate elegant core mechanics. These players want to understand *why* something works, not just *that* it works. The physics simulation appeals to this instinct — every interception is a mini-problem with a knowable solution, and mastery comes from understanding the system rather than memorizing patterns.

The Cold War retro-strategic aesthetic differentiates sharply from the dominant visual trends in indie strategy (pixel art, minimalist flat design, sci-fi neon). It evokes a specific time and tension that resonates culturally and has proven appeal (see: the enduring popularity of Cold War settings in film, television, and board games like Twilight Struggle).

**Platform rationale:**

Desktop-first via Tauri means reaching Windows, macOS, and Linux from a single codebase. The strategy gaming audience skews heavily toward desktop (particularly PC), and the mouse/keyboard input model is ideal for the precision clicking and hotkey-driven multitasking the game demands during high-pressure waves. A future mobile port is architecturally feasible (the Rust physics backend ports unchanged) but the desktop experience is the primary design target.

**Distribution:** Steam is the natural home. The game's genre, aesthetic, session length, and audience all align with Steam's core demographics. The "easy to learn, deep to master" structure and 8–12 hour campaign length position it well for the mid-price indie tier ($10–$20).

---

## User Stories

### New Player (First Session)

*"I downloaded Deterrence because the Cold War aesthetic caught my eye in a Steam trailer. I start the campaign and I'm defending a small cluster of cities. Missiles arc in from the edges of the screen — not in straight lines, but in actual curves. I click the sky to launch an interceptor and watch it trace its own arc toward the target. I miss the first one because I didn't account for how long my interceptor takes to reach that altitude. But I see the predicted arc before I commit, so on the next shot I adjust. By the end of Wave 3, I'm starting to feel the physics intuitively — leading my targets, choosing intercept points where the incoming missile is slowest. I don't fully understand blast radius yet, but I got a lucky double kill when two missiles were close together and I'm curious about how that happened."*

### Intermediate Player (Mid-Campaign)

*"I'm about 15 waves in and I've expanded into three regions. I have mountains on my eastern border that block low-angle attacks, so I've stacked my western defenses instead. I just unlocked sprint interceptors and I'm learning when to use them versus the standard ones — sprints are my emergency option for missiles I detect late, but they burn through ammo fast. Last session I had a storm roll in and my radar range dropped to almost nothing. I spotted a re-entry glow streak through a gap in the clouds and managed to intercept it manually. Felt incredible. I'm starting to think about where I want to expand next — the coastal region has a naval radar site that would give me great detection range over the ocean, but it also opens me up to sea-launched attacks from a new direction."*

### Advanced Player (Late Campaign)

*"I'm managing six regions and it's getting brutal. MIRVs are showing up regularly and I've been burned twice by trying the pre-split intercept and missing. Now I have a system: I use my exoatmospheric interceptor for the pre-split attempt, and if it misses, I have area denial interceptors positioned to catch the post-split cluster before they spread. My eastern mountain range took sustained bombardment last session and the terrain advantage is weakening — I'm considering pulling back and concentrating on my industrial core. I deliberately let a frontier region fall last campaign and it was actually the right call. Resource management between waves is where the real game is now. I spend as much time in the strategic phase as I do in the tactical waves."*

### Returning Player (Repeat Campaigns)

*"I've finished two full campaigns with different expansion strategies. First run I expanded aggressively and nearly collapsed from overextension around Wave 35. Second run I played compact and defensive — survived longer but hit a resource ceiling. Now I'm trying a hybrid approach: expand fast early, consolidate hard around Wave 20, then selectively push into high-value regions. Every campaign feels different because the terrain choices change everything about how attacks come in and how I position my network. The physics never change but the strategic puzzle is always fresh."*

---

## User Experience Goals

### Immediate (First 5 Minutes)

- The player understands the core loop within one wave: missiles come in, you click to intercept, your interceptor follows an arc, the detonation either catches the warhead or doesn't
- The predicted arc overlay teaches physics intuitively — the player sees *why* their interceptor will or won't reach the target before they commit
- The CRT aesthetic and audio immediately establish mood — this isn't a casual game, it's a Cold War command bunker
- Failure is visible and meaningful: a city getting hit produces a clear, impactful moment that the player feels, not an abstract score decrement

### Short-Term (First Hour)

- The player has developed basic trajectory intuition — they can eyeball whether an incoming missile is a high lob or a fast shallow attack and adjust their intercept strategy accordingly
- The strategic between-wave phase has been introduced and the player understands the core tension: spend resources on defense vs. invest in expansion
- The player has experienced at least one moment of emergent physics (a chain reaction, a near-miss deflection, a last-second sprint intercept) that surprised them and made them want to understand the system better
- Session pacing feels right — the player can put the game down after 10–15 minutes and feel like they accomplished something

### Medium-Term (Full Campaign)

- The player feels genuine ownership of their defense network — they placed every battery, chose every upgrade, decided which regions to hold and which to sacrifice
- Mastery of the physics has become a source of pride — the player can read incoming salvos and plan multi-intercept sequences with confidence
- The expanding-then-contracting campaign arc has delivered at least one moment of genuine strategic crisis where the player had to make a hard choice about what to give up
- The Cold War atmosphere has been sustained throughout — the tension never broke, the aesthetic never felt decorative or disconnected from the gameplay

### Long-Term (Post-Campaign)

- The player wants to try a different strategic approach in a new campaign
- The physics system has enough depth that the player is still discovering new tactics (intercept timing optimizations, chain reaction setups, terrain exploitation)
- The game has earned a permanent place in the player's rotation as a "I have 20 minutes, I want to think hard about something" option

---

## How Deterrence Should Feel

**The bunker fantasy.** You are an operator in a Cold War command center, staring at a radar screen, making decisions that determine whether millions of people live or die. The interface isn't friendly — it's functional. The aesthetic isn't beautiful — it's authentic. The green phosphor glow, the teletype chatter, the civil defense sirens — these aren't decorations, they're the texture of the world you're inhabiting. The game should feel like a declassified document brought to life.

**Controlled desperation.** Every wave should feel like you're barely holding on, but the "barely" should feel earned. You survived because you made good decisions, not because the game went easy on you. When you lose a city, it should feel like it was preventable — if only you'd invested in that radar upgrade, if only you'd placed that battery differently, if only you'd read that trajectory faster. The physics ensures that every outcome is explainable and every mistake is learnable.

**The weight of triage.** The emotional core of Deterrence is sacrifice. You cannot save everything. The game should never let you forget this. When three missiles are incoming and you only have time to intercept two, the choice of which city to abandon should feel heavy. Population numbers are not abstract scores — they represent the people under your protection. The game should make you care about losing them without being manipulative about it. Simple, restrained information design (a population counter ticking down, a city skyline going dark) is more powerful than dramatic cutscenes.

**Physics as a language.** After a few hours, the player should feel like they speak the language of trajectories. They should look at an incoming missile's arc and *know* — without calculating — that it's a high lob they have time for, or a fast shallow screamer they need to sprint-intercept now. This intuition is the game's deepest reward. It's the same satisfaction as understanding a complex system in Factorio or reading a board state in chess. The physics isn't an obstacle to fun — it *is* the fun.

**Quiet intensity.** Deterrence isn't loud or flashy. It doesn't celebrate kills with screen-shaking explosions or reward combos with juice effects. The CRT aesthetic constrains the visual language to something austere and procedural. Successful intercepts produce expanding rings on a radar screen. Failed intercepts produce a quiet impact marker and a population count decrement. The intensity comes from what's at stake and how little time you have to act, not from sensory overload. The game respects the player's attention and trusts that the stakes speak for themselves.

**Strategic regret as a feature.** The campaign should regularly produce moments where the player looks at their map and thinks: "I wish I'd expanded north instead of east" or "I should have invested in radar two waves ago." These aren't failures of game design — they're the heart of the strategic experience. Every decision should close some doors while opening others, and the player should be able to trace the consequences of their choices across many waves. This is what separates Deterrence from an arcade game: your past decisions shape your present crises.

---

*"You are not trying to win. You are trying to still be here tomorrow."*
