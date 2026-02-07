import { Application } from "pixi.js";
import { TacticalView } from "./TacticalView";
import { StrategicView } from "./StrategicView";
import { MenuBackground } from "./effects/MenuBackground";
import { ParticleManager } from "./effects/ParticleManager";
import { CRTFilter } from "./shaders/CRTFilter";
import { AudioManager } from "../audio/AudioManager";
import { InputManager } from "../input/InputManager";
import { setFullscreen, setWindowResolution } from "../bridge/commands";
import { registerGameActions } from "../ui/gameActions";
import { useGameStore } from "../ui/store";
import {
  onStateSnapshot,
  onDetonation,
  onWaveComplete,
  onCampaignUpdate,
  onMirvSplit,
  onImpact,
} from "../bridge/events";
import type { StateSnapshot } from "../types/snapshot";
import type { WaveCompleteEvent, MirvSplitEvent } from "../types/events";
import type { CampaignSnapshot } from "../types/campaign";

const WORLD_WIDTH = 1280;
const WORLD_HEIGHT = 720;

export class GameRenderer {
  app: Application;
  tacticalView!: TacticalView;
  strategicView!: StrategicView;
  inputManager!: InputManager;
  private menuBackground!: MenuBackground;
  private particleManager!: ParticleManager;
  private crtFilter?: CRTFilter;
  private crtEnabled: boolean = false;
  private audio!: AudioManager;
  private lastPhase: string = "MainMenu";
  private lastSnapshot: StateSnapshot | null = null;
  private store = useGameStore;

  // Screen shake state
  private shakeDecay: number = 0;
  private shakeIntensity: number = 0;

  constructor() {
    this.app = new Application();
  }

