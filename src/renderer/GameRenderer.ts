import { Application } from "pixi.js";
import { TacticalView } from "./TacticalView";
import { StrategicView } from "./StrategicView";
import { MainMenuView } from "./MainMenuView";
import { MenuBackground } from "./effects/MenuBackground";
import { ParticleManager } from "./effects/ParticleManager";
import { HUD } from "./HUD";
import { CRTFilter } from "./shaders/CRTFilter";
import { AudioManager } from "../audio/AudioManager";
import { InputManager } from "../input/InputManager";
import { newGame, loadGame } from "../bridge/commands";
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
  hud!: HUD;
  inputManager!: InputManager;
  private mainMenuView!: MainMenuView;
  private menuBackground!: MenuBackground;
  private particleManager!: ParticleManager;
  private crtFilter!: CRTFilter;
  private crtEnabled: boolean = true;
  private audio!: AudioManager;
  private lastPhase: string = "MainMenu";

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
      resolution: 1,
      antialias: true,
    });

    document.body.appendChild(this.app.canvas);

    // Scale canvas to fit window while maintaining 16:9 aspect ratio
    this.handleResize();
    window.addEventListener("resize", () => this.handleResize());

    // Menu background (added first — renders behind everything)
    this.menuBackground = new MenuBackground(this.app, WORLD_WIDTH, WORLD_HEIGHT);

    this.tacticalView = new TacticalView(this.app, WORLD_WIDTH, WORLD_HEIGHT);
    this.strategicView = new StrategicView(this.app);

    // Particle manager (renders on top of tactical entities)
    this.particleManager = new ParticleManager(this.app.stage);

    // Main menu view (added after views so it renders on top)
    this.mainMenuView = new MainMenuView(this.app);

    this.hud = new HUD(this.app, WORLD_WIDTH, WORLD_HEIGHT);
    this.inputManager = new InputManager(this.app, WORLD_WIDTH, WORLD_HEIGHT);

    // Wire arc prediction from InputManager to TacticalView
    this.inputManager.onArcUpdate = (prediction) => {
      this.tacticalView.updateArcOverlay(prediction);
    };

    // Wire battery selection change to highlight + HUD
    this.inputManager.onBatteryChange = (batteryId) => {
      this.tacticalView.updateBatteryHighlight(batteryId);
    };

    // Wire interceptor type change to HUD refresh
    this.inputManager.onTypeChange = (_typeName) => {
      // HUD will be refreshed on next state snapshot via updateBatteryHUD
    };

    // Wire strategic action clicks
    this.strategicView.onActionClick = (action, _index) => {
      this.inputManager.handleStrategicAction(action);
    };

    // Wire main menu callbacks
    this.mainMenuView.onNewGame = () => {
      newGame();
    };
    this.mainMenuView.onLoadGame = (slotName: string) => {
      loadGame(slotName);
    };
    this.mainMenuView.onCRTToggle = () => {
      this.crtEnabled = !this.crtEnabled;
      this.app.stage.filters = this.crtEnabled ? [this.crtFilter] : [];
      this.mainMenuView.setCRTState(this.crtEnabled);
    };
    this.mainMenuView.onMuteToggle = () => {
      const muted = this.audio.toggleMute();
      this.hud.updateMuteState(muted);
      this.mainMenuView.setAudioState(!muted);
    };
    this.mainMenuView.onVolumeChange = (volume: number) => {
      this.audio.setVolume(volume);
    };

    // Show main menu by default (initial snapshot may arrive before listener is ready)
    this.setViewForPhase("MainMenu");

    // Show initial battery highlight
    this.tacticalView.updateBatteryHighlight(this.inputManager.selectedBattery);

    // Listen for state snapshots from backend
    onStateSnapshot((snapshot: StateSnapshot) => {
      // Switch views based on phase
      this.setViewForPhase(snapshot.phase);

      if (snapshot.phase === "WaveActive" || snapshot.phase === "WaveResult") {
        this.tacticalView.update(snapshot);
      }
      if (snapshot.phase !== "MainMenu") {
        this.hud.update(snapshot);
      }

      // Track phase and wind for input manager
      this.inputManager.setPhase(snapshot.phase);
      this.inputManager.setWindX(snapshot.wind_x ?? 0);

      // Audio: phase transitions
      if (snapshot.phase !== this.lastPhase) {
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
        this.hud.hideWaveComplete();
      }

      // Clear overlays when not in WaveActive
      if (snapshot.phase !== "WaveActive") {
        this.tacticalView.clearOverlays();
      }

      // Update battery ammo display in HUD
      this.updateBatteryHUD(snapshot);
    });

    // Listen for campaign state updates
    onCampaignUpdate((campaign: CampaignSnapshot) => {
      this.strategicView.update(campaign);
      this.hud.updateResources(campaign.resources);
      // Store battery positions for input manager
      this.inputManager.updateBatteryPositions(campaign);
    });

    // Audio manager
    this.audio = new AudioManager();

    // Wire launch sound from InputManager
    this.inputManager.onLaunchSound = (worldX: number) => {
      this.audio.playLaunch(worldX);
    };

    // Wire mute toggle from InputManager
    this.inputManager.onMuteToggle = () => {
      const muted = this.audio.toggleMute();
      this.hud.updateMuteState(muted);
    };

    // Listen for detonation events
    onDetonation((event) => {
      this.audio.playDetonation(event.x, Math.min(event.yield_force / 100, 2.0));
      this.particleManager.spawnExplosion(event.x, event.y, Math.min(event.yield_force / 100, 2.0));
      this.triggerScreenShake(Math.min(event.yield_force / 100, 2.0));
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
      this.hud.showWaveComplete(event);
      this.audio.playWaveComplete();
    });

    // CRT post-processing filter
    this.crtFilter = new CRTFilter(WORLD_WIDTH, WORLD_HEIGHT);

    // Apply saved settings
    const settings = await this.mainMenuView.applyStartupSettings();
    this.crtEnabled = settings.crtEnabled;
    this.app.stage.filters = this.crtEnabled ? [this.crtFilter] : [];
    if (!settings.audioEnabled) {
      this.audio.toggleMute();
    }
    this.audio.setVolume(settings.volume);

    // Update CRT shader + menu background + particles each frame
    this.app.ticker.add((ticker) => {
      const dt = ticker.deltaTime;
      this.crtFilter.update(dt);
      this.menuBackground.update(dt);
      this.particleManager.update(dt);

      // Screen shake
      if (this.shakeDecay > 0) {
        this.shakeDecay -= dt * (1 / 60);
        const amp = this.shakeIntensity * Math.max(0, this.shakeDecay / 0.15);
        this.app.stage.x = (Math.random() - 0.5) * amp * 4;
        this.app.stage.y = (Math.random() - 0.5) * amp * 4;
      } else {
        if (this.app.stage.x !== 0 || this.app.stage.y !== 0) {
          this.app.stage.x = 0;
          this.app.stage.y = 0;
        }
      }
    });

    // Wire CRT toggle from InputManager
    this.inputManager.onCRTToggle = () => {
      this.crtEnabled = !this.crtEnabled;
      this.app.stage.filters = this.crtEnabled ? [this.crtFilter] : [];
      this.mainMenuView.setCRTState(this.crtEnabled);
    };
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
    canvas.style.width = `${Math.floor(WORLD_WIDTH * scale)}px`;
    canvas.style.height = `${Math.floor(WORLD_HEIGHT * scale)}px`;
  }

  private setViewForPhase(phase: string) {
    if (phase === "MainMenu") {
      this.mainMenuView.visible = true;
      this.menuBackground.visible = true;
      this.menuBackground.start();
      this.strategicView.visible = false;
      this.tacticalView.visible = false;
      this.hud.visible = false;
    } else if (phase === "Strategic") {
      this.mainMenuView.visible = false;
      this.menuBackground.visible = false;
      this.menuBackground.stop();
      this.strategicView.visible = true;
      this.tacticalView.visible = false;
      this.hud.visible = false;
    } else {
      this.mainMenuView.visible = false;
      this.menuBackground.visible = false;
      this.menuBackground.stop();
      this.strategicView.visible = false;
      this.tacticalView.visible = true;
      this.hud.visible = true;
    }
  }

  private updateBatteryHUD(snapshot: StateSnapshot) {
    const batteries = snapshot.entities
      .filter((e) => e.entity_type === "Battery")
      .sort((a, b) => a.x - b.x); // Sort by x: left (id=0) first

    const selectedBat = this.inputManager.selectedBattery;
    if (selectedBat < batteries.length) {
      const bat = batteries[selectedBat];
      if (bat.extra && "Battery" in bat.extra) {
        const batData = (
          bat.extra as { Battery: { ammo: number; max_ammo: number } }
        ).Battery;
        this.hud.updateBatterySelection(
          selectedBat,
          batData.ammo,
          batData.max_ammo,
          this.inputManager.selectedType
        );
      }
    }
  }
}
