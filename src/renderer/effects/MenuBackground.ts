import { Application, Container, Graphics } from "pixi.js";
import {
  HOT_PINK, NEON_ORANGE, NEON_CYAN, NEON_GREEN, NEON_MAGENTA,
  ELECTRIC_BLUE, SOLAR_YELLOW, PANEL_BORDER, MIRV_BODY,
} from "../ui/Theme";

// ─── Color pools ──────────────────────────────────────────────────────
const MISSILE_COLORS = [HOT_PINK, 0xff0044, NEON_ORANGE];
const STAR_COLORS = [NEON_CYAN, ELECTRIC_BLUE, NEON_MAGENTA];
const GRID_SPACING = 80;

// ─── Physics (screen-space: Y-down, gravity positive = downward) ──────
const MENU_GRAVITY = 9.81;
const GROUND_Y = 670; // screen-Y (= 720 − 50)
const DETONATION_PROXIMITY = 20;
const MIRV_SPLIT_SCREEN_Y = 350 + Math.random() * 80; // ~350-430
const MIRV_SPREAD_ANGLE = 0.5;
const MIRV_CHILD_COUNT = 3;

// ─── World layout ──────────────────────────────────────────────────────
const CITY_POSITIONS = [
  { x: 320, y: GROUND_Y },
  { x: 640, y: GROUND_Y },
  { x: 960, y: GROUND_Y },
];
const BATTERY_POSITIONS = [
  { x: 160, y: GROUND_Y },
  { x: 1120, y: GROUND_Y },
];

// ─── Interceptor profiles ──────────────────────────────────────────────
const INTERCEPTOR_PROFILES = [
  { name: "Standard", thrust: 600, burn: 1.0, color: NEON_GREEN },
  { name: "Sprint", thrust: 900, burn: 0.5, color: NEON_CYAN },
  { name: "Exo", thrust: 300, burn: 2.5, color: NEON_MAGENTA },
  { name: "AreaDenial", thrust: 400, burn: 1.2, color: NEON_ORANGE },
];

// ─── Entity caps ───────────────────────────────────────────────────────
const MAX_MISSILES = 50;
const MAX_INTERCEPTORS = 60;
const MAX_EXPLOSIONS = 25;

// ─── Interfaces ────────────────────────────────────────────────────────
interface MenuMissile {
  x: number; y: number;
  vx: number; vy: number;
  isMirv: boolean;
  hasSplit: boolean;
  life: number; maxLife: number;
  color: number;
  trail: { x: number; y: number }[];
}

interface MenuInterceptor {
  x: number; y: number;
  vx: number; vy: number;
  targetX: number; targetY: number;
  thrust: number;
  burnRemaining: number;
  life: number; maxLife: number;
  color: number;
  trail: { x: number; y: number }[];
}

interface MenuExplosion {
  x: number; y: number;
  radius: number;
  maxRadius: number;
  life: number; maxLife: number;
  color: number;
}

interface Star {
  x: number; y: number;
  vx: number; vy: number;
  radius: number;
  baseAlpha: number;
  twinkleRate: number;
  color: number;
}

interface Blip {
  x: number; y: number;
  life: number; maxLife: number;
  radius: number;
  color: number;
}

export class MenuBackground {
  private container: Container;
  private backdropGfx: Graphics;
  private gridGfx: Graphics;
  private starsGfx: Graphics;
  private groundGfx: Graphics;
  private particleGfx: Graphics;
  private scanGfx: Graphics;

  private missiles: MenuMissile[] = [];
  private interceptors: MenuInterceptor[] = [];
  private explosions: MenuExplosion[] = [];
  private stars: Star[] = [];
  private blips: Blip[] = [];

  private running: boolean = false;
  private spawnTimerMissile: number = 0;
  private spawnTimerInterceptor: number = 0;
  private blipTimer: number = 0;
  private timeSec: number = 0;
  private gridOffsetX: number = 0;
  private gridOffsetY: number = 0;
  private scanY: number = 0;
  private width: number;
  private height: number;

  constructor(app: Application, width: number, height: number) {
    this.width = width;
    this.height = height;

    this.container = new Container();
    app.stage.addChild(this.container);

    // Layer order (bottom → top):
    this.backdropGfx = new Graphics();
    this.drawBackdrop();
    this.container.addChild(this.backdropGfx);

    this.starsGfx = new Graphics();
    this.container.addChild(this.starsGfx);
    this.seedStars();

    this.gridGfx = new Graphics();
    this.drawGrid();
    this.container.addChild(this.gridGfx);

    this.groundGfx = new Graphics();
    this.container.addChild(this.groundGfx);

    this.particleGfx = new Graphics();
    this.container.addChild(this.particleGfx);

    this.scanGfx = new Graphics();
    this.container.addChild(this.scanGfx);
  }