  async init() {
    await this.app.init({
      background: "#000000",
      width: WORLD_WIDTH,
      height: WORLD_HEIGHT,
      resolution: window.devicePixelRatio || 1,
      autoDensity: true,
      antialias: true,
    });

    const gameRoot = document.getElementById("game-root");
    if (gameRoot) {
      gameRoot.appendChild(this.app.canvas);
    } else {
      document.body.appendChild(this.app.canvas);
    }

    // Scale canvas to fit window while maintaining 16:9 aspect ratio
    this.handleResize();
    window.addEventListener("resize", () => this.handleResize());

    // CRT post-process (used for menu background ambience)
    this.crtFilter = new CRTFilter(this.app.renderer.width, this.app.renderer.height);
    this.crtFilter.scanlineIntensity = 0.22;

    // Menu background (added first — renders behind everything)
    this.menuBackground = new MenuBackground(this.app, WORLD_WIDTH, WORLD_HEIGHT);

    this.tacticalView = new TacticalView(this.app, WORLD_WIDTH, WORLD_HEIGHT);
    this.strategicView = new StrategicView(this.app);

    // Particle manager (renders on top of tactical entities)
    this.particleManager = new ParticleManager(this.app.stage);
    this.inputManager = new InputManager(this.app, WORLD_WIDTH, WORLD_HEIGHT);

    // Wire arc prediction from InputManager to TacticalView
    this.inputManager.onArcUpdate = (prediction) => {
      this.tacticalView.updateArcOverlay(prediction);
    };

    // Wire battery selection change to highlight + HUD
    this.inputManager.onBatteryChange = (batteryId) => {
      const pos = this.inputManager.getBatteryPosition(batteryId);
      this.tacticalView.updateBatteryHighlight(batteryId, pos?.x);
      this.updateBatteryHudFromSnapshot();
    };

    // Wire interceptor type change to HUD refresh
    this.inputManager.onTypeChange = (_typeName) => {
      this.updateBatteryHudFromSnapshot();
    };

    this.inputManager.onMapHover = (mapX, mapY) => {
      const region = this.strategicView.getRegionAt(mapX, mapY);
      this.store.getState().setHoveredRegionId(region?.id ?? null);
    };

    this.inputManager.onFullscreenChange = (fullscreen) => {
      this.store.getState().updateSettings({ fullscreen });
    };

    // Show main menu by default (initial snapshot may arrive before listener is ready)
    this.setViewForPhase("MainMenu");

    // Show initial battery highlight
    const initialPos = this.inputManager.getBatteryPosition(this.inputManager.selectedBattery);
    this.tacticalView.updateBatteryHighlight(this.inputManager.selectedBattery, initialPos?.x);

    // Listen for state snapshots from backend
    onStateSnapshot((snapshot: StateSnapshot) => {
      this.lastSnapshot = snapshot;

      // Switch views based on phase
      this.setViewForPhase(snapshot.phase);

      if (snapshot.phase === "WaveActive" || snapshot.phase === "WaveResult") {
        this.tacticalView.update(snapshot);
      }

      // Track phase and wind for input manager
      this.inputManager.setPhase(snapshot.phase);
      this.inputManager.setWindX(snapshot.wind_x ?? 0);

      // Update HUD store data
      this.updateHudFromSnapshot(snapshot);
      this.updateBatteryHudFromSnapshot();

      // Audio: phase transitions
      if (snapshot.phase !== this.lastPhase) {
        this.audio.setPhase(snapshot.phase, snapshot.wave_number);
        if (snapshot.phase === "WaveActive") {
          this.audio.playWaveStart();
          this.audio.startAmbient(snapshot.weather ?? "Clear");
        } else if (snapshot.phase !== "WaveActive" && this.lastPhase === "WaveActive") {
          this.audio.stopAmbient();
        }
        this.lastPhase = snapshot.phase;
      }

      // Hide wave complete overlay when new wave starts
      if (snapshot.phase === "WaveActive") {
        this.store.getState().setWaveComplete(null);
      }

      // Clear overlays when not in WaveActive
      if (snapshot.phase !== "WaveActive") {
        this.tacticalView.clearOverlays();
      }
    });

    // Listen for campaign state updates
    onCampaignUpdate((campaign: CampaignSnapshot) => {
      this.strategicView.update(campaign);
      // Store battery positions for input manager
      this.inputManager.updateBatteryPositions(campaign);
      const batPos = this.inputManager.getBatteryPosition(this.inputManager.selectedBattery);
      this.tacticalView.updateBatteryHighlight(this.inputManager.selectedBattery, batPos?.x);
      this.updateBatteryHudFromSnapshot();
      this.store.getState().setCampaign(campaign);
      this.store.getState().setHud({
        resources: campaign.resources,
        waveIncome: campaign.wave_income ?? null,
      });
    });

    // Audio manager
    this.audio = new AudioManager();

    registerGameActions({
      setMuted: (muted) => this.setMuted(muted, true),
      setVolume: (volume) => this.setVolume(volume),
      setSfxVolume: (volume) => this.setSfxVolume(volume),
      setMusicVolume: (volume) => this.setMusicVolume(volume),
      playUiClick: () => this.playUiClick(),
      handleStrategicAction: (action) => this.inputManager.handleStrategicAction(action),
    });

    // Wire launch sound from InputManager
    this.inputManager.onLaunchSound = (worldX: number) => {
      this.audio.playLaunch(worldX);
    };

    // Wire mute toggle from InputManager
    this.inputManager.onMuteToggle = () => {
      const muted = this.audio.toggleMute();
      this.store.getState().setHud({ muted });
      this.store.getState().updateSettings({ audioEnabled: !muted });
    };

    // Listen for detonation events — scale visuals to yield
    onDetonation((event) => {
      const intensity = Math.min(event.yield_force / 80, 3.0);
      this.audio.playDetonation(event.x, intensity);
      this.particleManager.spawnExplosion(event.x, event.y, intensity);
      this.triggerScreenShake(intensity);
    });

    // Listen for impact events
    onImpact((event) => {
      this.particleManager.spawnImpact(event.x, event.y);
      this.triggerScreenShake(0.5);
    });

    // Listen for MIRV split events
    onMirvSplit((event: MirvSplitEvent) => {
      this.tacticalView.addMirvSplitEffect(event.x, event.y);
      this.particleManager.spawnMirvSplit(event.x, event.y);
      this.audio.playMirvSplit(event.x);
    });

    // Listen for wave completion
    onWaveComplete((event: WaveCompleteEvent) => {
      this.store.getState().setWaveComplete(event);
      this.audio.playWaveComplete();
    });

    // Apply saved settings
    const settings = this.store.getState().settings;
    if (settings.resolution && settings.resolution !== "720p") {
      const presets: Record<string, { width: number; height: number }> = {
        "720p": { width: 1280, height: 720 },
        "1080p": { width: 1920, height: 1080 },
        "1440p": { width: 2560, height: 1440 },
        "4K": { width: 3840, height: 2160 },
      };
      const res = presets[settings.resolution];
      if (res) {
        setWindowResolution(res.width, res.height).catch(() => {
          // Ignore size errors
        });
      }
    }
    if (settings.fullscreen) {
      setFullscreen(true).catch(() => {
        // Ignore fullscreen errors
      });
    }

    if (!settings.audioEnabled && !this.audio.muted) {
      this.audio.toggleMute();
    }
    this.audio.setVolume(settings.volume);
    this.audio.setSfxVolume(settings.sfxVolume ?? 0.8);
    this.audio.setMusicVolume(settings.musicVolume ?? 0.7);
    this.store.getState().setHud({ muted: this.audio.muted });

    // Preload and start title music
    this.audio.preloadMusic("MainMenu", 0);
    this.audio.setPhase("MainMenu", 0);

    // Update menu background + particles each frame
    this.app.ticker.add((ticker) => {
      const dt = ticker.deltaTime;
      this.menuBackground.update(dt);
      this.particleManager.update(dt);
      if (this.crtEnabled && this.crtFilter) {
        this.crtFilter.update(dt);
      }

      // Screen shake
      if (this.shakeDecay > 0) {
        this.shakeDecay -= dt * (1 / 60);
        const amp = this.shakeIntensity * Math.max(0, this.shakeDecay / 0.15);
        const shakeX = (Math.random() - 0.5) * amp * 4;
        const shakeY = (Math.random() - 0.5) * amp * 4;
        this.app.stage.x = shakeX;
        this.app.stage.y = shakeY;
        this.store.getState().setScreenShake(shakeX, shakeY);
      } else {
        if (this.app.stage.x !== 0 || this.app.stage.y !== 0) {
          this.app.stage.x = 0;
          this.app.stage.y = 0;
        }
        const uiShake = this.store.getState().screenShake;
        if (uiShake.x !== 0 || uiShake.y !== 0) {
          this.store.getState().setScreenShake(0, 0);
        }
      }
    });

  }

