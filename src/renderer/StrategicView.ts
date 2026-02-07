import { Application, Container, Graphics, Text, TextStyle } from "pixi.js";
import type {
  CampaignSnapshot,
  RegionSnapshot,
  AvailableAction,
} from "../types/campaign";
import { NeonButton } from "./ui/NeonButton";
import {
  NEON_CYAN, NEON_GREEN, SOLAR_YELLOW, DIM_TEXT,
  HOT_PINK, PANEL_DARK, FONT_FAMILY,
} from "./ui/Theme";

const WORLD_WIDTH = 1280;
const WORLD_HEIGHT = 720;

const REGION_RADIUS = 60;

interface RegionVisual {
  container: Container;
  circle: Graphics;
  label: Text;
  details: Text;
}

export class StrategicView {
  private container: Container;
  private regionsLayer: Container; // Bottom layer: adjacency lines + region circles
  private uiLayer: Container;     // Top layer: panel, actions, HUD text, intel
  private regionVisuals: Map<number, RegionVisual> = new Map();
  private adjacencyLines: Graphics;
  private titleText: Text;
  private resourceText: Text;
  private waveText: Text;
  private incomeText: Text;
  private actionsPanelBg: Graphics;
  private actionsContainer: Container;
  private actionButtons: NeonButton[] = [];
  private actionTickables: NeonButton[] = [];
  private intelText: Text;
  private snapshot: CampaignSnapshot | null = null;

  /** Callback when user clicks an action */
  onActionClick:
    | ((action: AvailableAction, index: number) => void)
    | null = null;

