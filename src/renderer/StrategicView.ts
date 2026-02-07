import { Application, Container, Graphics, Text, TextStyle } from "pixi.js";
import type { CampaignSnapshot, RegionSnapshot } from "../types/campaign";
import {
  NEON_CYAN, NEON_GREEN, SOLAR_YELLOW, DIM_TEXT,
  HOT_PINK, PANEL_DARK, FONT_FAMILY,
} from "./ui/Theme";

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
  private regionVisuals: Map<number, RegionVisual> = new Map();
  private adjacencyLines: Graphics;
  private snapshot: CampaignSnapshot | null = null;

  constructor(app: Application) {
    this.container = new Container();
    this.container.visible = false;
    app.stage.addChild(this.container);

    // Bottom layer: regions + adjacency lines (renders behind everything)
    this.regionsLayer = new Container();
    this.container.addChild(this.regionsLayer);

    // Adjacency lines (in regions layer, behind region circles)
    this.adjacencyLines = new Graphics();
    this.regionsLayer.addChild(this.adjacencyLines);
  }

  get visible(): boolean {
    return this.container.visible;
  }

  set visible(v: boolean) {
    this.container.visible = v;
  }

  update(snapshot: CampaignSnapshot) {
    this.snapshot = snapshot;

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