  // ─── Backdrop (static) ────────────────────────────────────────────────
  private drawBackdrop() {
    this.backdropGfx.clear();
    this.backdropGfx.rect(0, 0, this.width, this.height);
    this.backdropGfx.fill({ color: 0x050510, alpha: 1.0 });

    const r0 = Math.max(this.width, this.height) * 0.75;
    this.backdropGfx.circle(this.width * 0.5, this.height * 0.45, r0);
    this.backdropGfx.fill({ color: NEON_CYAN, alpha: 0.03 });
    this.backdropGfx.circle(this.width * 0.78, this.height * 0.72, r0 * 0.55);
    this.backdropGfx.fill({ color: NEON_MAGENTA, alpha: 0.02 });
    this.backdropGfx.circle(this.width * 0.2, this.height * 0.82, r0 * 0.45);
    this.backdropGfx.fill({ color: ELECTRIC_BLUE, alpha: 0.02 });
  }

  // ─── Grid (static geometry, animated via offset) ──────────────────────
  private drawGrid() {
    this.gridGfx.clear();
    const extra = GRID_SPACING * 2;
    for (let x = -extra; x <= this.width + extra; x += GRID_SPACING) {
      this.gridGfx.moveTo(x, -extra);
      this.gridGfx.lineTo(x, this.height + extra);
      this.gridGfx.stroke({ width: 1, color: PANEL_BORDER, alpha: 0.2 });
    }
    for (let y = -extra; y <= this.height + extra; y += GRID_SPACING) {
      this.gridGfx.moveTo(-extra, y);
      this.gridGfx.lineTo(this.width + extra, y);
      this.gridGfx.stroke({ width: 1, color: PANEL_BORDER, alpha: 0.2 });
    }
  }

  // ─── Stars ────────────────────────────────────────────────────────────
  private seedStars() {
    this.stars = [];
    for (let i = 0; i < 90; i++) {
      this.stars.push({
        x: Math.random() * this.width,
        y: Math.random() * this.height,
        vx: (Math.random() - 0.5) * 6,
        vy: 8 + Math.random() * 22,
        radius: 0.6 + Math.random() * 1.4,
        baseAlpha: 0.05 + Math.random() * 0.12,
        twinkleRate: 0.6 + Math.random() * 1.8,
        color: STAR_COLORS[Math.floor(Math.random() * STAR_COLORS.length)],
      });
    }
  }

  // ─── Lifecycle ────────────────────────────────────────────────────────
  start() {
    this.running = true;
    this.missiles = [];
    this.interceptors = [];
    this.explosions = [];
    this.blips = [];
    this.spawnTimerMissile = 0;
    this.spawnTimerInterceptor = 0;
    this.blipTimer = 0;
    this.timeSec = 0;
    this.gridOffsetX = 0;
    this.gridOffsetY = 0;
    this.scanY = Math.random() * this.height;
  }

  stop() {
    this.running = false;
    this.missiles = [];
    this.interceptors = [];
    this.explosions = [];
    this.blips = [];
    this.particleGfx.clear();
    this.starsGfx.clear();
    this.scanGfx.clear();
    this.groundGfx.clear();
  }

