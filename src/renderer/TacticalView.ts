import { Application, Container, Graphics, Text, TextStyle } from "pixi.js";
import type { StateSnapshot, EntitySnapshot, EntityExtra } from "../types/snapshot";
import type { ArcPrediction } from "../types/commands";
import {
  NEON_CYAN, HOT_PINK, NEON_GREEN, NEON_ORANGE, SOLAR_YELLOW,
  MISSILE_BODY, MISSILE_CORE, MIRV_BODY, MIRV_CORE,
  MISSILE_TRAIL_DIM, TYPE_COLORS, TYPE_TRAIL_DIM, FONT_FAMILY,
} from "./ui/Theme";

/** Convert world coordinates (Y-up) to screen coordinates (Y-down) */
function worldToScreen(worldY: number, worldHeight: number): number {
  return worldHeight - worldY;
}

// Battery world-X positions (must match config::BATTERY_POSITIONS)
const BATTERY_X = [160, 1120];
const GROUND_Y = 50;

interface MirvSplitEffect {
  x: number;
  y: number;
  radius: number;
  maxRadius: number;
  alpha: number;
}

interface EntityVisual {
  container: Container;
  graphic: Graphics;
  trail?: Graphics;
  trailPoints: { x: number; y: number }[];
}

export class TacticalView {
  private worldHeight: number;
  private worldWidth: number;
  private stage: Container;
  private groundLine: Graphics;
  private entityVisuals: Map<number, EntityVisual> = new Map();

  // Arc overlay layer
  private arcOverlay: Graphics;
  private timeLabel: Text;
  private batteryHighlight: Graphics;
  private mirvEffects: MirvSplitEffect[] = [];
  private mirvEffectGraphics: Graphics;

  get visible(): boolean {
    return this.stage.visible;
  }

  set visible(v: boolean) {
    this.stage.visible = v;
  }

  constructor(app: Application, worldWidth: number, worldHeight: number) {
    this.worldWidth = worldWidth;
    this.worldHeight = worldHeight;

    this.stage = new Container();
    app.stage.addChild(this.stage);

    // Draw ground line
    this.groundLine = new Graphics();
    this.drawGround();
    this.stage.addChild(this.groundLine);

    // Arc prediction overlay (rendered above entities)
    this.arcOverlay = new Graphics();
    this.stage.addChild(this.arcOverlay);

    // Battery selection highlight (rendered below entities but above ground)
    this.batteryHighlight = new Graphics();
    this.stage.addChild(this.batteryHighlight);

    // MIRV split effects layer
    this.mirvEffectGraphics = new Graphics();
    this.stage.addChild(this.mirvEffectGraphics);

    // Time-to-intercept label
    this.timeLabel = new Text({
      text: "",
      style: new TextStyle({
        fontFamily: FONT_FAMILY,
        fontSize: 12,
        fill: NEON_CYAN,
      }),
    });
    this.timeLabel.visible = false;
    this.stage.addChild(this.timeLabel);
  }

  private drawGround() {
    const groundScreenY = worldToScreen(GROUND_Y, this.worldHeight);
    this.groundLine.clear();

    // Glow line behind (wider, dimmer)
    this.groundLine.setStrokeStyle({ width: 4, color: NEON_CYAN, alpha: 0.15 });
    this.groundLine.moveTo(0, groundScreenY);
    this.groundLine.lineTo(this.worldWidth, groundScreenY);
    this.groundLine.stroke();

    // Sharp ground line
    this.groundLine.setStrokeStyle({ width: 2, color: NEON_CYAN });
    this.groundLine.moveTo(0, groundScreenY);
    this.groundLine.lineTo(this.worldWidth, groundScreenY);
    this.groundLine.stroke();
  }

  update(snapshot: StateSnapshot) {
    const activeIds = new Set<number>();

    for (const entity of snapshot.entities) {
      activeIds.add(entity.id);

      let visual = this.entityVisuals.get(entity.id);
      if (!visual) {
        visual = this.createVisual(entity);
        this.entityVisuals.set(entity.id, visual);
        this.stage.addChild(visual.container);
      }

      this.updateVisual(visual, entity);
    }

    // Remove visuals for entities no longer in snapshot
    for (const [id, visual] of this.entityVisuals) {
      if (!activeIds.has(id)) {
        this.stage.removeChild(visual.container);
        visual.container.destroy({ children: true });
        this.entityVisuals.delete(id);
      }
    }

    // Tick MIRV split effects
    this.updateMirvEffects();
  }

