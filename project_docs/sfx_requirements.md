# SFX Requirements — DETERRENCE

This document lists all sound effects needed for full audio polish of the DETERRENCE IAMD simulation game. Each sound includes a description, suggested parameters, duration, and priority tier.

## Format Requirements

- **Format:** WAV, 44.1 kHz, 16-bit, mono
- **Naming:** `kebab-case.wav` (e.g., `new-contact-ping.wav`)
- **Location:** `public/sfx/` directory
- **Peak level:** -3 dBFS (leave headroom for mixing)

## Priority Tiers

- **P0 — Gameplay-critical:** Required for minimum viable gameplay feedback
- **P1 — Atmosphere:** Significantly enhances immersion and tactical awareness
- **P2 — Nice-to-have:** Polish layer for full production quality

---

## P0 — Gameplay-Critical SFX

These sounds provide essential feedback for player decision-making.

### 1. New Contact Ping
- **File:** `new-contact-ping.wav`
- **Trigger:** `AudioEvent::NewContact`
- **Character:** Short sonar-style ping. Rising tone, crisp and attention-getting.
- **Duration:** 200-300ms
- **Suggested:** Sine sweep 800-1200 Hz with sharp attack and exponential decay.

### 2. Veto Clock Start
- **File:** `veto-clock-start.wav`
- **Trigger:** `AudioEvent::VetoClockStart`
- **Character:** Two-tone attention beep. Clear, professional military alert.
- **Duration:** 300ms (two 100ms pulses with 50ms gap)
- **Suggested:** Dual 880 Hz sine pulses with clean gating.

### 3. Veto Clock Warning
- **File:** `veto-clock-warning.wav`
- **Trigger:** `AudioEvent::VetoClockWarning`
- **Character:** Urgent beeping, faster than start tone. Higher pitch for final warning.
- **Duration:** 200-400ms
- **Suggested:** 1000-1200 Hz square wave, 2-3 rapid pulses. Two variants: 3-second warning and 1-second warning (more urgent).

### 4. Bird Away (Missile Launch)
- **File:** `bird-away.wav`
- **Trigger:** `AudioEvent::BirdAway`
- **Character:** Rising sweep suggesting acceleration. Mechanical-sounding, not explosive.
- **Duration:** 400-600ms
- **Suggested:** Sawtooth sweep 200-2000 Hz with attack envelope.

### 5. Splash — Hit
- **File:** `splash-hit.wav`
- **Trigger:** `AudioEvent::Splash` with `result: Hit`
- **Character:** Sharp noise burst. Brief, decisive, satisfying.
- **Duration:** 50-100ms
- **Suggested:** Band-passed white noise burst (2 kHz center, Q=1.5).

### 6. Splash — Miss
- **File:** `splash-miss.wav`
- **Trigger:** `AudioEvent::Splash` with `result: Miss`
- **Character:** Low descending tone. Deflating, ominous.
- **Duration:** 250-400ms
- **Suggested:** 200 Hz sine with exponential fade-out.

### 7. Vampire Impact
- **File:** `vampire-impact.wav`
- **Trigger:** `AudioEvent::VampireImpact`
- **Character:** Alarm klaxon. Alternating tones, unmistakably bad.
- **Duration:** 800ms-1.2s
- **Suggested:** Alternating 440/880 Hz square wave, 6 alternations over 1 second.

### 8. Contact Lost
- **File:** `contact-lost.wav`
- **Trigger:** `AudioEvent::ContactLost`
- **Character:** Descending tone. Track faded from scope.
- **Duration:** 300-400ms
- **Suggested:** Sine sweep 1000-500 Hz with decay envelope.

---

## P1 — Atmosphere SFX

These sounds enhance immersion and spatial awareness.

### 9. CIC Ambient Hum
- **File:** `cic-ambient-loop.wav`
- **Trigger:** Background loop during Active/Paused phases
- **Character:** Low electronics hum with subtle equipment noise. Constant, unobtrusive.
- **Duration:** 10-15s seamless loop
- **Suggested:** 60 Hz filtered hum + subtle random equipment clicks and whirrs.

### 10. Radar Sweep Tick
- **File:** `radar-sweep-tick.wav`
- **Trigger:** Each PPI sweep pass (every ~4 seconds)
- **Character:** Soft tick or click. Rhythmic timing cue.
- **Duration:** 30-50ms
- **Suggested:** Very short noise burst with high-pass filter. Barely audible.

### 11. Track Promoted (Firm)
- **File:** `track-promoted.wav`
- **Trigger:** Track quality reaches "firm" threshold (0.6)
- **Character:** Subtle confirmation tone. Track is now engagement-eligible.
- **Duration:** 150-200ms
- **Suggested:** Two ascending tones (600-800 Hz), quick.

### 12. Classification Change
- **File:** `classification-change.wav`
- **Trigger:** `AudioEvent::ThreatEvaluated`
- **Character:** Brief data-processing tone. Neutral, informational.
- **Duration:** 100-150ms
- **Suggested:** Digital chirp, 1200 Hz sine with brief 50ms ramp.

### 13. Engagement Phase Transition
- **File:** `engagement-phase.wav`
- **Trigger:** Engagement transitions (Midcourse, Terminal)
- **Character:** Subtle status change indicator.
- **Duration:** 100-200ms
- **Suggested:** Soft double-click or quiet beep.

