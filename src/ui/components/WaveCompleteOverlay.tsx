import { useMemo } from "react";
import { continueToStrategic } from "../../bridge/commands";
import { useGameStore } from "../store";
import { playUiClick } from "../gameActions";
import styles from "../styles/HUD.module.css";

export function WaveCompleteOverlay() {
  const event = useGameStore((state) => state.waveComplete);

  const lines = useMemo(() => {
    if (!event) return [];
    const totalMissiles = event.missiles_destroyed + event.missiles_impacted;
    const efficiency =
      event.interceptors_launched > 0
        ? Math.round((event.missiles_destroyed / event.interceptors_launched) * 100)
        : 0;
    return [
      `WAVE ${event.wave_number} COMPLETE`,
      "",
      `Missiles Destroyed: ${event.missiles_destroyed}/${totalMissiles}`,
      `Missiles Impacted:  ${event.missiles_impacted}`,
      `Interceptors Used:  ${event.interceptors_launched}`,
      `Efficiency:         ${efficiency}%`,
      `Cities Remaining:   ${event.cities_remaining}`,
      "",
      "Press ENTER or Click to Continue",
    ];
  }, [event]);

  if (!event) return null;

  return (
    <div
      className={styles.waveComplete}
      onClick={() => {
        playUiClick();
        continueToStrategic();
      }}
    >
      <div className={styles.wavePanel}>
        {lines.map((line, index) => (
          <div key={`${index}-${line}`} className={styles.waveLine}>
            {line}
          </div>
        ))}
      </div>
    </div>
  );
}
