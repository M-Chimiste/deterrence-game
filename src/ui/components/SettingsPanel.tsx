import { useMemo } from "react";
import { setFullscreen, setWindowResolution } from "../../bridge/commands";
import { useGameStore } from "../store";
import { playUiClick, setMuted, setVolume } from "../gameActions";
import { NeonButton } from "./controls/NeonButton";
import { NeonToggle } from "./controls/NeonToggle";
import { NeonSlider } from "./controls/NeonSlider";
import styles from "../styles/MainMenu.module.css";

interface SettingsPanelProps {
  onBack: () => void;
}

const RESOLUTION_PRESETS: Record<string, { width: number; height: number }> = {
  "720p": { width: 1280, height: 720 },
  "1080p": { width: 1920, height: 1080 },
  "1440p": { width: 2560, height: 1440 },
  "4K": { width: 3840, height: 2160 },
};

export function SettingsPanel({ onBack }: SettingsPanelProps) {
  const settings = useGameStore((state) => state.settings);
  const updateSettings = useGameStore((state) => state.updateSettings);
  const setHud = useGameStore((state) => state.setHud);

  const resolutions = useMemo(() => Object.keys(RESOLUTION_PRESETS), []);

  const applyResolution = async (preset: string) => {
    updateSettings({ resolution: preset });
    const res = RESOLUTION_PRESETS[preset];
    if (!res) return;
    try {
      await setWindowResolution(res.width, res.height);
    } catch {
      // Ignore resolution errors
    }
  };

  const applyFullscreen = async (enabled: boolean) => {
    updateSettings({ fullscreen: enabled });
    try {
      await setFullscreen(enabled);
    } catch {
      // Ignore fullscreen errors
    }
  };

  return (
    <div className={styles.panel}>
      <div className={styles.panelHeader}>SETTINGS</div>
      <div className={styles.panelBody}>
        <div className={styles.sectionLabel}>DISPLAY</div>
        <div className={styles.sectionRow}>
          <NeonToggle
            label="Fullscreen"
            checked={settings.fullscreen}
            onChange={(checked) => {
              playUiClick();
              applyFullscreen(checked);
            }}
          />
        </div>

        <div className={styles.sectionLabel}>RESOLUTION</div>
        <div className={styles.resolutionRow}>
          {resolutions.map((preset) => (
            <NeonButton
              key={preset}
              label={preset}
              size="sm"
              variant={settings.resolution === preset ? "primary" : "secondary"}
              onClick={() => {
                playUiClick();
                applyResolution(preset);
              }}
            />
          ))}
        </div>

        <div className={styles.sectionLabel}>AUDIO</div>
        <div className={styles.sectionRow}>
          <NeonToggle
            label="Sound"
            checked={settings.audioEnabled}
            onChange={(checked) => {
              playUiClick();
              updateSettings({ audioEnabled: checked });
              setMuted(!checked);
              setHud({ muted: !checked });
            }}
          />
          <NeonSlider
            label="Volume"
            value={settings.volume}
            onChange={(value) => {
              updateSettings({ volume: value });
              setVolume(value);
            }}
          />
        </div>
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
