import { useGameStore } from "./store";
import { MainMenu } from "./components/MainMenu";
import { HUDOverlay } from "./components/HUDOverlay";
import { StrategicOverlay } from "./components/StrategicOverlay";
import { WaveCompleteOverlay } from "./components/WaveCompleteOverlay";
import styles from "./styles/App.module.css";

export function App() {
  const phase = useGameStore((state) => state.phase);
  const screenShake = useGameStore((state) => state.screenShake);

  return (
    <div
      className={styles.uiRoot}
      style={{
        transform: `translate(${screenShake.x}px, ${screenShake.y}px)`,
      }}
    >
      {phase === "MainMenu" && <MainMenu />}
      {phase === "Strategic" && <StrategicOverlay />}
      {(phase === "WaveActive" || phase === "WaveResult") && <HUDOverlay />}
      {phase === "WaveResult" && <WaveCompleteOverlay />}
    </div>
  );
}