### 14. Illuminator Assigned
- **File:** `illuminator-assigned.wav`
- **Trigger:** Illuminator channel locks onto target
- **Character:** Targeting lock-on tone. Steady, focused.
- **Duration:** 300-500ms
- **Suggested:** 1 kHz sine sustained with slight warble.

### 15. Threat Pop-Up Detected
- **File:** `threat-popup.wav`
- **Trigger:** Sea-skimmer begins pop-up maneuver (detected via altitude change)
- **Character:** Attention-grabbing rising tone. Something changed.
- **Duration:** 200-300ms
- **Suggested:** Rapid ascending chirp 600-1500 Hz.

---

## P2 — Nice-to-Have SFX

Polish sounds for full production quality.

### 16. UI Button Click
- **File:** `ui-click.wav`
- **Trigger:** Any button press in UI
- **Character:** Subtle tactical console click.
- **Duration:** 20-40ms
- **Suggested:** Very short filtered noise impulse.

### 17. UI Button Hover
- **File:** `ui-hover.wav`
- **Trigger:** Mouse hover over interactive elements
- **Character:** Nearly silent tick. Feedback without distraction.
- **Duration:** 10-20ms
- **Suggested:** Extremely brief high-frequency click.

### 18. Mission Start
- **File:** `mission-start.wav`
- **Trigger:** Scenario selected, mission begins
- **Character:** General quarters alarm or action stations tone.
- **Duration:** 1-2s
- **Suggested:** Rising klaxon with authority, then silence.

### 19. Mission Complete — Success
- **File:** `mission-complete-success.wav`
- **Trigger:** Mission ends with assets protected
- **Character:** Resolved, professional completion tone.
- **Duration:** 1-2s
- **Suggested:** Ascending three-note chime.

### 20. Mission Complete — Failure
- **File:** `mission-complete-failure.wav`
- **Trigger:** Mission ends with assets damaged
- **Character:** Somber, weighty failure tone.
- **Duration:** 1.5-2.5s
- **Suggested:** Low descending tone with reverb tail.

### 21. Saturation Warning
- **File:** `saturation-warning.wav`
- **Trigger:** More threats than available illuminator channels
- **Character:** System stress indicator. Distinct from other warnings.
- **Duration:** 500-700ms
- **Suggested:** Warbling 600 Hz tone, pulsing 4x/sec.

### 22. VLS Magazine Low
- **File:** `vls-low.wav`
- **Trigger:** VLS ready count drops below 25%
- **Character:** Resource depletion warning.
- **Duration:** 400-500ms
- **Suggested:** Slow descending two-tone, 500-300 Hz.

---

## Voice Callout Suggestions (Future)

These would be pre-recorded or TTS-generated voice lines for maximum authenticity.

| Callout | Text | Trigger |
|---|---|---|
| New contact | "NEW CONTACT BEARING [NNN]" | NewContact event |
| Hostile ID | "HOSTILE, TRACK [NNN]" | Classification → Hostile |
| Bird away | "BIRD AWAY" | BirdAway event |
| Splash one | "SPLASH ONE" / "SPLASH TWO" etc. | Splash Hit |
| Vampire | "VAMPIRE VAMPIRE VAMPIRE, BEARING [NNN]" | VampireImpact |
| Weapons tight | "WEAPONS TIGHT" | Doctrine → Manual |
| Weapons free | "WEAPONS FREE" | Doctrine → AutoComposite |
| General quarters | "GENERAL QUARTERS, ALL HANDS MAN YOUR BATTLE STATIONS" | Mission start |

---

## Tension System (Future Enhancement)

A layered ambient system that scales with threat intensity:

| Layer | Condition | Character |
|---|---|---|
| Base | Always (during Active) | CIC ambient hum |
| Layer 1 | 1-3 active tracks | Subtle low-frequency pulse |
| Layer 2 | 4-6 active tracks | Mid-frequency tension drone |
| Layer 3 | 7+ active tracks or any active engagements | High-frequency tension, faster pulse |
| Layer 4 | Saturation (more threats than illuminators) | Full tension, warbling alarm undertone |

Each layer crossfades in/out as threat count changes. Combined with the level-specific music tracks, this creates dynamic audio that reflects tactical pressure.

---

## Implementation Notes

1. **Current state:** All P0 sounds are currently implemented as Web Audio API procedural sounds (oscillators/noise) in `SfxEngine.ts`. Replacing with WAV files requires updating each method to use `Howl` playback instead of `AudioContext` oscillators.

2. **Howler.js integration:** Import each sound as a `Howl` instance, preload on init, trigger `.play()` from `consumeAudioEvents()`. The existing `SfxEngine` class structure maps 1:1 to method-per-sound-file.

3. **Volume mixing:** Consider separate volume controls for music, SFX, and voice (future). The current implementation has `MusicManager.setVolume()` and `SfxEngine.setVolume()` as independent controls.

4. **Spatial audio (future):** Some sounds (NewContact, VampireImpact) include bearing data. This could drive stereo panning or Web Audio API `PannerNode` for spatial positioning relative to the PPI view.