  // ═══════════════════════════════════════════════════════════════════════
  //  MAIN UPDATE LOOP
  // ═══════════════════════════════════════════════════════════════════════
  update(dt: number) {
    if (!this.running) return;

    const dtSec = dt * (1 / 60);
    this.timeSec += dtSec;

    // ── Spawning ──────────────────────────────────────────────────────
    this.spawnTimerMissile += dtSec;
    if (this.spawnTimerMissile > 0.6 && this.missiles.length < MAX_MISSILES) {
      this.spawnTimerMissile = 0;
      this.spawnMissile();
    }

    this.spawnTimerInterceptor += dtSec;
    if (this.spawnTimerInterceptor > 0.4 && this.interceptors.length < MAX_INTERCEPTORS) {
      this.spawnTimerInterceptor = 0;
      this.spawnInterceptor();
      if (Math.random() < 0.4) this.spawnInterceptor();
    }

    this.blipTimer -= dtSec;
    if (this.blipTimer <= 0) {
      this.spawnBlip();
      this.blipTimer = 0.35 + Math.random() * 0.9;
    }

    // ── Grid drift ────────────────────────────────────────────────────
    this.gridOffsetX = (this.gridOffsetX + dtSec * 9) % GRID_SPACING;
    this.gridOffsetY = (this.gridOffsetY + dtSec * 14) % GRID_SPACING;
    this.gridGfx.x = -this.gridOffsetX;
    this.gridGfx.y = -this.gridOffsetY;
    this.gridGfx.alpha = 0.16 + 0.04 * (0.5 + 0.5 * Math.sin(this.timeSec * 0.35));

    // ── Missile physics ───────────────────────────────────────────────
    const mirvSpawns: MenuMissile[] = [];
    for (const m of this.missiles) {
      m.life += dtSec;

      // Gravity
      m.vy += MENU_GRAVITY * dtSec;

      // Movement
      m.x += m.vx * dtSec;
      m.y += m.vy * dtSec;

      // Trail
      m.trail.push({ x: m.x, y: m.y });
      if (m.trail.length > 40) m.trail.shift();

      // MIRV split
      if (m.isMirv && !m.hasSplit && m.y > MIRV_SPLIT_SCREEN_Y && m.vy > 0) {
        m.hasSplit = true;
        const speed = Math.sqrt(m.vx * m.vx + m.vy * m.vy);
        const baseAngle = Math.atan2(m.vy, m.vx);
        const halfSpread = MIRV_SPREAD_ANGLE / 2;

        for (let i = 0; i < MIRV_CHILD_COUNT; i++) {
          const offset = -halfSpread + MIRV_SPREAD_ANGLE * (i / (MIRV_CHILD_COUNT - 1));
          const angle = baseAngle + offset;
          mirvSpawns.push({
            x: m.x, y: m.y,
            vx: Math.cos(angle) * speed,
            vy: Math.sin(angle) * speed,
            isMirv: false, hasSplit: false,
            life: 0, maxLife: 8,
            color: HOT_PINK,
            trail: [{ x: m.x, y: m.y }],
          });
        }

        // Split burst effect
        this.addExplosion(m.x, m.y, NEON_CYAN, 25, 0.3);
        m.life = m.maxLife; // remove carrier
      }

      // Ground impact
      if (m.y > GROUND_Y) {
        this.addExplosion(m.x, GROUND_Y, HOT_PINK, 30, 0.5);
        m.life = m.maxLife;
      }

      // OOB
      if (m.x < -80 || m.x > this.width + 80 || m.y < -200) {
        m.life = m.maxLife;
      }
    }
    // Add MIRV children after iteration
    for (const child of mirvSpawns) this.missiles.push(child);

    // ── Interceptor physics ───────────────────────────────────────────
    for (const ic of this.interceptors) {
      ic.life += dtSec;

      // Thrust (during burn)
      if (ic.burnRemaining > 0) {
        const dx = ic.targetX - ic.x;
        const dy = ic.targetY - ic.y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist > 1) {
          ic.vx += (dx / dist) * ic.thrust * dtSec;
          ic.vy += (dy / dist) * ic.thrust * dtSec;
        }
        ic.burnRemaining = Math.max(0, ic.burnRemaining - dtSec);
      }

      // Gravity
      ic.vy += MENU_GRAVITY * dtSec;

      // Movement
      ic.x += ic.vx * dtSec;
      ic.y += ic.vy * dtSec;

      // Trail
      ic.trail.push({ x: ic.x, y: ic.y });
      if (ic.trail.length > 30) ic.trail.shift();

      // Proximity detonation vs missiles
      for (const m of this.missiles) {
        if (m.life >= m.maxLife) continue;
        const dx = ic.x - m.x;
        const dy = ic.y - m.y;
        if (dx * dx + dy * dy < DETONATION_PROXIMITY * DETONATION_PROXIMITY) {
          this.addExplosion(ic.x, ic.y, ic.color, 45, 0.6);
          ic.life = ic.maxLife;
          m.life = m.maxLife;
          break;
        }
      }

      // Overshoot check (post-burn, moving away from target)
      if (ic.burnRemaining <= 0 && ic.life > 0.5) {
        const tx = ic.targetX - ic.x;
        const ty = ic.targetY - ic.y;
        if (ic.vx * tx + ic.vy * ty < 0) {
          this.addExplosion(ic.x, ic.y, ic.color, 35, 0.5);
          ic.life = ic.maxLife;
        }
      }

      // OOB / ground
      if (ic.y > GROUND_Y || ic.x < -50 || ic.x > this.width + 50 || ic.y < -100) {
        ic.life = ic.maxLife;
      }
    }