  updateArcOverlay(prediction: ArcPrediction | null) {
    this.arcOverlay.clear();
    this.timeLabel.visible = false;

    if (!prediction || prediction.points.length < 2) return;

    const color = prediction.reaches_target ? NEON_CYAN : HOT_PINK;

    // Draw dashed arc line (alternating draw/skip every 3 segments)
    for (let i = 1; i < prediction.points.length; i++) {
      // Skip every other group of 3 for dashed effect
      if (Math.floor(i / 3) % 2 === 1) continue;

      const [, wy0] = prediction.points[i - 1];
      const [, wy1] = prediction.points[i];
      const sx0 = prediction.points[i - 1][0];
      const sy0 = worldToScreen(wy0, this.worldHeight);
      const sx1 = prediction.points[i][0];
      const sy1 = worldToScreen(wy1, this.worldHeight);

      this.arcOverlay.setStrokeStyle({ width: 1.5, color, alpha: 0.7 });
      this.arcOverlay.moveTo(sx0, sy0);
      this.arcOverlay.lineTo(sx1, sy1);
      this.arcOverlay.stroke();
    }

    // Target crosshair at endpoint
    const lastPt = prediction.points[prediction.points.length - 1];
    const tx = lastPt[0];
    const ty = worldToScreen(lastPt[1], this.worldHeight);
    const crossSize = 8;

    this.arcOverlay.setStrokeStyle({ width: 1, color, alpha: 0.6 });
    this.arcOverlay.moveTo(tx - crossSize, ty);
    this.arcOverlay.lineTo(tx + crossSize, ty);
    this.arcOverlay.stroke();
    this.arcOverlay.moveTo(tx, ty - crossSize);
    this.arcOverlay.lineTo(tx, ty + crossSize);
    this.arcOverlay.stroke();

    // Time label near endpoint
    this.timeLabel.text = `${prediction.time_to_target.toFixed(1)}s`;
    this.timeLabel.style.fill = color;
    this.timeLabel.x = tx + 12;
    this.timeLabel.y = ty - 14;
    this.timeLabel.visible = true;
  }

  updateBatteryHighlight(selectedBatteryId: number, batteryWorldX?: number) {
    this.batteryHighlight.clear();
    const bx = batteryWorldX ?? BATTERY_X[selectedBatteryId] ?? BATTERY_X[0];
    const sy = worldToScreen(GROUND_Y, this.worldHeight);

    // Glow ring (outer, dimmer)
    this.batteryHighlight.setStrokeStyle({ width: 3, color: NEON_CYAN, alpha: 0.2 });
    this.batteryHighlight.circle(bx, sy - 4, 22);
    this.batteryHighlight.stroke();

    // Selection ring
    this.batteryHighlight.setStrokeStyle({ width: 2, color: NEON_CYAN, alpha: 0.8 });
    this.batteryHighlight.circle(bx, sy - 4, 18);
    this.batteryHighlight.stroke();
  }

  clearOverlays() {
    this.arcOverlay.clear();
    this.timeLabel.visible = false;
  }

  /** Trigger an expanding ring at a MIRV split location */
  addMirvSplitEffect(worldX: number, worldY: number) {
    const screenY = worldToScreen(worldY, this.worldHeight);
    this.mirvEffects.push({
      x: worldX,
      y: screenY,
      radius: 5,
      maxRadius: 40,
      alpha: 1.0,
    });
  }

