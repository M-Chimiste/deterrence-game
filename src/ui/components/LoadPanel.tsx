import { useEffect, useState } from "react";
import { listSaves, loadGame } from "../../bridge/commands";
import type { SaveMetadata } from "../../types/commands";
import { playUiClick } from "../gameActions";
import { NeonButton } from "./controls/NeonButton";
import styles from "../styles/MainMenu.module.css";

interface LoadPanelProps {
  onBack: () => void;
}

export function LoadPanel({ onBack }: LoadPanelProps) {
  const [loading, setLoading] = useState(true);
  const [saves, setSaves] = useState<SaveMetadata[]>([]);

  useEffect(() => {
    let mounted = true;
    setLoading(true);
    listSaves()
      .then((data) => {
        if (!mounted) return;
        setSaves(data.sort((a, b) => b.timestamp - a.timestamp));
      })
      .catch(() => {
        if (!mounted) return;
        setSaves([]);
      })
      .finally(() => {
        if (mounted) setLoading(false);
      });
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <div className={styles.panel}>
      <div className={styles.panelHeader}>LOAD GAME</div>
      <div className={styles.panelBody}>
        {loading && <div className={styles.panelMuted}>Loading saves...</div>}
        {!loading && saves.length === 0 && (
          <div className={styles.panelWarning}>No saved games found.</div>
        )}
        {!loading &&
          saves.map((save) => {
            const date = new Date(save.timestamp * 1000).toLocaleString();
            const label = `${save.slot_name.toUpperCase()}  |  WAVE ${
              save.wave_number
            }  |  $${save.resources}  |  ${date}`;
            return (
              <NeonButton
                key={save.slot_name}
                label={label}
                size="sm"
                fullWidth
                onClick={() => {
                  playUiClick();
                  loadGame(save.slot_name);
                }}
              />
            );
          })}
      </div>
      <div className={styles.panelFooter}>
        <NeonButton
          label="BACK"
          size="md"
          variant="secondary"
          onClick={() => {
            playUiClick();
            onBack();
          }}
        />
      </div>
    </div>
  );
}