  constructor(app: Application) {
    this.container = new Container();
    this.container.visible = false;
    app.stage.addChild(this.container);

    // Bottom layer: regions + adjacency lines (renders behind everything)
    this.regionsLayer = new Container();
    this.container.addChild(this.regionsLayer);

    // Top layer: UI panel, actions, HUD text (always above regions)
    this.uiLayer = new Container();
    this.container.addChild(this.uiLayer);

    // Adjacency lines (in regions layer, behind region circles)
    this.adjacencyLines = new Graphics();
    this.regionsLayer.addChild(this.adjacencyLines);

    // Title
    this.titleText = new Text({
      text: "STRATEGIC COMMAND",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 22,
        fill: NEON_CYAN,
        fontWeight: "bold",
      }),
    });
    this.titleText.anchor.set(0.5, 0);
    this.titleText.x = WORLD_WIDTH / 2;
    this.titleText.y = 12;
    this.uiLayer.addChild(this.titleText);

    // Resources display
    this.resourceText = new Text({
      text: "RESOURCES: ---",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 16,
        fill: SOLAR_YELLOW,
      }),
    });
    this.resourceText.x = 20;
    this.resourceText.y = 12;
    this.uiLayer.addChild(this.resourceText);

    // Wave number
    this.waveText = new Text({
      text: "",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 16,
        fill: NEON_CYAN,
      }),
    });
    this.waveText.anchor.set(1, 0);
    this.waveText.x = WORLD_WIDTH - 20;
    this.waveText.y = 12;
    this.uiLayer.addChild(this.waveText);

    // Wave income notification
    this.incomeText = new Text({
      text: "",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 14,
        fill: SOLAR_YELLOW,
      }),
    });
    this.incomeText.x = 20;
    this.incomeText.y = 34;
    this.uiLayer.addChild(this.incomeText);

    // Actions panel background (dark panel on right side)
    const panelX = WORLD_WIDTH - 370;
    const panelWidth = 360;
    this.actionsPanelBg = new Graphics();
    this.actionsPanelBg.rect(panelX - 10, 55, panelWidth + 20, WORLD_HEIGHT - 100);
    this.actionsPanelBg.fill({ color: PANEL_DARK, alpha: 0.92 });
    this.actionsPanelBg.setStrokeStyle({ width: 1, color: NEON_CYAN, alpha: 0.15 });
    this.actionsPanelBg.rect(panelX - 10, 55, panelWidth + 20, WORLD_HEIGHT - 100);
    this.actionsPanelBg.stroke();
    this.uiLayer.addChild(this.actionsPanelBg);

    // Actions panel (right side, on top of background)
    this.actionsContainer = new Container();
    this.actionsContainer.x = panelX;
    this.actionsContainer.y = 80;
    this.uiLayer.addChild(this.actionsContainer);

    // Intel briefing (bottom)
    this.intelText = new Text({
      text: "",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 13,
        fill: DIM_TEXT,
        wordWrap: true,
        wordWrapWidth: WORLD_WIDTH - 40,
      }),
    });
    this.intelText.anchor.set(0, 1);
    this.intelText.x = 20;
    this.intelText.y = WORLD_HEIGHT - 16;
    this.uiLayer.addChild(this.intelText);

    // Ticker for button animations
    app.ticker.add((ticker) => {
      if (this.container.visible) {
        for (const btn of this.actionTickables) {
          btn.tick(ticker.deltaTime);
        }
      }
    });
  }

  get visible(): boolean {
    return this.container.visible;
  }

  set visible(v: boolean) {
    this.container.visible = v;
  }

  update(snapshot: CampaignSnapshot) {
    this.snapshot = snapshot;

    this.resourceText.text = `RESOURCES: $${snapshot.resources}`;
    this.waveText.text =
      snapshot.wave_number > 0
        ? `WAVES SURVIVED: ${snapshot.wave_number}`
        : "WAVE: FIRST DEPLOYMENT";

    if (snapshot.wave_income != null) {
      this.incomeText.text = `+${snapshot.wave_income} INCOME FROM SURVIVING CITIES`;
    } else {
      this.incomeText.text = "";
    }

    // Draw adjacency lines
    this.drawAdjacencyLines(snapshot.regions);

    // Update region visuals
    for (const region of snapshot.regions) {
      let visual = this.regionVisuals.get(region.id);
      if (!visual) {
        visual = this.createRegionVisual(region);
        this.regionVisuals.set(region.id, visual);
        this.regionsLayer.addChild(visual.container);
      }
      this.updateRegionVisual(visual, region);
    }

    // Update actions panel
    this.updateActions(snapshot.available_actions, snapshot.resources);

    // Intel briefing
    const owned = snapshot.regions.filter((r) => r.owned);
    const totalCities = owned.reduce((s, r) => s + r.cities.length, 0);
    const totalBatteries = owned.reduce(
      (s, r) => s + r.battery_slots.filter((b) => b.occupied).length,
      0
    );
    const emptySlots = owned.reduce(
      (s, r) => s + r.battery_slots.filter((b) => !b.occupied).length,
      0
    );
    this.intelText.text =
      `INTEL: ${owned.length} regions secured | ${totalCities} cities | ` +
      `${totalBatteries} batteries deployed | ${emptySlots} open slots | ` +
      `ENTER=Start Wave | F5=Quick Save | F9=Quick Load`;
  }

  private drawAdjacencyLines(regions: RegionSnapshot[]) {
    this.adjacencyLines.clear();

    const regionMap = new Map(regions.map((r) => [r.id, r]));

    const drawn = new Set<string>();
    for (const region of regions) {
      if (!region.owned && !region.expandable) continue;
      for (const other of regions) {
        if (other.id === region.id) continue;
        if (!other.owned && !other.expandable) continue;
        const key =
          Math.min(region.id, other.id) + "-" + Math.max(region.id, other.id);
        if (drawn.has(key)) continue;

        const isAdjacent = this.areAdjacent(region, other, regionMap);
        if (!isAdjacent) continue;

        drawn.add(key);
        const color = region.owned && other.owned ? NEON_CYAN : DIM_TEXT;
        const alpha = region.owned && other.owned ? 0.4 : 0.3;
        this.adjacencyLines.setStrokeStyle({
          width: 1,
          color,
          alpha,
        });
        this.adjacencyLines.moveTo(region.map_x, region.map_y);
        this.adjacencyLines.lineTo(other.map_x, other.map_y);
        this.adjacencyLines.stroke();
      }
    }
  }

  private areAdjacent(
    a: RegionSnapshot,
    b: RegionSnapshot,
    _regionMap: Map<number, RegionSnapshot>
  ): boolean {
    const dx = a.map_x - b.map_x;
    const dy = a.map_y - b.map_y;
    const dist = Math.sqrt(dx * dx + dy * dy);
    return dist < 400;
  }

  private createRegionVisual(region: RegionSnapshot): RegionVisual {
    const container = new Container();
    const circle = new Graphics();
    container.addChild(circle);

    const label = new Text({
      text: region.name,
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 12,
        fill: NEON_CYAN,
        fontWeight: "bold",
        align: "center",
      }),
    });
    label.anchor.set(0.5);
    label.x = region.map_x;
    label.y = region.map_y - REGION_RADIUS - 14;
    container.addChild(label);

    const details = new Text({
      text: "",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 10,
        fill: NEON_CYAN,
        align: "center",
      }),
    });
    details.anchor.set(0.5);
    details.x = region.map_x;
    details.y = region.map_y;
    container.addChild(details);

    return { container, circle, label, details };
  }

  private updateRegionVisual(visual: RegionVisual, region: RegionSnapshot) {
    const g = visual.circle;
    g.clear();

    if (region.owned) {
      // Owned region: neon cyan border, dark fill
      g.circle(region.map_x, region.map_y, REGION_RADIUS);
      g.fill({ color: PANEL_DARK, alpha: 0.8 });

      // Glow border
      g.setStrokeStyle({ width: 3, color: NEON_CYAN, alpha: 0.15 });
      g.circle(region.map_x, region.map_y, REGION_RADIUS + 2);
      g.stroke();
      g.setStrokeStyle({ width: 2, color: NEON_CYAN, alpha: 0.9 });
      g.circle(region.map_x, region.map_y, REGION_RADIUS);
      g.stroke();

      // City dots inside
      const cityCount = region.cities.length;
      for (let i = 0; i < cityCount; i++) {
        const angle = ((2 * Math.PI) / Math.max(cityCount, 1)) * i - Math.PI / 2;
        const cx = region.map_x + Math.cos(angle) * 25;
        const cy = region.map_y + Math.sin(angle) * 25;
        const healthRatio =
          region.cities[i].health / region.cities[i].max_health;
        const color = healthRatio > 0.6 ? NEON_GREEN : healthRatio > 0.3 ? SOLAR_YELLOW : HOT_PINK;
        g.circle(cx, cy, 5);
        g.fill(color);
      }

      const occupied = region.battery_slots.filter((b) => b.occupied).length;
      const total = region.battery_slots.length;
      visual.details.text = `${region.terrain}\n${region.cities.length}C ${occupied}/${total}B`;
      visual.details.y = region.map_y + 10;
      visual.details.style.fill = NEON_CYAN;
      visual.label.style.fill = NEON_CYAN;
    } else if (region.expandable) {
      // Expandable: yellow pulsing border
      g.circle(region.map_x, region.map_y, REGION_RADIUS);
      g.fill({ color: PANEL_DARK, alpha: 0.4 });
      g.setStrokeStyle({ width: 1, color: SOLAR_YELLOW, alpha: 0.6 });
      g.circle(region.map_x, region.map_y, REGION_RADIUS);
      g.stroke();

      visual.details.text = `${region.terrain}\nCOST: $${region.expansion_cost}`;
      visual.details.y = region.map_y;
      visual.details.style.fill = SOLAR_YELLOW;
      visual.label.style.fill = SOLAR_YELLOW;
    } else {
      // Unknown region: very dim
      g.circle(region.map_x, region.map_y, REGION_RADIUS * 0.7);
      g.fill({ color: PANEL_DARK, alpha: 0.2 });
      g.setStrokeStyle({ width: 1, color: DIM_TEXT, alpha: 0.3 });
      g.circle(region.map_x, region.map_y, REGION_RADIUS * 0.7);
      g.stroke();

      visual.details.text = "???";
      visual.details.y = region.map_y;
      visual.details.style.fill = DIM_TEXT;
      visual.label.style.fill = DIM_TEXT;
    }
  }

  private updateActions(actions: AvailableAction[], resources: number) {
    // Clear old buttons
    for (const btn of this.actionButtons) {
      this.actionsContainer.removeChild(btn);
      btn.destroy({ children: true });
    }
    this.actionButtons = [];
    this.actionTickables = [];

    // Section header
    const headerStyle = new TextStyle({
      fontFamily: FONT_FAMILY,
      fontSize: 14,
      fill: NEON_CYAN,
      fontWeight: "bold",
    });

    const header = new Text({ text: "AVAILABLE ACTIONS", style: headerStyle });
    header.y = 0;
    // Header is a Text, not a button, add directly
    const headerContainer = new Container();
    headerContainer.addChild(header);
    this.actionsContainer.addChild(headerContainer as unknown as NeonButton);
    // We'll track it separately since it's not a NeonButton
    this.actionButtons.push(headerContainer as unknown as NeonButton);

    let y = 30;
    let actionIndex = 0;
    for (const action of actions) {
      const { label, cost } = this.formatAction(action);
      const affordable = cost === 0 || resources >= cost;

      const isStartWave = action === "StartWave";
      const btnColor = isStartWave ? NEON_GREEN : affordable ? NEON_CYAN : HOT_PINK;

      const btn = new NeonButton({
        label,
        width: 340,
        height: isStartWave ? 40 : 32,
        fontSize: isStartWave ? 16 : 12,
        color: btnColor,
        disabled: !affordable,
        onClick: () => {
          this.onActionClick?.(action, idx);
        },
      });

      const idx = actionIndex;
      btn.x = 170; // Center in 340px width
      btn.y = y + (isStartWave ? 20 : 16);
      this.actionsContainer.addChild(btn);
      this.actionButtons.push(btn);
      this.actionTickables.push(btn);
      y += isStartWave ? 48 : 36;
      actionIndex++;
    }

    // Resize panel background to fit content
    const panelX = WORLD_WIDTH - 370;
    const panelWidth = 360;
    const panelHeight = y + 40;
    this.actionsPanelBg.clear();
    this.actionsPanelBg.rect(panelX - 10, 55, panelWidth + 20, panelHeight);
    this.actionsPanelBg.fill({ color: PANEL_DARK, alpha: 0.92 });
    this.actionsPanelBg.setStrokeStyle({ width: 1, color: NEON_CYAN, alpha: 0.15 });
    this.actionsPanelBg.rect(panelX - 10, 55, panelWidth + 20, panelHeight);
    this.actionsPanelBg.stroke();
  }

  private formatAction(action: AvailableAction): {
    label: string;
    cost: number;
  } {
    if (action === "StartWave") {
      return { label: "START WAVE", cost: 0 };
    }
    if ("ExpandRegion" in action) {
      const { region_id, cost } = action.ExpandRegion;
      const name =
        this.snapshot?.regions.find((r) => r.id === region_id)?.name ??
        `Region ${region_id}`;
      return { label: `EXPAND: ${name} ($${cost})`, cost };
    }
    if ("PlaceBattery" in action) {
      const { region_id, cost } = action.PlaceBattery;
      const name =
        this.snapshot?.regions.find((r) => r.id === region_id)?.name ??
        `Region ${region_id}`;
      return { label: `PLACE BATTERY: ${name} ($${cost})`, cost };
    }
    if ("RestockBattery" in action) {
      const { region_id, cost } = action.RestockBattery;
      const name =
        this.snapshot?.regions.find((r) => r.id === region_id)?.name ??
        `Region ${region_id}`;
      return { label: `RESTOCK: ${name} ($${cost})`, cost };
    }
    if ("RepairCity" in action) {
      const { region_id, cost } = action.RepairCity;
      const name =
        this.snapshot?.regions.find((r) => r.id === region_id)?.name ??
        `Region ${region_id}`;
      return { label: `REPAIR CITY: ${name} ($${cost})`, cost };
    }
    if ("UnlockInterceptor" in action) {
      const { interceptor_type, cost } = action.UnlockInterceptor;
      return { label: `UNLOCK: ${interceptor_type} ($${cost})`, cost };
    }
    if ("UpgradeInterceptor" in action) {
      const { interceptor_type, axis, cost, current_level } = action.UpgradeInterceptor;
      return { label: `UPGRADE: ${interceptor_type} ${axis} Lv${current_level + 1} ($${cost})`, cost };
    }
    return { label: "UNKNOWN", cost: 0 };
  }

  /** Get the region at a map position (for click handling) */
  getRegionAt(x: number, y: number): RegionSnapshot | null {
    if (!this.snapshot) return null;
    for (const region of this.snapshot.regions) {
      const dx = x - region.map_x;
      const dy = y - region.map_y;
      if (dx * dx + dy * dy <= REGION_RADIUS * REGION_RADIUS) {
        return region;
      }
    }
    return null;
  }
}