  private updateMirvEffects() {
    this.mirvEffectGraphics.clear();
    this.mirvEffects = this.mirvEffects.filter((e) => e.alpha > 0.05);
    for (const e of this.mirvEffects) {
      e.radius += 2;
      e.alpha = Math.max(0, 1.0 - e.radius / e.maxRadius);
      // Outer ring (hot pink)
      this.mirvEffectGraphics.setStrokeStyle({ width: 2, color: HOT_PINK, alpha: e.alpha });
      this.mirvEffectGraphics.circle(e.x, e.y, e.radius);
      this.mirvEffectGraphics.stroke();
      // Inner ring (cyan)
      this.mirvEffectGraphics.setStrokeStyle({ width: 1, color: NEON_CYAN, alpha: e.alpha * 0.5 });
      this.mirvEffectGraphics.circle(e.x, e.y, e.radius * 0.6);
      this.mirvEffectGraphics.stroke();
    }
  }

  private createVisual(entity: EntitySnapshot): EntityVisual {
    const container = new Container();
    const graphic = new Graphics();
    container.addChild(graphic);

    const visual: EntityVisual = {
      container,
      graphic,
      trailPoints: [],
    };

    // Add trail for missiles and interceptors
    if (entity.entity_type === "Missile" || entity.entity_type === "Interceptor") {
      const trail = new Graphics();
      container.addChild(trail);
      // Move trail behind the dot
      container.setChildIndex(trail, 0);
      visual.trail = trail;
    }

    return visual;
  }

  private updateVisual(visual: EntityVisual, entity: EntitySnapshot) {
    const screenX = entity.x;
    const screenY = worldToScreen(entity.y, this.worldHeight);

    visual.container.x = 0;
    visual.container.y = 0;

    // Update trail
    if (visual.trail && (entity.entity_type === "Missile" || entity.entity_type === "Interceptor")) {
      visual.trailPoints.push({ x: screenX, y: screenY });
      if (visual.trailPoints.length > 60) {
        visual.trailPoints.shift();
      }

      let trailColor = MISSILE_BODY;
      let trailDimColor = MISSILE_TRAIL_DIM;
      let trailWidth = 1;

      if (entity.entity_type === "Interceptor" && entity.extra && "Interceptor" in entity.extra) {
        const data = (entity.extra as { Interceptor: { burn_remaining: number; burn_time: number; interceptor_type: string } }).Interceptor;
        trailColor = TYPE_COLORS[data.interceptor_type] ?? NEON_GREEN;
        trailDimColor = TYPE_TRAIL_DIM[data.interceptor_type] ?? 0x0a3300;
        if (data.burn_remaining > 0) trailWidth = 1.5;
      } else if (entity.entity_type === "Missile" && entity.extra && "Missile" in entity.extra) {
        const data = (entity.extra as { Missile: { is_mirv: boolean; detected_by_radar: boolean; detected_by_glow: boolean } }).Missile;
        if (data.is_mirv) {
          trailColor = MIRV_BODY;
          trailDimColor = 0x330011;
        }
      }
      const isMissileTrail = entity.entity_type === "Missile";
      this.drawTrailGlow(visual.trail, visual.trailPoints, trailColor, trailDimColor, trailWidth, isMissileTrail);
    }

    // Redraw the main graphic
    visual.graphic.clear();

    switch (entity.entity_type) {
      case "Missile":
        this.drawMissile(visual.graphic, screenX, screenY, entity.extra);
        break;
      case "Interceptor":
        this.drawInterceptor(visual.graphic, screenX, screenY, entity.extra);
        break;
      case "Shockwave":
        this.drawShockwave(visual.graphic, screenX, screenY, entity.extra);
        break;
      case "City":
        this.drawCity(visual.graphic, screenX, screenY, entity.extra);
        break;
      case "Battery":
        this.drawBattery(visual.graphic, screenX, screenY, entity.extra);
        break;
    }
  }

