import { useEffect, useMemo, useState } from "react";
import { newGame } from "../../bridge/commands";
import { playUiClick } from "../gameActions";
import { NeonButton } from "./controls/NeonButton";
import { LoadPanel } from "./LoadPanel";
import { SettingsPanel } from "./SettingsPanel";
import styles from "../styles/MainMenu.module.css";

type Panel = "main" | "load" | "settings";

const BOOT_LINES = [
  "INITIALIZING STRATEGIC COMMAND INTERFACE",
  "CALIBRATING RADAR ARRAYS",
  "LINKING BATTERY NETWORK",
  "SYNCHRONIZING DEFENSE PROTOCOLS",
  "READY",
];

export function MainMenu() {
  const [panel, setPanel] = useState<Panel>("main");
  const [bootIndex, setBootIndex] = useState(0);

  useEffect(() => {
    const timer = setInterval(() => {
      setBootIndex((prev) => Math.min(prev + 1, BOOT_LINES.length));
    }, 280);
    return () => clearInterval(timer);
  }, []);

  const bootLines = useMemo(() => BOOT_LINES.slice(0, bootIndex), [bootIndex]);
  const bootComplete = bootIndex >= BOOT_LINES.length;

  return (
    <div className={styles.menuRoot}>
      <div className={styles.titleBlock}>
        <div className={styles.titleGlow}>DETERRENCE</div>
        <div className={styles.title}>DETERRENCE</div>
        <div className={styles.subtitle}>A MISSILE DEFENSE SIMULATION</div>
      </div>

      <div className={styles.bootPanel}>
        {bootLines.map((line, i) => (
          <div key={line} className={styles.bootLine} style={{ opacity: 0.4 + i * 0.12 }}>
            {line}
          </div>
        ))}
        {!bootComplete && <div className={styles.bootCursor}>|</div>}
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
