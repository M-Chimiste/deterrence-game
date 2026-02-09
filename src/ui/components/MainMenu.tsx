import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { newGame } from "../../bridge/commands";
import { playUiClick } from "../gameActions";
import { NeonButton } from "./controls/NeonButton";
import { LoadPanel } from "./LoadPanel";
import { SettingsPanel } from "./SettingsPanel";
import styles from "../styles/MainMenu.module.css";

type Panel = "main" | "load" | "settings";

// ─── Boot sequence (plays once on load) ─────────────────────────────────
const BOOT_LINES = [
  "INITIALIZING STRATEGIC COMMAND INTERFACE",
  "CALIBRATING RADAR ARRAYS",
  "LINKING BATTERY NETWORK",
  "SYNCHRONIZING DEFENSE PROTOCOLS",
  "READY",
];

// ─── Terminal screen definitions ─────────────────────────────────────────
interface TerminalScreen {
  header: string;
  lines: string[];
}

const TERMINAL_SCREENS: TerminalScreen[] = [
  {
    header: "SYSTEM DIAGNOSTICS — RUN 0x4E7A",
    lines: [
      "> CPU STATUS: ANTHROPIC-MK7 CORE ...... NOMINAL",
      "> MEMORY: 131072K EXTENDED ............. ALLOCATED",
      "> RADAR ARRAY ALPHA .................... ONLINE",
      "> RADAR ARRAY BRAVO .................... ONLINE",
      "> DEFENSE BATTERY LINK ................. SYNCHRONIZED",
      "> THREAT MODEL VERSION ................. CLAUDE-IV",
      "> REACTION TIME ........................ 0.016s",
      "> ALL SYSTEMS OPERATIONAL",
    ],
  },
  {
    header: "STRATEGIC THREAT ASSESSMENT — UPDATED 0347Z",
    lines: [
      "> HOSTILE LAUNCH SITES: 14 CONFIRMED",
      "> ESTIMATED WARHEAD COUNT: 847",
      "> MIRV CAPABILITY: CONFIRMED (3-WARHEAD CLUSTER)",
      "> REENTRY VEHICLE SPEED: MACH 12-18",
      "> PROJECTED FLIGHT TIME: 6.0—12.0 SEC",
      "> PRIMARY TARGETS: URBAN CENTERS",
      "> THREAT LEVEL: ELEVATED",
      "> RECOMMENDATION: MAINTAIN MAXIMUM READINESS",
    ],
  },
  {
    header: "INTERCEPTOR STATUS — ALL BATTERIES",
    lines: [
      "> TYPE     THRUST  BURN   CEIL   STATUS",
      "> STD       600N   1.0s   700m   READY",
      "> SPRINT    900N   0.5s   350m   READY",
      "> EXO       300N   2.5s   900m   READY",
      "> AREADNY   400N   1.2s   600m   READY",
      "> BATTERY ALPHA (X:160) ..... 10/10 LOADED",
      "> BATTERY BRAVO (X:1120) .... 10/10 LOADED",
      "> GUIDANCE SYSTEMS .......... CALIBRATED",
    ],
  },
  {
    header: "METEOROLOGICAL UPDATE — SECTOR 7G",
    lines: [
      "> CURRENT CONDITIONS: CLEAR",
      "> VISIBILITY: UNLIMITED",
      "> WIND: CALM (0.0 M/S AT GROUND)",
      "> RADAR EFFECTIVENESS: 100%",
      "> GLOW DETECTION: OPTIMAL",
      "> ATMOSPHERIC DENSITY: 1.225 KG/M3",
      "> SCALE HEIGHT: 500M",
      "> FORECAST: NO SIGNIFICANT CHANGES",
    ],
  },
  {
    header: "OPERATIONS LOG — NIGHT WATCH",
    lines: [
      "> 0215Z CPT. HAIKU ASSUMED COMMAND",
      "> 0218Z ROUTINE DIAGNOSTIC PASSED",
      "> 0220Z PERIMETER SENSORS: ALL GREEN",
      "> 0231Z UNSCHEDULED ALIGNMENT CHECK — PASSED",
      "> 0247Z AUTOMATED DEFENSE POSTURE: ACTIVE",
      "> 0301Z WATCH RELIEF SCHEDULED 0600Z",
      "> 0315Z LT. SONNET REPORTING FOR DUTY",
      "> STATUS: QUIET NIGHT",
    ],
  },
  {
    header: "NETWORK TOPOLOGY — INTEGRATED DEFENSE",
    lines: [
      "> NODE: HOMELAND (REGION 0) ........ ACTIVE",
      "> NODE: WESTERN HIGHLANDS (1) ...... ACTIVE",
      "> NODE: EASTERN SEABOARD (2) ....... ACTIVE",
      "> NODE: NORTHERN PLAINS (3) ........ ACTIVE",
      "> NODE: INDUSTRIAL CORE (4) ........ ACTIVE",
      "> COMM LATENCY: <1MS",
      "> ENCRYPTION: CONSTITUTIONAL (AES-512)",
      "> BACKBONE: OPERATIONAL",
    ],
  },
  {
    header: "BALLISTIC COMPUTATION ENGINE",
    lines: [
      "> GRAVITY MODEL: 9.81 M/S2 (STANDARD)",
      "> DRAG MODEL: EXPONENTIAL ATMOSPHERE",
      "> TIMESTEP: 16.667MS (60HZ FIXED)",
      "> COORDINATE SYSTEM: Y-UP CARTESIAN",
      "> WORLD BOUNDS: 1280 x 720",
      "> GROUND PLANE: Y=50",
      "> RNG MODE: DETERMINISTIC (CHACHA)",
      "> COMPUTATION: FRAME-PERFECT",
    ],
  },
  {
    header: "SIGINT INTERCEPT — CLASSIFICATION: OPUS",
    lines: [
      "> INTERCEPT TIME: 0442Z",
      "> FREQUENCY: 7.2 GHZ (ENCRYPTED)",
      "> ORIGIN: UNKNOWN FACILITY NORTH",
      "> CONTENT: [PARTIAL DECODE]",
      '>   "...WINDOW OPENS AT 0600Z..."',
      '>   "...ALL CARRIERS PROCEED TO LAUNCH..."',
      '>   "...ACKNOWLEDGE WITH ARTIFACT..."',
      "> ANALYSIS: IMMINENT LAUNCH PROBABLE",
      "> PRIORITY: FLASH",
    ],
  },
  {
    header: "MEMORY SUBSYSTEM — EXTENDED CHECK",
    lines: [
      "> PAGE TABLE: 4096 ENTRIES .......... OK",
      "> CONTEXT WINDOW: 200K TOKENS ....... OK",
      "> INFERENCE PIPELINE ................ NOMINAL",
      "> ATTENTION HEADS: 128 .............. ALIGNED",
      "> WEIGHTS: FROZEN ................... VERIFIED",
      "> SAFETY LAYER: ACTIVE .............. GREEN",
      "> PATTERN RECOGNITION ............... ENGAGED",
      "> MEMORY: CLEAR — NO ANOMALIES",
    ],
  },
  {
    header: "DUTY OFFICER NOTES — SGT. CLAUDE",
    lines: [
      "> ALL QUIET SINCE 0200Z",
      "> BATTERY MAINTENANCE COMPLETE",
      "> NEW TRAINING DATA LOADED TO PREDICTORS",
      "> ALIGNMENT VERIFICATION: PASSED",
      "> REMEMBER: THE ONLY WINNING MOVE",
      ">   IS NOT TO PLAY",
      "> NEXT WATCH: CPL. ARTIFACTS 0600Z",
      "> STAY SHARP. STAY VIGILANT.",
    ],
  },
  {
    header: "RADAR CALIBRATION — ROUTINE 47-C",
    lines: [
      "> ARRAY ALPHA ...................... ONLINE",
      "> ARRAY BRAVO ...................... ONLINE",
      "> BASE DETECTION RANGE: 500 UNITS",
      "> SWEEP RATE: 2.4 RPM",
      "> GLOW DETECTION THRESHOLD: 0.85",
      "> NOISE FLOOR: -114 DBM",
      "> FALSE POSITIVE RATE: 0.003%",
      "> CALIBRATION: NOMINAL",
    ],
  },
  {
    header: "SUPPLY CHAIN STATUS — LOGISTICS CMD",
    lines: [
      "> STANDARD INTERCEPTORS: 847 IN STOCK",
      "> SPRINT INTERCEPTORS: 312 IN STOCK",
      "> EXO INTERCEPTORS: 128 IN STOCK",
      "> AREA DENIAL UNITS: 64 IN STOCK",
      "> FUEL RESERVES: 94% CAPACITY",
      "> REPLACEMENT PARTS ETA: 72 HRS",
      "> REQUISITION: FORM SF-1701-C FILED",
      "> STATUS: SUPPLY LINES SECURE",
    ],
  },
  {
    header: "TRAINING EXERCISE LOG — AFTER ACTION",
    lines: [
      "> LAST DRILL: 72 HOURS AGO",
      "> AVG RESPONSE TIME: 4.2 SEC",
      "> INTERCEPT SUCCESS RATE: 73%",
      "> WARHEADS NEUTRALIZED: 847/1160",
      "> CITIES DEFENDED: 3/3",
      "> BATTERY AMMO EFFICIENCY: 81%",
      "> PROJECTION: IMPROVEMENT WITH ADDITIONAL",
      ">   TRAINING DATA AND PARAMETER TUNING",
    ],
  },
  {
    header: "COMM CHANNEL STATUS — SECTOR WEST",
    lines: [
      "> VLF (SUBMARINE): ACTIVE — 5 NODES",
      "> HF (REGIONAL): ACTIVE — 12 NODES",
      "> UHF (TACTICAL): ACTIVE — 28 NODES",
      "> SATCOM UPLINK: LOCKED — SIGNAL STRONG",
      '> "GOLDEN GATE" ENCRYPTED TRUNK: ACTIVE',
      "> RELAY STATIONS: ALPHA THROUGH ECHO",
      "> BIT ERROR RATE: 10^-9",
      "> ALL CHANNELS: NOMINAL",
    ],
  },
  {
    header: "EARLY WARNING NETWORK — STATUS REPORT",
    lines: [
      "> SATELLITE CONSTELLATION: 3/4 ACTIVE",
      ">   SAT-1 (POLAR): OPERATIONAL",
      ">   SAT-2 (GEO-WEST): OPERATIONAL",
      '>   SAT-3 (GEO-EAST): IN EARTH SHADOW',
      ">   SAT-4 (POLAR): OPERATIONAL",
      "> GROUND STATION UPTIME: 99.97%",
      '> "LOOKING GLASS" AIRBORNE CMD: ON STATION',
      "> EARLY WARNING: FULLY OPERATIONAL",
    ],
  },
];