  private drawTrailGlow(
    g: Graphics,
    points: { x: number; y: number }[],
    color: number,
    _dimColor: number,
    widthBase: number,
    smokeTrail: boolean = false,
  ) {
    g.clear();
    if (points.length < 2) return;

    // Smoke pass for missiles — soft expanding puffs along the trail
    if (smokeTrail && points.length > 3) {
      // Use a seeded pseudo-random based on point positions for consistent jitter
      for (let i = 2; i < points.length; i += 2) {
        const t = i / points.length; // 0..1, older→newer
        const age = 1 - t; // older trail points = more expanded smoke
        const radius = 3 + age * 10; // smoke expands as it ages
        const alpha = t * 0.06 * (1 - age * 0.5); // fades for older parts
        // Slight jitter offset for organic feel
        const jx = Math.sin(i * 7.3 + points[i].y * 0.1) * (2 + age * 4);
        const jy = Math.cos(i * 5.1 + points[i].x * 0.1) * (2 + age * 4);
        g.circle(points[i].x + jx, points[i].y + jy, radius);
        g.fill({ color, alpha });
      }
    }

    // Glow pass (wider, dimmer)
    for (let i = 1; i < points.length; i++) {
      const alpha = (i / points.length) * 0.2;
      g.setStrokeStyle({ width: widthBase * 3, color, alpha });
      g.moveTo(points[i - 1].x, points[i - 1].y);
      g.lineTo(points[i].x, points[i].y);
      g.stroke();
    }

    // Sharp pass
    for (let i = 1; i < points.length; i++) {
      const alpha = (i / points.length) * 0.7;
      g.setStrokeStyle({ width: widthBase, color, alpha });
      g.moveTo(points[i - 1].x, points[i - 1].y);
      g.lineTo(points[i].x, points[i].y);
      g.stroke();
    }
  }

  private drawMissile(g: Graphics, x: number, y: number, extra: EntityExtra | null) {
    let isMirv = false;
    if (extra && "Missile" in extra) {
      const data = (extra as { Missile: { is_mirv: boolean; detected_by_radar: boolean; detected_by_glow: boolean } }).Missile;
      isMirv = data.is_mirv;
    }

    if (isMirv) {
      // MIRV carrier: larger, glow + body + core
      g.circle(x, y, 10);
      g.fill({ color: MIRV_BODY, alpha: 0.12 });
      g.circle(x, y, 5);
      g.fill(MIRV_BODY);
      g.circle(x, y, 2.5);
      g.fill(MIRV_CORE);
    } else {
      // Regular missile: glow + body + core
      g.circle(x, y, 7);
      g.fill({ color: MISSILE_BODY, alpha: 0.12 });
      g.circle(x, y, 3);
      g.fill(MISSILE_BODY);
      g.circle(x, y, 1.5);
      g.fill(MISSILE_CORE);
    }
  }

  private drawInterceptor(g: Graphics, x: number, y: number, extra: EntityExtra | null) {
    let burning = false;
    let burnRatio = 0;
    let typeName = "Standard";
    if (extra && "Interceptor" in extra) {
      const data = (extra as { Interceptor: { burn_remaining: number; burn_time: number; interceptor_type: string } }).Interceptor;
      burning = data.burn_remaining > 0;
      burnRatio = data.burn_time > 0 ? data.burn_remaining / data.burn_time : 0;
      typeName = data.interceptor_type;
    }

    const typeColor = TYPE_COLORS[typeName] ?? NEON_GREEN;

    if (burning) {
      const size = 3 + burnRatio * 2;

      // Outer bloom glow
      g.circle(x, y, size * 2.5);
      g.fill({ color: typeColor, alpha: 0.06 * burnRatio });

      // Inner exhaust glow
      g.circle(x, y, size + 4);
      g.fill({ color: typeColor, alpha: 0.25 * burnRatio });

      // Body
      g.circle(x, y, size);
      g.fill(typeColor);

      // Bright core
      g.circle(x, y, size * 0.5);
      g.fill(0xffffff);
    } else {
      // Coasting — glow + body + core
      g.circle(x, y, 6);
      g.fill({ color: typeColor, alpha: 0.1 });
      g.circle(x, y, 2.5);
      g.fill({ color: typeColor, alpha: 0.8 });
      g.circle(x, y, 1);
      g.fill({ color: 0xffffff, alpha: 0.6 });
    }
  }