    // ── Explosions ────────────────────────────────────────────────────
    for (const e of this.explosions) {
      e.life += dtSec;
      const progress = e.life / e.maxLife;
      e.radius = e.maxRadius * Math.min(1, progress * 2.5);
    }

    // ── Cleanup ───────────────────────────────────────────────────────
    this.missiles = this.missiles.filter((m) => m.life < m.maxLife);
    this.interceptors = this.interceptors.filter((ic) => ic.life < ic.maxLife);
    this.explosions = this.explosions.filter((e) => e.life < e.maxLife);

    // ── Stars ─────────────────────────────────────────────────────────
    for (const s of this.stars) {
      s.x += s.vx * dtSec;
      s.y += s.vy * dtSec;
      if (s.y > this.height + 10) { s.y = -10; s.x = Math.random() * this.width; }
      if (s.x < -10) s.x = this.width + 10;
      if (s.x > this.width + 10) s.x = -10;
    }

    // ── Blips ─────────────────────────────────────────────────────────
    for (const b of this.blips) {
      b.life += dtSec;
      b.radius += 55 * dtSec;
    }
    this.blips = this.blips.filter((b) => b.life < b.maxLife);

    // ── Scan band ─────────────────────────────────────────────────────
    this.scanY += dtSec * 60;
    if (this.scanY > this.height + 80) this.scanY = -80;