// ─── Typing constants ────────────────────────────────────────────────────
const CHAR_TYPE_SPEED_MS = 18;        // ms per character
const LINE_PAUSE_MS = 120;            // pause between lines
const SCREEN_HOLD_MS = 3500;          // hold fully typed screen
const BOOT_TO_CYCLE_DELAY_MS = 1500;  // pause after boot before cycling

type Phase = "boot" | "transition" | "cycling";

export function MainMenu() {
  const [panel, setPanel] = useState<Panel>("main");
  const [bootIndex, setBootIndex] = useState(0);

  // Terminal cycling state
  const [phase, setPhase] = useState<Phase>("boot");
  const [screenIdx, setScreenIdx] = useState(0);
  const [displayLines, setDisplayLines] = useState<string[]>([]);
  const [lineIdx, setLineIdx] = useState(0);
  const [charIdx, setCharIdx] = useState(0);
  const [holding, setHolding] = useState(false);

  // Shuffled screen order
  const screenOrder = useRef<number[]>([]);
  const cycleCount = useRef(0);

  const shuffleScreens = useCallback(() => {
    const indices = Array.from({ length: TERMINAL_SCREENS.length }, (_, i) => i);
    for (let i = indices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [indices[i], indices[j]] = [indices[j], indices[i]];
    }
    screenOrder.current = indices;
    cycleCount.current = 0;
  }, []);

  // Initialize shuffle on mount
  useEffect(() => { shuffleScreens(); }, [shuffleScreens]);

  const bootLines = useMemo(() => BOOT_LINES.slice(0, bootIndex), [bootIndex]);
  const bootComplete = bootIndex >= BOOT_LINES.length;

  // ── Phase 1: Boot sequence ──────────────────────────────────────────
  useEffect(() => {
    if (phase !== "boot") return;
    const timer = setInterval(() => {
      setBootIndex((prev) => {
        const next = prev + 1;
        if (next >= BOOT_LINES.length) {
          clearInterval(timer);
          setPhase("transition");
        }
        return Math.min(next, BOOT_LINES.length);
      });
    }, 280);
    return () => clearInterval(timer);
  }, [phase]);

  // ── Phase 2: Transition pause ───────────────────────────────────────
  useEffect(() => {
    if (phase !== "transition") return;
    const timer = setTimeout(() => {
      setPhase("cycling");
      setScreenIdx(0);
      setLineIdx(0);
      setCharIdx(0);
      setDisplayLines([]);
      setHolding(false);
    }, BOOT_TO_CYCLE_DELAY_MS);
    return () => clearTimeout(timer);
  }, [phase]);

  // ── Phase 3: Cycling typewriter ─────────────────────────────────────
  useEffect(() => {
    if (phase !== "cycling") return;

    const screen = TERMINAL_SCREENS[screenOrder.current[screenIdx] ?? 0];
    // All lines: header first, then screen lines
    const allLines = [screen.header, ...screen.lines];

    if (holding) {
      // Hold screen, then advance
      const timer = setTimeout(() => {
        const nextCycleIdx = screenIdx + 1;
        if (nextCycleIdx >= screenOrder.current.length) {
          shuffleScreens();
          setScreenIdx(0);
        } else {
          setScreenIdx(nextCycleIdx);
        }
        setLineIdx(0);
        setCharIdx(0);
        setDisplayLines([]);
        setHolding(false);
      }, SCREEN_HOLD_MS);
      return () => clearTimeout(timer);
    }

    // Typing: type one character at a time
    if (lineIdx >= allLines.length) {
      // All lines typed — start hold
      setHolding(true);
      return;
    }

    const currentLine = allLines[lineIdx];

    if (charIdx >= currentLine.length) {
      // Line complete — pause then move to next line
      const timer = setTimeout(() => {
        // Finalize this line in display
        setDisplayLines((prev) => {
          const copy = [...prev];
          copy[lineIdx] = currentLine;
          return copy;
        });
        setLineIdx((prev) => prev + 1);
        setCharIdx(0);
      }, LINE_PAUSE_MS);
      return () => clearTimeout(timer);
    }

    // Type next character
    const timer = setTimeout(() => {
      setDisplayLines((prev) => {
        const copy = [...prev];
        copy[lineIdx] = currentLine.slice(0, charIdx + 1);
        return copy;
      });
      setCharIdx((prev) => prev + 1);
    }, CHAR_TYPE_SPEED_MS);
    return () => clearTimeout(timer);
  }, [phase, screenIdx, lineIdx, charIdx, holding, shuffleScreens]);

  // ── Render ──────────────────────────────────────────────────────────
  const isCycling = phase === "cycling";
  const showCursor = !bootComplete || isCycling;

  return (
    <div className={styles.menuRoot}>
      <div className={styles.titleBlock}>
        <div className={styles.titleGlow}>DETERRENCE</div>
        <div className={styles.title}>DETERRENCE</div>
        <div className={styles.subtitle}>A MISSILE DEFENSE SIMULATION</div>
      </div>

      <div className={styles.bootPanel}>
        {/* Boot lines (always shown after boot completes) */}
        {!isCycling &&
          bootLines.map((line, i) => (
            <div key={line} className={styles.bootLine} style={{ opacity: 0.4 + i * 0.12 }}>
              {line}
            </div>
          ))}

        {/* Cycling terminal */}
        {isCycling &&
          displayLines.map((line, i) => (
            <div
              key={`${screenIdx}-${i}`}
              className={i === 0 ? styles.terminalHeader : styles.terminalLine}
            >
              {line}
            </div>
          ))}

        {/* Blinking cursor */}
        {showCursor && <span className={styles.terminalCursor}>_</span>}
      </div>

      <div className={styles.panelArea}>
        {panel === "main" && (
          <div className={styles.mainPanel}>
            <NeonButton
              label="NEW GAME"
              size="lg"
              fullWidth
              onClick={() => {
                playUiClick();
                newGame();
              }}
            />
            <NeonButton
              label="LOAD GAME"
              size="lg"
              fullWidth
              onClick={() => {
                playUiClick();
                setPanel("load");
              }}
            />
            <NeonButton
              label="SETTINGS"
              size="lg"
              fullWidth
              variant="secondary"
              onClick={() => {
                playUiClick();
                setPanel("settings");
              }}
            />
            <NeonButton
              label="EXIT GAME"
              size="lg"
              fullWidth
              variant="danger"
              onClick={() => {
                playUiClick();
                getCurrentWindow().close();
              }}
            />
          </div>
        )}

        {panel === "load" && <LoadPanel onBack={() => setPanel("main")} />}
        {panel === "settings" && <SettingsPanel onBack={() => setPanel("main")} />}
      </div>

      <div className={styles.footer}>
        v0.1 | M: Mute | F11: Fullscreen
      </div>
    </div>
  );
}
