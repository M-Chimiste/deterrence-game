/**
 * Debug overlay showing simulation telemetry.
 *
 * Displays: tick counter, entity count, snapshot receive rate,
 * game phase, and basic simulation controls.
 */

import { useEffect, useRef } from "preact/hooks";
import { useGameStore } from "../store/gameState";
import {
  pauseSimulation,
  resumeSimulation,
  setTimeScale,
  startMission,
} from "../ipc/bridge";

export function DebugOverlay() {
  const snapshot = useGameStore((s) => s.snapshot);
  const connected = useGameStore((s) => s.connected);
  const snapshotRate = useGameStore((s) => s.snapshotRate);
  const fpsRef = useRef(0);
  const frameCountRef = useRef(0);
  const lastFpsUpdate = useRef(performance.now());

  // Measure rendering FPS via requestAnimationFrame
  useEffect(() => {
    let animFrame: number;
    const measure = () => {
      frameCountRef.current += 1;
      const now = performance.now();
      if (now - lastFpsUpdate.current >= 1000) {
        fpsRef.current = frameCountRef.current;
        frameCountRef.current = 0;
        lastFpsUpdate.current = now;
      }
      animFrame = requestAnimationFrame(measure);
    };
    animFrame = requestAnimationFrame(measure);
    return () => cancelAnimationFrame(animFrame);
  }, []);

  if (!connected) return null;

  const phase = snapshot?.phase ?? "Unknown";
  const tick = snapshot?.time.tick ?? 0;
  const elapsed = snapshot?.time.elapsed_secs ?? 0;
  const trackCount = snapshot?.tracks.length ?? 0;

  return (
    <div
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        padding: "12px 16px",
        fontFamily: "'JetBrains Mono', 'Courier New', monospace",
        fontSize: "11px",
        color: "#00ff41",
        background: "rgba(0, 0, 0, 0.85)",
        borderRight: "1px solid #00ff4133",
        borderBottom: "1px solid #00ff4133",
        zIndex: 9999,
        minWidth: "220px",
        lineHeight: "1.6",
      }}
    >
      <div style={{ opacity: 0.5, marginBottom: "8px", letterSpacing: "2px" }}>
        DEBUG
      </div>
      <div>PHASE: {phase}</div>
      <div>
        TICK: {tick} ({elapsed.toFixed(1)}s)
      </div>
      <div>TRACKS: {trackCount}</div>
      <div>SNAP RATE: {snapshotRate.toFixed(1)} Hz</div>
      <div>RENDER FPS: {fpsRef.current}</div>

      <div
        style={{
          marginTop: "12px",
          display: "flex",
          flexDirection: "column",
          gap: "4px",
        }}
      >
        {phase === "MainMenu" && (
          <DebugButton label="START MISSION" onClick={startMission} />
        )}
        {phase === "Active" && (
          <DebugButton label="PAUSE" onClick={pauseSimulation} />
        )}
        {phase === "Paused" && (
          <DebugButton label="RESUME" onClick={resumeSimulation} />
        )}

        <div style={{ marginTop: "4px", display: "flex", gap: "4px" }}>
          {[0.5, 1, 2, 4].map((scale) => (
            <DebugButton
              key={scale}
              label={`${scale}x`}
              onClick={() => setTimeScale(scale)}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

function DebugButton({
  label,
  onClick,
}: {
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        background: "transparent",
        border: "1px solid #00ff4166",
        color: "#00ff41",
        padding: "2px 8px",
        fontFamily: "'JetBrains Mono', 'Courier New', monospace",
        fontSize: "10px",
        cursor: "pointer",
        letterSpacing: "1px",
      }}
    >
      {label}
    </button>
  );
}
