import { Application } from "pixi.js";
import {
  launchInterceptor,
  predictArc,
  startWave,
  continueToStrategic,
  expandRegion,
  placeBattery,
  restockBattery,
  repairCity,
  unlockInterceptor,
  upgradeInterceptor,
  saveGame,
  loadGame,
} from "../bridge/commands";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { ArcPrediction } from "../types/commands";
import type { CampaignSnapshot, AvailableAction } from "../types/campaign";

const GROUND_Y = 50;
const ARC_THROTTLE_MS = 67; // ~15Hz

const TYPE_KEYS: Record<string, string> = {
  q: "Standard",
  w: "Sprint",
  e: "Exoatmospheric",
  r: "AreaDenial",
};

interface BatteryPosition {
  x: number;
  y: number;
}

export class InputManager {
  private app: Application;
  private worldWidth: number;
  private worldHeight: number;
  private currentPhase: string = "Strategic";
  private _selectedBattery: number = 0;
  private _selectedType: string = "Standard";
  private unlockedTypes: string[] = ["Standard"];
  private currentWindX: number = 0;
  private lastArcRequest: number = 0;
  /** Dynamic battery positions from campaign state */
  private batteryPositions: BatteryPosition[] = [
    { x: 160, y: GROUND_Y },
    { x: 1120, y: GROUND_Y },
  ];

  /** Callback invoked with arc prediction results (or null to clear). */
  onArcUpdate: ((prediction: ArcPrediction | null) => void) | null = null;

  /** Callback invoked when selected battery changes. */
  onBatteryChange: ((batteryId: number) => void) | null = null;

  /** Callback invoked when selected interceptor type changes. */
  onTypeChange: ((typeName: string) => void) | null = null;

  /** Callback invoked when CRT filter is toggled. */
  onCRTToggle: (() => void) | null = null;

  /** Callback invoked on interceptor launch (for audio). */
  onLaunchSound: ((worldX: number) => void) | null = null;

  /** Callback invoked when mute is toggled. */
  onMuteToggle: (() => void) | null = null;