    // ── Draw ──────────────────────────────────────────────────────────
    this.drawStars();
    this.drawGroundElements();
    this.drawParticles();
    this.drawScanAndBlips();
  }

  // ═══════════════════════════════════════════════════════════════════════
  //  SPAWNING
  // ═══════════════════════════════════════════════════════════════════════

  private spawnMissile() {
    const spawnX = 50 + Math.random() * (this.width - 100);
    const spawnY = -10;

    // Target a random city (with slight spread)
    const city = CITY_POSITIONS[Math.floor(Math.random() * CITY_POSITIONS.length)];
    const targetX = city.x + (Math.random() - 0.5) * 40;
    const targetY = city.y;

    // Flight time — longer = higher arc
    const flightTime = 4.0 + Math.random() * 5.0;

    // Ballistic arc in screen coords (Y-down, gravity positive)
    // y(T) = y0 + vy*T + 0.5*g*T^2  →  vy = (targetY-spawnY)/T - 0.5*g*T
    const vx = (targetX - spawnX) / flightTime;
    const vy = (targetY - spawnY) / flightTime - 0.5 * MENU_GRAVITY * flightTime;

    const isMirv = Math.random() < 0.25;
    const color = isMirv
      ? MIRV_BODY
      : MISSILE_COLORS[Math.floor(Math.random() * MISSILE_COLORS.length)];

    this.missiles.push({
      x: spawnX, y: spawnY, vx, vy,
      isMirv, hasSplit: false,
      life: 0, maxLife: flightTime + 4,
      color,
      trail: [],
    });
  }

  private spawnInterceptor() {
    // Pick active missiles as potential targets
    const targets = this.missiles.filter(
      (m) => m.life < m.maxLife * 0.8 && m.y > 0 && m.y < this.height * 0.7
    );
    if (targets.length === 0) return;

    const target = targets[Math.floor(Math.random() * targets.length)];
    const battery = BATTERY_POSITIONS[Math.floor(Math.random() * BATTERY_POSITIONS.length)];

    // Predict intercept point (lead the target)
    const leadTime = 1.0 + Math.random() * 1.5;
    const predictedX = target.x + target.vx * leadTime;
    const predictedY = target.y + target.vy * leadTime + 0.5 * MENU_GRAVITY * leadTime * leadTime;
    const targetY = Math.min(predictedY, GROUND_Y - 40);

    // Random interceptor type
    const profile = INTERCEPTOR_PROFILES[Math.floor(Math.random() * INTERCEPTOR_PROFILES.length)];

    // Small initial velocity toward target
    const dx = predictedX - battery.x;
    const dy = targetY - battery.y;
    const dist = Math.max(Math.sqrt(dx * dx + dy * dy), 1);

    this.interceptors.push({
      x: battery.x, y: battery.y,
      vx: (dx / dist) * 10,
      vy: (dy / dist) * 10,
      targetX: predictedX,
      targetY,
      thrust: profile.thrust,
      burnRemaining: profile.burn,
      life: 0, maxLife: 8,
      color: profile.color,
      trail: [],
    });
  }

  private addExplosion(x: number, y: number, color: number, maxRadius: number, maxLife: number) {
    if (this.explosions.length >= MAX_EXPLOSIONS) return;
    this.explosions.push({
      x, y, radius: 0, maxRadius, life: 0, maxLife, color,
    });
  }

  private spawnBlip() {
    const padding = 80;
    const colors = [NEON_CYAN, ELECTRIC_BLUE, NEON_MAGENTA, SOLAR_YELLOW];
    this.blips.push({
      x: padding + Math.random() * (this.width - padding * 2),
      y: padding + Math.random() * (this.height - padding * 2),
      life: 0,
      maxLife: 0.9 + Math.random() * 0.8,
      radius: 8 + Math.random() * 10,
      color: colors[Math.floor(Math.random() * colors.length)],
    });
  }

  // ═══════════════════════════════════════════════════════════════════════
  //  DRAWING
  // ═══════════════════════════════════════════════════════════════════════

  private drawStars() {
    this.starsGfx.clear();
    for (const s of this.stars) {
      const twinkle = 0.5 + 0.5 * Math.sin(this.timeSec * s.twinkleRate + s.x * 0.01);
      const alpha = s.baseAlpha + twinkle * 0.08;
      this.starsGfx.circle(s.x, s.y, s.radius);
      this.starsGfx.fill({ color: s.color, alpha });
    }
  }

  private drawGroundElements() {
    this.groundGfx.clear();

    // Ground line
    this.groundGfx.moveTo(0, GROUND_Y);
    this.groundGfx.lineTo(this.width, GROUND_Y);
    this.groundGfx.stroke({ width: 1, color: NEON_CYAN, alpha: 0.12 });

    // Cities — dim silhouettes
    for (const city of CITY_POSITIONS) {
      const cx = city.x;
      const cy = city.y;
      // Left building
      this.groundGfx.rect(cx - 15, cy - 18, 8, 18);
      this.groundGfx.fill({ color: NEON_CYAN, alpha: 0.06 });
      // Center building (taller)
      this.groundGfx.rect(cx - 4, cy - 28, 8, 28);
      this.groundGfx.fill({ color: NEON_CYAN, alpha: 0.06 });
      // Right building
      this.groundGfx.rect(cx + 7, cy - 14, 8, 14);
      this.groundGfx.fill({ color: NEON_CYAN, alpha: 0.06 });
    }

    // Batteries — small triangles
    for (const bat of BATTERY_POSITIONS) {
      this.groundGfx.moveTo(bat.x, bat.y - 10);
      this.groundGfx.lineTo(bat.x - 6, bat.y);
      this.groundGfx.lineTo(bat.x + 6, bat.y);
      this.groundGfx.closePath();
      this.groundGfx.fill({ color: NEON_CYAN, alpha: 0.12 });
    }
  }

  private drawParticles() {
    const gfx = this.particleGfx;
    gfx.clear();

    // ── Missile trails + heads ──────────────────────────────────────
    for (const m of this.missiles) {
      const alpha = Math.max(0, 1 - m.life / m.maxLife);

      // Trail (glow pass then sharp pass)
      if (m.trail.length > 1) {
        for (let i = 1; i < m.trail.length; i++) {
          const t = i / m.trail.length;
          const ta = t * alpha * 0.3;
          // Glow
          gfx.moveTo(m.trail[i - 1].x, m.trail[i - 1].y);
          gfx.lineTo(m.trail[i].x, m.trail[i].y);
          gfx.stroke({ width: 5, color: m.color, alpha: ta * 0.4 });
          // Sharp
          gfx.moveTo(m.trail[i - 1].x, m.trail[i - 1].y);
          gfx.lineTo(m.trail[i].x, m.trail[i].y);
          gfx.stroke({ width: 1.5, color: m.color, alpha: ta });
        }
      }

      // Head — glow ring + body + core
      const headR = m.isMirv ? 4 : 3;
      gfx.circle(m.x, m.y, headR + 4);
      gfx.fill({ color: m.color, alpha: alpha * 0.1 });
      gfx.circle(m.x, m.y, headR);
      gfx.fill({ color: m.color, alpha: alpha * 0.9 });
      gfx.circle(m.x, m.y, headR * 0.5);
      gfx.fill({ color: 0xffffff, alpha: alpha * 0.7 });
    }

    // ── Interceptor trails + heads ──────────────────────────────────
    for (const ic of this.interceptors) {
      const alpha = Math.max(0, 1 - ic.life / ic.maxLife);
      const burning = ic.burnRemaining > 0;

      // Trail
      if (ic.trail.length > 1) {
        for (let i = 1; i < ic.trail.length; i++) {
          const t = i / ic.trail.length;
          const ta = t * alpha * 0.35;
          gfx.moveTo(ic.trail[i - 1].x, ic.trail[i - 1].y);
          gfx.lineTo(ic.trail[i].x, ic.trail[i].y);
          gfx.stroke({ width: 4, color: ic.color, alpha: ta * 0.35 });
          gfx.moveTo(ic.trail[i - 1].x, ic.trail[i - 1].y);
          gfx.lineTo(ic.trail[i].x, ic.trail[i].y);
          gfx.stroke({ width: 1.2, color: ic.color, alpha: ta });
        }
      }

      // Head
      if (burning) {
        // Exhaust glow
        gfx.circle(ic.x, ic.y, 7);
        gfx.fill({ color: ic.color, alpha: alpha * 0.12 });
        gfx.circle(ic.x, ic.y, 3.5);
        gfx.fill({ color: ic.color, alpha: alpha * 0.9 });
        gfx.circle(ic.x, ic.y, 1.5);
        gfx.fill({ color: 0xffffff, alpha: alpha * 0.8 });
      } else {
        // Coasting — smaller, dimmer
        gfx.circle(ic.x, ic.y, 2.5);
        gfx.fill({ color: ic.color, alpha: alpha * 0.7 });
        gfx.circle(ic.x, ic.y, 1);
        gfx.fill({ color: 0xffffff, alpha: alpha * 0.5 });
      }
    }

    // ── Explosions ──────────────────────────────────────────────────
    for (const e of this.explosions) {
      const alpha = Math.max(0, 1 - e.life / e.maxLife);

      // Outer ring
      gfx.circle(e.x, e.y, e.radius);
      gfx.stroke({ color: e.color, alpha: alpha * 0.8, width: 2 });

      // Inner glow
      gfx.circle(e.x, e.y, e.radius * 0.5);
      gfx.fill({ color: 0xffffff, alpha: alpha * 0.2 });

      // Core flash (only early in life)
      if (e.life < e.maxLife * 0.3) {
        const flashAlpha = (1 - e.life / (e.maxLife * 0.3)) * alpha;
        gfx.circle(e.x, e.y, 4);
        gfx.fill({ color: 0xffffff, alpha: flashAlpha * 0.6 });
      }
    }
  }

  private drawScanAndBlips() {
    this.scanGfx.clear();

    // Scan band
    const y = this.scanY;
    this.scanGfx.rect(0, y - 16, this.width, 32);
    this.scanGfx.fill({ color: NEON_CYAN, alpha: 0.012 });
    this.scanGfx.rect(0, y - 1, this.width, 2);
    this.scanGfx.fill({ color: NEON_CYAN, alpha: 0.05 });

    // Signal blips
    for (const b of this.blips) {
      const t = b.life / b.maxLife;
      const a = Math.max(0, 1 - t);
      const r = b.radius;

      this.scanGfx.circle(b.x, b.y, r);
      this.scanGfx.stroke({ color: b.color, alpha: a * 0.7, width: 1.5 });
      this.scanGfx.circle(b.x, b.y, r * 0.55);
      this.scanGfx.stroke({ color: b.color, alpha: a * 0.25, width: 3 });

      if (t < 0.25) {
        const tick = 6 + t * 10;
        this.scanGfx.moveTo(b.x - tick, b.y);
        this.scanGfx.lineTo(b.x + tick, b.y);
        this.scanGfx.stroke({ color: b.color, alpha: a * 0.15, width: 1 });
        this.scanGfx.moveTo(b.x, b.y - tick);
        this.scanGfx.lineTo(b.x, b.y + tick);
        this.scanGfx.stroke({ color: b.color, alpha: a * 0.15, width: 1 });
      }
    }
  }

  // ─── Visibility ───────────────────────────────────────────────────────
  set visible(v: boolean) { this.container.visible = v; }
  get visible(): boolean { return this.container.visible; }
}