  private triggerScreenShake(intensity: number) {
    this.shakeDecay = 0.15;
    this.shakeIntensity = Math.min(intensity, 3);
  }

  private handleResize() {
    const windowWidth = window.innerWidth;
    const windowHeight = window.innerHeight;
    const scale = Math.min(windowWidth / WORLD_WIDTH, windowHeight / WORLD_HEIGHT);
    const canvas = this.app.canvas;
    const viewWidth = Math.round(WORLD_WIDTH * scale);
    const viewHeight = Math.round(WORLD_HEIGHT * scale);
    this.app.renderer.resize(viewWidth, viewHeight);
    this.app.stage.scale.set(viewWidth / WORLD_WIDTH, viewHeight / WORLD_HEIGHT);
    canvas.style.width = `${viewWidth}px`;
    canvas.style.height = `${viewHeight}px`;
    this.crtFilter?.setResolution(viewWidth, viewHeight);
  }

  private setViewForPhase(phase: string) {
    this.store.getState().setPhase(phase);
    if (phase === "MainMenu") {
      this.menuBackground.visible = true;
      this.menuBackground.start();
      this.crtEnabled = true;
      if (this.crtFilter) {
        this.app.stage.filters = [this.crtFilter];
      }
      this.strategicView.visible = false;
      this.tacticalView.visible = false;
      this.store.getState().setCampaign(null);
    } else if (phase === "Strategic") {
      this.menuBackground.visible = false;
      this.menuBackground.stop();
      this.crtEnabled = false;
      this.app.stage.filters = null;
      this.strategicView.visible = true;
      this.tacticalView.visible = false;
    } else {
      this.menuBackground.visible = false;
      this.menuBackground.stop();
      this.crtEnabled = false;
      this.app.stage.filters = null;
      this.strategicView.visible = false;
      this.tacticalView.visible = true;
    }
    if (phase !== "Strategic") {
      this.store.getState().setHoveredRegionId(null);
    }
  }