  private drawShockwave(g: Graphics, x: number, y: number, extra: EntityExtra | null) {
    if (!extra || !("Shockwave" in extra)) return;
    const sw = (extra as { Shockwave: { radius: number; max_radius: number } }).Shockwave;
    const progress = sw.radius / sw.max_radius;
    const alpha = Math.max(0, 1.0 - progress * 0.8);

    // Core flash (first few frames)
    if (progress < 0.1) {
      const flashAlpha = (0.1 - progress) * 5;
      g.circle(x, y, sw.radius * 0.3);
      g.fill({ color: 0xffffff, alpha: flashAlpha * 0.6 });
    }

    // Inner ring
    g.setStrokeStyle({ width: 1.5, color: NEON_ORANGE, alpha: alpha * 0.5 });
    g.circle(x, y, sw.radius * 0.6);
    g.stroke();

    // Outer ring
    g.setStrokeStyle({ width: 2, color: SOLAR_YELLOW, alpha });
    g.circle(x, y, sw.radius);
    g.stroke();

    // Inner fill
    if (alpha > 0.3) {
      g.circle(x, y, sw.radius * 0.5);
      g.fill({ color: NEON_ORANGE, alpha: alpha * 0.15 });
    }
  }

  private drawCity(g: Graphics, x: number, y: number, extra: EntityExtra | null) {
    let healthRatio = 1;
    if (extra && "City" in extra) {
      const city = (extra as { City: { health: number; max_health: number } }).City;
      healthRatio = city.health / city.max_health;
    }

    // Color transitions: green → orange → hot pink → dark
    let baseColor: number;
    let glowColor: number;
    if (healthRatio <= 0) {
      baseColor = 0x331111;
      glowColor = 0x331111;
    } else if (healthRatio > 0.6) {
      baseColor = NEON_GREEN;
      glowColor = NEON_GREEN;
    } else if (healthRatio > 0.3) {
      baseColor = NEON_ORANGE;
      glowColor = NEON_ORANGE;
    } else {
      baseColor = HOT_PINK;
      glowColor = HOT_PINK;
    }
    const alpha = healthRatio > 0 ? 0.5 + healthRatio * 0.5 : 0.3;

    // Glow outline (drawn first, behind buildings)
    if (healthRatio > 0) {
      g.rect(x - 22, y - 37, 14, 27);
      g.fill({ color: glowColor, alpha: 0.08 });
      g.rect(x - 7, y - 37, 14, 37);
      g.fill({ color: glowColor, alpha: 0.08 });
      g.rect(x + 8, y - 22, 14, 22);
      g.fill({ color: glowColor, alpha: 0.08 });
    }

    // Left building
    g.rect(x - 20, y - 25, 10, 25);
    g.fill({ color: baseColor, alpha });

    // Center building (taller)
    g.rect(x - 5, y - 35, 10, 35);
    g.fill({ color: baseColor, alpha });

    // Right building
    g.rect(x + 10, y - 20, 10, 20);
    g.fill({ color: baseColor, alpha });

    // Health bar (neon styled)
    if (healthRatio > 0 && healthRatio < 1) {
      const barWidth = 40;
      // Dark track
      g.rect(x - barWidth / 2, y + 5, barWidth, 3);
      g.fill({ color: 0x111122 });
      // Fill with gradient color
      const barColor = healthRatio > 0.6 ? NEON_GREEN : healthRatio > 0.3 ? NEON_ORANGE : HOT_PINK;
      g.rect(x - barWidth / 2, y + 5, barWidth * healthRatio, 3);
      g.fill({ color: barColor });
    }
  }

  private drawBattery(g: Graphics, x: number, y: number, extra: EntityExtra | null) {
    // Glow behind triangle
    g.circle(x, y - 6, 10);
    g.fill({ color: NEON_CYAN, alpha: 0.08 });

    // Triangle pointing up
    g.moveTo(x, y - 12);
    g.lineTo(x - 8, y);
    g.lineTo(x + 8, y);
    g.closePath();
    g.fill(NEON_CYAN);

    // Ammo display
    if (extra && "Battery" in extra) {
      const bat = (extra as { Battery: { ammo: number; max_ammo: number } }).Battery;
      for (let i = 0; i < bat.max_ammo; i++) {
        const dotX = x - 15 + (i % 5) * 7;
        const dotY = y + 8 + Math.floor(i / 5) * 7;
        g.circle(dotX, dotY, 2);
        g.fill(i < bat.ammo ? NEON_CYAN : 0x222233);
      }
    }
  }
}
