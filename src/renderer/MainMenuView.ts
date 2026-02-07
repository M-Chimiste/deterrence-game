import { Application, Container, Graphics, Text, TextStyle } from "pixi.js";
import { listSaves, setWindowResolution, setFullscreen } from "../bridge/commands";
import type { SaveMetadata } from "../types/commands";
import { NeonButton } from "./ui/NeonButton";
import { NeonToggle } from "./ui/NeonToggle";
import { NeonSlider } from "./ui/NeonSlider";
import {
  NEON_CYAN, ELECTRIC_BLUE, NEON_GREEN, SOLAR_YELLOW,
  DIM_TEXT, FONT_FAMILY,
} from "./ui/Theme";

const WORLD_WIDTH = 1280;
const WORLD_HEIGHT = 720;

type MenuPanel = "main" | "load" | "settings";

export interface GameSettings {
  crtEnabled: boolean;
  audioEnabled: boolean;
  volume: number;
  resolution: string;
  fullscreen: boolean;
}

const RESOLUTION_PRESETS: Record<string, { width: number; height: number }> = {
  "720p": { width: 1280, height: 720 },
  "1080p": { width: 1920, height: 1080 },
  "1440p": { width: 2560, height: 1440 },
  "4K": { width: 3840, height: 2160 },
};

const DEFAULT_SETTINGS: GameSettings = {
  crtEnabled: true,
  audioEnabled: true,
  volume: 0.5,
  resolution: "720p",
  fullscreen: false,
};

export class MainMenuView {
  private container: Container;
  private mainPanel: Container;
  private loadPanel: Container;
  private settingsPanel: Container;
  private versionText: Text;

  // UI components that need tick updates
  private tickables: { tick: (dt: number) => void }[] = [];

  // Settings state
  private settings: GameSettings;
  private crtToggle: NeonToggle | null = null;
  private fullscreenToggle: NeonToggle | null = null;
  private audioToggle: NeonToggle | null = null;
  private volumeSlider: NeonSlider | null = null;
  private resolutionButtons: Map<string, NeonButton> = new Map();

  /** Callbacks */
  onNewGame: (() => void) | null = null;
  onLoadGame: ((slotName: string) => void) | null = null;
  onCRTToggle: (() => void) | null = null;
  onMuteToggle: (() => void) | null = null;
  onVolumeChange: ((volume: number) => void) | null = null;