  private updateHudFromSnapshot(snapshot: StateSnapshot) {
    const cities = snapshot.entities.filter((e) => e.entity_type === "City");
    const aliveCities = cities.filter((e) => {
      if (e.extra && "City" in e.extra) {
        return (e.extra as { City: { health: number } }).City.health > 0;
      }
      return true;
    });

    const missiles = snapshot.entities.filter((e) => e.entity_type === "Missile");
    let radarCount = 0;
    let glowCount = 0;
    for (const m of missiles) {
      if (m.extra && "Missile" in m.extra) {
        const data = (
          m.extra as { Missile: { detected_by_radar: boolean; detected_by_glow: boolean } }
        ).Missile;
        if (data.detected_by_radar) radarCount++;
        else if (data.detected_by_glow) glowCount++;
      }
    }

    this.store.getState().setHud({
      waveNumber: snapshot.wave_number,
      citiesAlive: aliveCities.length,
      citiesTotal: cities.length,
      contactsTotal: missiles.length,
      contactsRadar: radarCount,
      contactsGlow: glowCount,
      weather: snapshot.weather ?? null,
      windX: snapshot.wind_x ?? 0,
    });
  }

  private updateBatteryHudFromSnapshot() {
    if (!this.lastSnapshot) return;
    const batteries = this.lastSnapshot.entities.filter((e) => e.entity_type === "Battery");
    const selectedBat = this.inputManager.selectedBattery;
    const expected = this.inputManager.getBatteryPosition(selectedBat);

    let chosen = null;
    if (expected) {
      let bestDist = Infinity;
      for (const bat of batteries) {
        const dx = bat.x - expected.x;
        const dy = bat.y - expected.y;
        const dist = dx * dx + dy * dy;
        if (dist < bestDist) {
          bestDist = dist;
          chosen = bat;
        }
      }
    } else if (selectedBat < batteries.length) {
      chosen = batteries.sort((a, b) => a.x - b.x)[selectedBat] ?? null;
    }

    if (chosen && chosen.extra && "Battery" in chosen.extra) {
      const batData = (
        chosen.extra as { Battery: { ammo: number; max_ammo: number } }
      ).Battery;
      this.store.getState().setHud({
        battery: {
          index: selectedBat,
          ammo: batData.ammo,
          maxAmmo: batData.max_ammo,
          typeName: this.inputManager.selectedType,
        },
      });
      return;
    }

    this.store.getState().setHud({ battery: null });
  }

  private setMuted(muted: boolean, persist: boolean) {
    if (this.audio.muted !== muted) {
      this.audio.toggleMute();
    }
    this.store.getState().setHud({ muted });
    if (persist) {
      this.store.getState().updateSettings({ audioEnabled: !muted });
    }
  }

  private setVolume(volume: number) {
    this.audio.setVolume(volume);
  }

  private setSfxVolume(volume: number) {
    this.audio.setSfxVolume(volume);
  }

  private setMusicVolume(volume: number) {
    this.audio.setMusicVolume(volume);
  }

  private playUiClick() {
    this.audio.playUiClick();
  }
}
