import { create } from "zustand";
import type { CampaignSnapshot } from "../types/campaign";
import type { WaveCompleteEvent } from "../types/events";
import type { SaveMetadata } from "../types/commands";

export type GamePhase =
  | "MainMenu"
  | "Strategic"
  | "WaveActive"
  | "WaveResult"
  | string;

export interface GameSettings {
  audioEnabled: boolean;
  volume: number;
  musicVolume: number;
  resolution: string;
  fullscreen: boolean;
}

export interface BatteryStatus {
  index: number;
  ammo: number;
  maxAmmo: number;
  typeName: string;
}

export interface HudState {
  waveNumber: number;
  citiesAlive: number;
  citiesTotal: number;
  contactsTotal: number;
  contactsRadar: number;
  contactsGlow: number;
  weather: string | null;
  windX: number;
  resources: number;
  waveIncome: number | null;
  battery: BatteryStatus | null;
  muted: boolean;
}

const SETTINGS_KEY = "deterrence_settings";

const DEFAULT_SETTINGS: GameSettings = {
  audioEnabled: true,
  volume: 0.5,
  musicVolume: 0.7,
  resolution: "720p",
  fullscreen: false,
};

const DEFAULT_HUD: HudState = {
  waveNumber: 0,
  citiesAlive: 0,
  citiesTotal: 0,
  contactsTotal: 0,
  contactsRadar: 0,
  contactsGlow: 0,
  weather: null,
  windX: 0,
  resources: 0,
  waveIncome: null,
  battery: null,
  muted: false,
};

function loadSettings(): GameSettings {
  try {
    const stored = localStorage.getItem(SETTINGS_KEY);
    if (stored) {
      return { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
    }
  } catch {
    // Ignore invalid storage
  }
  return { ...DEFAULT_SETTINGS };
}

function saveSettings(settings: GameSettings) {
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
  } catch {
    // Ignore storage errors
  }
}

interface GameStoreState {
  phase: GamePhase;
  hud: HudState;
  campaign: CampaignSnapshot | null;
  hoveredRegionId: number | null;
  waveComplete: WaveCompleteEvent | null;
  saves: SaveMetadata[];
  settings: GameSettings;
  screenShake: { x: number; y: number };

  setPhase: (phase: GamePhase) => void;
  setHud: (partial: Partial<HudState>) => void;
  setCampaign: (campaign: CampaignSnapshot | null) => void;
  setHoveredRegionId: (id: number | null) => void;
  setWaveComplete: (event: WaveCompleteEvent | null) => void;
  setSaves: (saves: SaveMetadata[]) => void;
  setSettings: (settings: GameSettings) => void;
  updateSettings: (partial: Partial<GameSettings>) => void;
  setScreenShake: (x: number, y: number) => void;
}

export const useGameStore = create<GameStoreState>((set, get) => ({
  phase: "MainMenu",
  hud: { ...DEFAULT_HUD },
  campaign: null,
  hoveredRegionId: null,
  waveComplete: null,
  saves: [],
  settings: loadSettings(),
  screenShake: { x: 0, y: 0 },

  setPhase: (phase) => set({ phase }),
  setHud: (partial) =>
    set((state) => ({
      hud: { ...state.hud, ...partial },
    })),
  setCampaign: (campaign) => set({ campaign }),
  setHoveredRegionId: (id) => set({ hoveredRegionId: id }),
  setWaveComplete: (event) => set({ waveComplete: event }),
  setSaves: (saves) => set({ saves }),
  setSettings: (settings) => {
    saveSettings(settings);
    set({ settings });
  },
  updateSettings: (partial) => {
    const next = { ...get().settings, ...partial };
    saveSettings(next);
    set({ settings: next });
  },
  setScreenShake: (x, y) => set({ screenShake: { x, y } }),
}));