  constructor(app: Application) {
    this.settings = MainMenuView.loadSettings();

    this.container = new Container();
    this.container.visible = false;
    app.stage.addChild(this.container);

    // Semi-transparent dark overlay behind text for readability
    const overlay = new Graphics();
    overlay.rect(0, 0, WORLD_WIDTH, WORLD_HEIGHT);
    overlay.fill({ color: 0x000000, alpha: 0.4 });
    this.container.addChild(overlay);

    // Title glow (drawn behind, slightly larger, lower alpha)
    const titleGlowStyle = new TextStyle({
      fontFamily: FONT_FAMILY,
      fontSize: 50,
      fill: NEON_CYAN,
      fontWeight: "bold",
      letterSpacing: 14,
    });
    const titleGlow = new Text({ text: "DETERRENCE", style: titleGlowStyle });
    titleGlow.anchor.set(0.5, 0);
    titleGlow.x = WORLD_WIDTH / 2;
    titleGlow.y = 138;
    titleGlow.alpha = 0.15;
    this.container.addChild(titleGlow);

    // Title
    const titleStyle = new TextStyle({
      fontFamily: FONT_FAMILY,
      fontSize: 48,
      fill: NEON_CYAN,
      fontWeight: "bold",
      letterSpacing: 12,
    });
    const title = new Text({ text: "DETERRENCE", style: titleStyle });
    title.anchor.set(0.5, 0);
    title.x = WORLD_WIDTH / 2;
    title.y = 140;
    this.container.addChild(title);

    // Subtitle
    const subtitleStyle = new TextStyle({
      fontFamily: FONT_FAMILY,
      fontSize: 16,
      fill: ELECTRIC_BLUE,
      letterSpacing: 2,
    });
    const subtitle = new Text({
      text: "A MISSILE DEFENSE SIMULATION",
      style: subtitleStyle,
    });
    subtitle.anchor.set(0.5, 0);
    subtitle.x = WORLD_WIDTH / 2;
    subtitle.y = 200;
    this.container.addChild(subtitle);

    // Main menu panel
    this.mainPanel = new Container();
    this.container.addChild(this.mainPanel);
    this.buildMainPanel();

    // Load game panel
    this.loadPanel = new Container();
    this.loadPanel.visible = false;
    this.container.addChild(this.loadPanel);

    // Settings panel
    this.settingsPanel = new Container();
    this.settingsPanel.visible = false;
    this.container.addChild(this.settingsPanel);
    this.buildSettingsPanel();

    // Version + hints at bottom
    this.versionText = new Text({
      text: "v0.1  |  C: CRT  |  M: Mute  |  F11: Fullscreen",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 12,
        fill: DIM_TEXT,
      }),
    });
    this.versionText.anchor.set(0.5, 0);
    this.versionText.x = WORLD_WIDTH / 2;
    this.versionText.y = WORLD_HEIGHT - 30;
    this.container.addChild(this.versionText);

    // Ticker for UI animations
    app.ticker.add((ticker) => {
      if (this.container.visible) {
        for (const t of this.tickables) {
          t.tick(ticker.deltaTime);
        }
      }
    });
  }

  get visible(): boolean {
    return this.container.visible;
  }

  set visible(v: boolean) {
    this.container.visible = v;
    if (v) {
      this.showPanel("main");
    }
  }

  getSettings(): GameSettings {
    return { ...this.settings };
  }

  private buildMainPanel() {
    const cx = WORLD_WIDTH / 2;

    const newGameBtn = new NeonButton({
      label: "NEW GAME",
      width: 260,
      height: 50,
      fontSize: 22,
      color: NEON_CYAN,
      onClick: () => this.onNewGame?.(),
    });
    newGameBtn.x = cx;
    newGameBtn.y = 310;
    this.mainPanel.addChild(newGameBtn);
    this.tickables.push(newGameBtn);

    const loadBtn = new NeonButton({
      label: "LOAD GAME",
      width: 260,
      height: 50,
      fontSize: 22,
      color: NEON_CYAN,
      onClick: () => this.showLoadPanel(),
    });
    loadBtn.x = cx;
    loadBtn.y = 380;
    this.mainPanel.addChild(loadBtn);
    this.tickables.push(loadBtn);

    const settingsBtn = new NeonButton({
      label: "SETTINGS",
      width: 260,
      height: 50,
      fontSize: 22,
      color: ELECTRIC_BLUE,
      onClick: () => this.showPanel("settings"),
    });
    settingsBtn.x = cx;
    settingsBtn.y = 450;
    this.mainPanel.addChild(settingsBtn);
    this.tickables.push(settingsBtn);
  }

  private buildSettingsPanel() {
    const cx = WORLD_WIDTH / 2;
    const leftCol = cx - 250;

    // Header
    const header = new Text({
      text: "S E T T I N G S",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 24,
        fill: NEON_CYAN,
        fontWeight: "bold",
        letterSpacing: 4,
      }),
    });
    header.anchor.set(0.5, 0);
    header.x = cx;
    header.y = 230;
    this.settingsPanel.addChild(header);

    // Divider line
    const divider = new Graphics();
    divider.moveTo(cx - 200, 265);
    divider.lineTo(cx + 200, 265);
    divider.stroke({ width: 1, color: NEON_CYAN, alpha: 0.3 });
    this.settingsPanel.addChild(divider);

    // --- DISPLAY SECTION ---
    const displayLabel = new Text({
      text: "DISPLAY",
      style: new TextStyle({ fontFamily: FONT_FAMILY, fontSize: 14, fill: DIM_TEXT }),
    });
    displayLabel.x = leftCol;
    displayLabel.y = 280;
    this.settingsPanel.addChild(displayLabel);

    // CRT Effect toggle
    const crtLabel = new Text({
      text: "CRT Effect",
      style: new TextStyle({ fontFamily: FONT_FAMILY, fontSize: 16, fill: NEON_CYAN }),
    });
    crtLabel.x = leftCol;
    crtLabel.y = 310;
    this.settingsPanel.addChild(crtLabel);

    this.crtToggle = new NeonToggle({
      value: this.settings.crtEnabled,
      color: NEON_GREEN,
      onChange: (val) => {
        this.settings.crtEnabled = val;
        this.saveSettings();
        this.onCRTToggle?.();
      },
    });
    this.crtToggle.x = leftCol + 140;
    this.crtToggle.y = 310;
    this.settingsPanel.addChild(this.crtToggle);
    this.tickables.push(this.crtToggle);

    // Fullscreen toggle
    const fsLabel = new Text({
      text: "Fullscreen",
      style: new TextStyle({ fontFamily: FONT_FAMILY, fontSize: 16, fill: NEON_CYAN }),
    });
    fsLabel.x = leftCol + 260;
    fsLabel.y = 310;
    this.settingsPanel.addChild(fsLabel);

    this.fullscreenToggle = new NeonToggle({
      value: this.settings.fullscreen,
      color: NEON_GREEN,
      onChange: async (val) => {
        this.settings.fullscreen = val;
        this.saveSettings();
        try {
          await setFullscreen(val);
        } catch {
          // Fullscreen may not be available
        }
      },
    });
    this.fullscreenToggle.x = leftCol + 400;
    this.fullscreenToggle.y = 310;
    this.settingsPanel.addChild(this.fullscreenToggle);
    this.tickables.push(this.fullscreenToggle);

    // --- RESOLUTION SECTION ---
    const resLabel = new Text({
      text: "RESOLUTION",
      style: new TextStyle({ fontFamily: FONT_FAMILY, fontSize: 14, fill: DIM_TEXT }),
    });
    resLabel.x = leftCol;
    resLabel.y = 355;
    this.settingsPanel.addChild(resLabel);

    const resPresets = ["720p", "1080p", "1440p", "4K"];
    let resX = leftCol;
    for (const preset of resPresets) {
      const btn = new NeonButton({
        label: preset,
        width: 90,
        height: 36,
        fontSize: 14,
        color: ELECTRIC_BLUE,
        onClick: () => this.selectResolution(preset),
      });
      btn.x = resX + 45; // NeonButton pivot is centered
      btn.y = 393;
      this.settingsPanel.addChild(btn);
      this.tickables.push(btn);
      this.resolutionButtons.set(preset, btn);
      resX += 105;
    }
    this.updateResolutionButtons();

    // --- AUDIO SECTION ---
    const audioSectionLabel = new Text({
      text: "AUDIO",
      style: new TextStyle({ fontFamily: FONT_FAMILY, fontSize: 14, fill: DIM_TEXT }),
    });
    audioSectionLabel.x = leftCol;
    audioSectionLabel.y = 430;
    this.settingsPanel.addChild(audioSectionLabel);

    // Sound toggle
    const soundLabel = new Text({
      text: "Sound",
      style: new TextStyle({ fontFamily: FONT_FAMILY, fontSize: 16, fill: NEON_CYAN }),
    });
    soundLabel.x = leftCol;
    soundLabel.y = 460;
    this.settingsPanel.addChild(soundLabel);

    this.audioToggle = new NeonToggle({
      value: this.settings.audioEnabled,
      color: NEON_GREEN,
      onChange: (val) => {
        this.settings.audioEnabled = val;
        this.saveSettings();
        this.onMuteToggle?.();
      },
    });
    this.audioToggle.x = leftCol + 80;
    this.audioToggle.y = 460;
    this.settingsPanel.addChild(this.audioToggle);
    this.tickables.push(this.audioToggle);

    // Volume slider
    const volLabel = new Text({
      text: "Volume",
      style: new TextStyle({ fontFamily: FONT_FAMILY, fontSize: 16, fill: NEON_CYAN }),
    });
    volLabel.x = leftCol + 200;
    volLabel.y = 460;
    this.settingsPanel.addChild(volLabel);

    this.volumeSlider = new NeonSlider({
      width: 200,
      value: this.settings.volume,
      color: NEON_CYAN,
      onChange: (val) => {
        this.settings.volume = val;
        this.saveSettings();
        this.onVolumeChange?.(val);
      },
    });
    this.volumeSlider.x = leftCol + 280;
    this.volumeSlider.y = 462;
    this.settingsPanel.addChild(this.volumeSlider);
    this.tickables.push(this.volumeSlider);

    // Back button
    const backBtn = new NeonButton({
      label: "BACK",
      width: 160,
      height: 42,
      fontSize: 18,
      color: NEON_CYAN,
      onClick: () => this.showPanel("main"),
    });
    backBtn.x = cx;
    backBtn.y = 530;
    this.settingsPanel.addChild(backBtn);
    this.tickables.push(backBtn);
  }

  private selectResolution(preset: string) {
    this.settings.resolution = preset;
    this.saveSettings();
    this.updateResolutionButtons();

    const res = RESOLUTION_PRESETS[preset];
    if (res) {
      setWindowResolution(res.width, res.height).catch(() => {
        // Ignore errors (e.g. when in fullscreen, size may not apply)
      });
    }
  }

  private updateResolutionButtons() {
    for (const [preset, btn] of this.resolutionButtons) {
      btn.setActive(preset === this.settings.resolution);
    }
  }

  private async showLoadPanel() {
    // Clear previous load panel contents
    this.loadPanel.removeChildren();
    // Remove old load panel tickables (keep non-load tickables)
    // Load panel buttons get garbage collected when removeChildren destroys them

    const header = new Text({
      text: "LOAD GAME",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 24,
        fill: NEON_CYAN,
        fontWeight: "bold",
      }),
    });
    header.anchor.set(0.5, 0);
    header.x = WORLD_WIDTH / 2;
    header.y = 280;
    this.loadPanel.addChild(header);

    // Loading text
    const loadingText = new Text({
      text: "Loading saves...",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 16,
        fill: DIM_TEXT,
      }),
    });
    loadingText.anchor.set(0.5, 0);
    loadingText.x = WORLD_WIDTH / 2;
    loadingText.y = 340;
    this.loadPanel.addChild(loadingText);

    this.showPanel("load");

    // Fetch saves
    let saves: SaveMetadata[] = [];
    try {
      saves = await listSaves();
    } catch {
      // No saves or error
    }

    // Remove loading text
    this.loadPanel.removeChild(loadingText);
    loadingText.destroy();

    if (saves.length === 0) {
      const noSaves = new Text({
        text: "NO SAVED GAMES FOUND",
        style: new TextStyle({
          fontFamily: FONT_FAMILY,
          fontSize: 16,
          fill: SOLAR_YELLOW,
        }),
      });
      noSaves.anchor.set(0.5, 0);
      noSaves.x = WORLD_WIDTH / 2;
      noSaves.y = 340;
      this.loadPanel.addChild(noSaves);
    } else {
      // Sort by timestamp descending (most recent first)
      saves.sort((a, b) => b.timestamp - a.timestamp);

      let y = 330;
      for (const save of saves) {
        const date = new Date(save.timestamp * 1000);
        const dateStr = date.toLocaleString();
        const label = `${save.slot_name.toUpperCase()} - WAVE ${save.wave_number} - $${save.resources} - ${dateStr}`;

        const saveBtn = new NeonButton({
          label,
          width: 600,
          height: 34,
          fontSize: 13,
          color: NEON_CYAN,
          onClick: () => this.onLoadGame?.(save.slot_name),
        });
        saveBtn.x = WORLD_WIDTH / 2;
        saveBtn.y = y;
        this.loadPanel.addChild(saveBtn);
        this.tickables.push(saveBtn);
        y += 42;
      }
    }

    // Back button
    const backY = saves.length > 0 ? 330 + saves.length * 42 + 20 : 400;
    const backBtn = new NeonButton({
      label: "BACK",
      width: 160,
      height: 42,
      fontSize: 18,
      color: NEON_CYAN,
      onClick: () => this.showPanel("main"),
    });
    backBtn.x = WORLD_WIDTH / 2;
    backBtn.y = backY;
    this.loadPanel.addChild(backBtn);
    this.tickables.push(backBtn);
  }

  private showPanel(panel: MenuPanel) {
    this.mainPanel.visible = panel === "main";
    this.loadPanel.visible = panel === "load";
    this.settingsPanel.visible = panel === "settings";
  }

  /** Update CRT display state (called from external toggle) */
  setCRTState(on: boolean) {
    this.settings.crtEnabled = on;
    if (this.crtToggle) this.crtToggle.value = on;
  }

  /** Update audio display state (called from external toggle) */
  setAudioState(on: boolean) {
    this.settings.audioEnabled = on;
    if (this.audioToggle) this.audioToggle.value = on;
  }

  /** Update fullscreen state (called from external toggle) */
  setFullscreenState(on: boolean) {
    this.settings.fullscreen = on;
    if (this.fullscreenToggle) this.fullscreenToggle.value = on;
  }

  // --- Settings persistence ---

  static loadSettings(): GameSettings {
    try {
      const stored = localStorage.getItem("deterrence_settings");
      if (stored) {
        return { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
      }
    } catch {
      // Invalid JSON or no localStorage
    }
    return { ...DEFAULT_SETTINGS };
  }

  private saveSettings() {
    try {
      localStorage.setItem("deterrence_settings", JSON.stringify(this.settings));
    } catch {
      // localStorage may not be available
    }
  }

  /** Apply saved settings on startup (called by GameRenderer) */
  async applyStartupSettings(): Promise<GameSettings> {
    const s = this.settings;

    // Apply resolution
    const res = RESOLUTION_PRESETS[s.resolution];
    if (res && s.resolution !== "720p") {
      try {
        await setWindowResolution(res.width, res.height);
      } catch {
        // Ignore
      }
    }

    // Apply fullscreen
    if (s.fullscreen) {
      try {
        await setFullscreen(true);
      } catch {
        // Ignore
      }
    }

    return s;
  }
}