  constructor(app: Application, worldWidth: number, worldHeight: number) {
    this.app = app;
    this.worldWidth = worldWidth;
    this.worldHeight = worldHeight;

    this.app.canvas.addEventListener("click", (e: MouseEvent) => {
      this.handleClick(e);
    });

    this.app.canvas.addEventListener("mousemove", (e: MouseEvent) => {
      this.handleMouseMove(e);
    });

    this.app.canvas.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      this.onArcUpdate?.(null);
    });

    document.addEventListener("keydown", (e: KeyboardEvent) => {
      this.handleKeyDown(e);
    });
  }

  get selectedBattery(): number {
    return this._selectedBattery;
  }

  get selectedType(): string {
    return this._selectedType;
  }

  setWindX(windX: number) {
    this.currentWindX = windX;
  }

  setPhase(phase: string) {
    const prev = this.currentPhase;
    this.currentPhase = phase;
    // Clear arc overlay when leaving WaveActive
    if (prev === "WaveActive" && phase !== "WaveActive") {
      this.onArcUpdate?.(null);
    }
  }

  /** Update battery positions and tech tree from campaign state */
  updateBatteryPositions(campaign: CampaignSnapshot) {
    const positions: BatteryPosition[] = [];
    for (const region of campaign.regions) {
      if (!region.owned) continue;
      for (const slot of region.battery_slots) {
        if (slot.occupied) {
          positions.push({ x: slot.x, y: slot.y });
        }
      }
    }
    if (positions.length > 0) {
      this.batteryPositions = positions.sort((a, b) => a.x - b.x);
    }
    // Clamp selected battery
    if (this._selectedBattery >= this.batteryPositions.length) {
      this._selectedBattery = 0;
    }
    // Update unlocked types from tech tree
    if (campaign.tech_tree) {
      this.unlockedTypes = campaign.tech_tree.unlocked_types;
      // If current selected type is no longer unlocked, reset to Standard
      if (!this.unlockedTypes.includes(this._selectedType)) {
        this._selectedType = "Standard";
        this.onTypeChange?.(this._selectedType);
      }
    }
  }

  /** Handle a strategic action click from the StrategicView */
  handleStrategicAction(action: AvailableAction) {
    if (this.currentPhase !== "Strategic") return;

    if (action === "StartWave") {
      startWave();
    } else if ("ExpandRegion" in action) {
      expandRegion(action.ExpandRegion.region_id);
    } else if ("PlaceBattery" in action) {
      placeBattery(
        action.PlaceBattery.region_id,
        action.PlaceBattery.slot_index
      );
    } else if ("RestockBattery" in action) {
      // Find battery index from region/slot info
      restockBattery(this.findBatteryIndex(action.RestockBattery.region_id, action.RestockBattery.slot_index));
    } else if ("RepairCity" in action) {
      repairCity(this.findCityIndex(action.RepairCity.region_id, action.RepairCity.city_index));
    } else if ("UnlockInterceptor" in action) {
      unlockInterceptor(action.UnlockInterceptor.interceptor_type);
    } else if ("UpgradeInterceptor" in action) {
      upgradeInterceptor(action.UpgradeInterceptor.interceptor_type, action.UpgradeInterceptor.axis);
    }
  }

  private findBatteryIndex(_regionId: number, _slotIndex: number): number {
    // For now, use the global battery index (batteries are sorted by x)
    // This matches the backend's battery_ids ordering
    // TODO: more precise mapping if needed
    return 0;
  }

  private findCityIndex(_regionId: number, _cityIndex: number): number {
    return 0;
  }

  private screenToWorld(e: MouseEvent): { worldX: number; worldY: number } {
    const rect = this.app.canvas.getBoundingClientRect();
    // Map from CSS-scaled canvas coordinates to game coordinates
    const gameX = ((e.clientX - rect.left) / rect.width) * this.worldWidth;
    const gameY = ((e.clientY - rect.top) / rect.height) * this.worldHeight;
    return {
      worldX: gameX,
      worldY: this.worldHeight - gameY,
    };
  }

  /** Pick the nearest battery to a world-X position. */
  private nearestBattery(worldX: number): number {
    let bestIdx = 0;
    let bestDist = Infinity;
    for (let i = 0; i < this.batteryPositions.length; i++) {
      const d = Math.abs(worldX - this.batteryPositions[i].x);
      if (d < bestDist) {
        bestDist = d;
        bestIdx = i;
      }
    }
    return bestIdx;
  }

  private handleClick(e: MouseEvent) {
    const { worldX, worldY } = this.screenToWorld(e);

    if (this.currentPhase === "WaveActive") {
      if (worldY > GROUND_Y + 20) {
        // Auto-select nearest battery on click
        this.selectBattery(this.nearestBattery(worldX));
        launchInterceptor(this._selectedBattery, worldX, worldY, this._selectedType);
        this.onLaunchSound?.(this.batteryPositions[this._selectedBattery]?.x ?? worldX);
      }
    } else if (this.currentPhase === "WaveResult") {
      continueToStrategic();
    }
    // Strategic phase clicks handled by StrategicView actions
  }

  private handleMouseMove(e: MouseEvent) {
    if (this.currentPhase !== "WaveActive") return;

    const { worldX, worldY } = this.screenToWorld(e);

    if (worldY <= GROUND_Y + 20) {
      this.onArcUpdate?.(null);
      return;
    }

    // Auto-select nearest battery as mouse moves
    this.selectBattery(this.nearestBattery(worldX));

    const now = Date.now();
    if (now - this.lastArcRequest < ARC_THROTTLE_MS) return;
    this.lastArcRequest = now;

    const bat = this.batteryPositions[this._selectedBattery];
    if (!bat) return;

    predictArc(bat.x, bat.y, worldX, worldY, this._selectedType, this.currentWindX)
      .then((prediction) => {
        this.onArcUpdate?.(prediction);
      })
      .catch(() => {
        // Silently ignore prediction errors (e.g. during phase transitions)
      });
  }

  private handleKeyDown(e: KeyboardEvent) {
    switch (e.key) {
      case "Tab":
        e.preventDefault();
        if (this.batteryPositions.length > 1) {
          this._selectedBattery =
            (this._selectedBattery + 1) % this.batteryPositions.length;
          this.onBatteryChange?.(this._selectedBattery);
          this.lastArcRequest = 0;
        }
        break;
      case "1":
        if (this.batteryPositions.length > 0) this.selectBattery(0);
        break;
      case "2":
        if (this.batteryPositions.length > 1) this.selectBattery(1);
        break;
      case "3":
        if (this.batteryPositions.length > 2) this.selectBattery(2);
        break;
      case "Enter":
        if (this.currentPhase === "WaveResult") {
          continueToStrategic();
        } else if (this.currentPhase === "Strategic") {
          startWave();
        }
        break;
      case "F5":
        e.preventDefault();
        if (this.currentPhase === "Strategic") {
          saveGame("quicksave");
        }
        break;
      case "F9":
        e.preventDefault();
        if (this.currentPhase === "Strategic") {
          loadGame("quicksave");
        }
        break;
      case "c":
        this.onCRTToggle?.();
        break;
      case "m":
        this.onMuteToggle?.();
        break;
      case "F11":
        e.preventDefault();
        this.toggleFullscreen();
        break;
      default: {
        // Q/W/E/R select interceptor type (only if unlocked)
        const typeName = TYPE_KEYS[e.key.toLowerCase()];
        if (typeName && this.unlockedTypes.includes(typeName)) {
          this._selectedType = typeName;
          this.onTypeChange?.(typeName);
          this.lastArcRequest = 0; // Force arc refresh
        }
        break;
      }
    }
  }

  private async toggleFullscreen() {
    const win = getCurrentWindow();
    const isFullscreen = await win.isFullscreen();
    await win.setFullscreen(!isFullscreen);
  }

  private selectBattery(id: number) {
    if (this._selectedBattery !== id && id < this.batteryPositions.length) {
      this._selectedBattery = id;
      this.onBatteryChange?.(this._selectedBattery);
      this.lastArcRequest = 0;
    }
  }
}
