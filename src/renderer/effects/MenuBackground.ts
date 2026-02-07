import { Application, Container, Graphics } from "pixi.js";
import {
  HOT_PINK, NEON_ORANGE, NEON_CYAN, NEON_GREEN, NEON_MAGENTA,
  ELECTRIC_BLUE, SOLAR_YELLOW, PANEL_BORDER,
} from "../ui/Theme";

const MISSILE_COLORS = [HOT_PINK, 0xff0044, NEON_ORANGE];
const INTERCEPTOR_COLORS = [NEON_CYAN, NEON_GREEN, NEON_MAGENTA, ELECTRIC_BLUE];
const EXPLOSION_COLORS = [SOLAR_YELLOW, NEON_ORANGE, NEON_CYAN];
const STAR_COLORS = [NEON_CYAN, ELECTRIC_BLUE, NEON_MAGENTA];
const GRID_SPACING = 80;

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  type: "missile" | "interceptor" | "explosion";
  life: number;
  maxLife: number;
  radius: number;
  targetX: number;
  targetY: number;
  color: number;
}

interface Star {
  x: number;
  y: number;
  vx: number;
  vy: number;
  radius: number;
  baseAlpha: number;
  twinkleRate: number;
  color: number;
}

interface Blip {
  x: number;
  y: number;
  life: number;
  maxLife: number;
  radius: number;
  color: number;
}

export class MenuBackground {
  private container: Container;
  private backdropGfx: Graphics;
  private gridGfx: Graphics;
  private starsGfx: Graphics;
  private scanGfx: Graphics;
  private particleGfx: Graphics;
  private particles: Particle[] = [];
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

  constructor(
    app: Application,
    width: number,
    height: number
  ) {
    this.width = width;
    this.height = height;

    this.container = new Container();
    app.stage.addChild(this.container);

    // Backdrop wash (behind everything)
    this.backdropGfx = new Graphics();
    this.drawBackdrop();
    this.container.addChild(this.backdropGfx);

    // Stars (behind grid)
    this.starsGfx = new Graphics();
    this.container.addChild(this.starsGfx);
    this.seedStars();

    // Grid lines (behind particles)
    this.gridGfx = new Graphics();
    this.drawGrid();
    this.container.addChild(this.gridGfx);

    // Particles (missiles/interceptors/explosions)
    this.particleGfx = new Graphics();
    this.container.addChild(this.particleGfx);

    // Scan band + blips (above particles)
    this.scanGfx = new Graphics();
    this.container.addChild(this.scanGfx);
  }

  private drawBackdrop() {
    this.backdropGfx.clear();

    // Deep base
    this.backdropGfx.rect(0, 0, this.width, this.height);
    this.backdropGfx.fill({ color: 0x050510, alpha: 1.0 });

    // Subtle “phosphor haze” blooms (static, very low alpha)
    const r0 = Math.max(this.width, this.height) * 0.75;
    this.backdropGfx.circle(this.width * 0.5, this.height * 0.45, r0);
    this.backdropGfx.fill({ color: NEON_CYAN, alpha: 0.03 });

    this.backdropGfx.circle(this.width * 0.78, this.height * 0.72, r0 * 0.55);
    this.backdropGfx.fill({ color: NEON_MAGENTA, alpha: 0.02 });

    this.backdropGfx.circle(this.width * 0.2, this.height * 0.82, r0 * 0.45);
    this.backdropGfx.fill({ color: ELECTRIC_BLUE, alpha: 0.02 });
  }

  private drawGrid() {
    this.gridGfx.clear();
    const extra = GRID_SPACING * 2;

    // Vertical lines
    for (let x = -extra; x <= this.width + extra; x += GRID_SPACING) {
      this.gridGfx.moveTo(x, -extra);
      this.gridGfx.lineTo(x, this.height + extra);
      this.gridGfx.stroke({ width: 1, color: PANEL_BORDER, alpha: 0.2 });
    }

    // Horizontal lines
    for (let y = -extra; y <= this.height + extra; y += GRID_SPACING) {
      this.gridGfx.moveTo(-extra, y);
      this.gridGfx.lineTo(this.width + extra, y);
      this.gridGfx.stroke({ width: 1, color: PANEL_BORDER, alpha: 0.2 });
    }
  }

  private seedStars() {
    const starCount = 90;
    this.stars = [];
    for (let i = 0; i < starCount; i++) {
      const color = STAR_COLORS[Math.floor(Math.random() * STAR_COLORS.length)];
      this.stars.push({
        x: Math.random() * this.width,
        y: Math.random() * this.height,
        vx: (Math.random() - 0.5) * 6,
        vy: 8 + Math.random() * 22,
        radius: 0.6 + Math.random() * 1.4,
        baseAlpha: 0.05 + Math.random() * 0.12,
        twinkleRate: 0.6 + Math.random() * 1.8,
        color,
      });
    }
  }

  start() {
    this.running = true;
    this.particles = [];
    this.spawnTimerMissile = 0;
    this.spawnTimerInterceptor = 0;
    this.blips = [];
    this.blipTimer = 0;
    this.timeSec = 0;
    this.gridOffsetX = 0;
    this.gridOffsetY = 0;
    this.scanY = Math.random() * this.height;
  }

  stop() {
    this.running = false;
    this.particles = [];
    this.blips = [];
    this.particleGfx.clear();
    this.starsGfx.clear();
    this.scanGfx.clear();
  }

  update(dt: number) {
    if (!this.running) return;

    const dtSec = dt * (1 / 60); // Pixi ticker dt is in 60fps “frames”
    this.timeSec += dtSec;

    // Spawn missiles from top
    this.spawnTimerMissile += dtSec;
    if (this.spawnTimerMissile > 1.2) {
      this.spawnTimerMissile = 0;
      this.spawnMissile();
    }

    // Spawn interceptors from bottom
    this.spawnTimerInterceptor += dtSec;
    if (this.spawnTimerInterceptor > 1.6) {
      this.spawnTimerInterceptor = 0;
      this.spawnInterceptor();
    }

    // Spawn occasional UI “signal blips”
    this.blipTimer -= dtSec;
    if (this.blipTimer <= 0) {
      this.spawnBlip();
      this.blipTimer = 0.35 + Math.random() * 0.9;
    }

    // Animate grid drift (static geometry; we just offset the container)
    this.gridOffsetX = (this.gridOffsetX + dtSec * 9) % GRID_SPACING;
    this.gridOffsetY = (this.gridOffsetY + dtSec * 14) % GRID_SPACING;
    this.gridGfx.x = -this.gridOffsetX;
    this.gridGfx.y = -this.gridOffsetY;
    this.gridGfx.alpha = 0.16 + 0.04 * (0.5 + 0.5 * Math.sin(this.timeSec * 0.35));

    // Update particles
    for (const p of this.particles) {
      p.life += dtSec;

      if (p.type === "missile") {
        p.x += p.vx * dtSec;
        p.y += p.vy * dtSec;
      } else if (p.type === "interceptor") {
        // Move toward target
        const dx = p.targetX - p.x;
        const dy = p.targetY - p.y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist > 5) {
          const speed = 200;
          p.vx = (dx / dist) * speed;
          p.vy = (dy / dist) * speed;
        }
        p.x += p.vx * dtSec;
        p.y += p.vy * dtSec;

        // Check if interceptor reached near a missile — trigger explosion
        if (dist < 20 && p.life > 0.3) {
          this.spawnExplosion(p.x, p.y);
          p.life = p.maxLife; // mark for removal
        }
      } else if (p.type === "explosion") {
        p.radius += 80 * dtSec;
      }
    }

    // Remove dead particles
    this.particles = this.particles.filter((p) => p.life < p.maxLife);

    // Occasionally spawn random explosions
    const randomExplosionRatePerSec = 0.18;
    if (Math.random() < randomExplosionRatePerSec * dtSec) {
      this.spawnExplosion(
        100 + Math.random() * (this.width - 200),
        100 + Math.random() * (this.height - 200)
      );
    }

    // Update stars (slow drift + twinkle)
    for (const s of this.stars) {
      s.x += s.vx * dtSec;
      s.y += s.vy * dtSec;
      if (s.y > this.height + 10) {
        s.y = -10;
        s.x = Math.random() * this.width;
      }
      if (s.x < -10) s.x = this.width + 10;
      if (s.x > this.width + 10) s.x = -10;
    }

    // Update blips (expanding rings)
    for (const b of this.blips) {
      b.life += dtSec;
      b.radius += 55 * dtSec;
    }
    this.blips = this.blips.filter((b) => b.life < b.maxLife);

    // Scan band (slow vertical sweep)
    this.scanY += dtSec * 60;
    if (this.scanY > this.height + 80) {
      this.scanY = -80;
    }

    this.drawStars();
    this.drawParticles();
    this.drawScanAndBlips();
  }

  private drawStars() {
    this.starsGfx.clear();
    for (const s of this.stars) {
      const twinkle = 0.5 + 0.5 * Math.sin(this.timeSec * s.twinkleRate + s.x * 0.01);
      const alpha = s.baseAlpha + twinkle * 0.08;
      this.starsGfx.circle(s.x, s.y, s.radius);
      this.starsGfx.fill({ color: s.color, alpha });
    }
  }

  private drawParticles() {
    this.particleGfx.clear();

    for (const p of this.particles) {
      const alpha = Math.max(0, 1 - p.life / p.maxLife);

      if (p.type === "missile") {
        // Neon dot with short trail
        const trailLen = 4;
        for (let i = 0; i < trailLen; i++) {
          const t = i / trailLen;
          const tx = p.x - p.vx * t * 0.05;
          const ty = p.y - p.vy * t * 0.05;
          this.particleGfx.circle(tx, ty, 2.5 - i * 0.4);
          this.particleGfx.fill({ color: p.color, alpha: alpha * (1 - t * 0.7) });
        }
      } else if (p.type === "interceptor") {
        // Neon dot with trail
        const trailLen = 3;
        for (let i = 0; i < trailLen; i++) {
          const t = i / trailLen;
          const tx = p.x - p.vx * t * 0.03;
          const ty = p.y - p.vy * t * 0.03;
          this.particleGfx.circle(tx, ty, 2 - i * 0.4);
          this.particleGfx.fill({ color: p.color, alpha: alpha * (1 - t * 0.6) });
        }
        // Glow ring on head
        this.particleGfx.circle(p.x, p.y, 4);
        this.particleGfx.fill({ color: p.color, alpha: alpha * 0.15 });
      } else if (p.type === "explosion") {
        // Expanding neon circle that fades
        this.particleGfx.circle(p.x, p.y, p.radius);
        this.particleGfx.stroke({ color: p.color, alpha: alpha * 0.8, width: 2 });
        // Inner glow
        this.particleGfx.circle(p.x, p.y, p.radius * 0.5);
        this.particleGfx.fill({ color: 0xffffff, alpha: alpha * 0.2 });
      }
    }
  }

  private drawScanAndBlips() {
    this.scanGfx.clear();

    // Scan band (thin bright line + wider glow)
    const y = this.scanY;
    this.scanGfx.rect(0, y - 16, this.width, 32);
    this.scanGfx.fill({ color: NEON_CYAN, alpha: 0.012 });
    this.scanGfx.rect(0, y - 1, this.width, 2);
    this.scanGfx.fill({ color: NEON_CYAN, alpha: 0.05 });

    // Signal blips (expanding rings)
    for (const b of this.blips) {
      const t = b.life / b.maxLife;
      const a = Math.max(0, 1 - t);
      const r = b.radius;

      this.scanGfx.circle(b.x, b.y, r);
      this.scanGfx.stroke({ color: b.color, alpha: a * 0.7, width: 1.5 });

      this.scanGfx.circle(b.x, b.y, r * 0.55);
      this.scanGfx.stroke({ color: b.color, alpha: a * 0.25, width: 3 });

      // Small crosshair tick marks at peak intensity
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

  private spawnMissile() {
    const x = 50 + Math.random() * (this.width - 100);
    const color = MISSILE_COLORS[Math.floor(Math.random() * MISSILE_COLORS.length)];
    this.particles.push({
      x,
      y: -10,
      vx: (Math.random() - 0.5) * 30,
      vy: 60 + Math.random() * 40,
      type: "missile",
      life: 0,
      maxLife: 10,
      radius: 0,
      targetX: 0,
      targetY: 0,
      color,
    });
  }

  private spawnInterceptor() {
    const fromLeft = Math.random() > 0.5;
    const x = fromLeft ? 80 + Math.random() * 100 : this.width - 180 + Math.random() * 100;
    const y = this.height - 30;

    const missiles = this.particles.filter((p) => p.type === "missile" && p.life < p.maxLife * 0.8);
    let targetX: number, targetY: number;
    if (missiles.length > 0) {
      const target = missiles[Math.floor(Math.random() * missiles.length)];
      targetX = target.x + target.vx * 1.5;
      targetY = target.y + target.vy * 1.5;
    } else {
      targetX = 100 + Math.random() * (this.width - 200);
      targetY = 100 + Math.random() * 300;
    }

    const color = INTERCEPTOR_COLORS[Math.floor(Math.random() * INTERCEPTOR_COLORS.length)];
    this.particles.push({
      x,
      y,
      vx: 0,
      vy: 0,
      type: "interceptor",
      life: 0,
      maxLife: 6,
      radius: 0,
      targetX,
      targetY,
      color,
    });
  }

  private spawnExplosion(x: number, y: number) {
    const color = EXPLOSION_COLORS[Math.floor(Math.random() * EXPLOSION_COLORS.length)];
    this.particles.push({
      x,
      y,
      vx: 0,
      vy: 0,
      type: "explosion",
      life: 0,
      maxLife: 1.0,
      radius: 3,
      targetX: 0,
      targetY: 0,
      color,
    });
  }

  private spawnBlip() {
    const padding = 80;
    const x = padding + Math.random() * (this.width - padding * 2);
    const y = padding + Math.random() * (this.height - padding * 2);
    const color = [NEON_CYAN, ELECTRIC_BLUE, NEON_MAGENTA, SOLAR_YELLOW][
      Math.floor(Math.random() * 4)
    ];
    this.blips.push({
      x,
      y,
      life: 0,
      maxLife: 0.9 + Math.random() * 0.8,
      radius: 8 + Math.random() * 10,
      color,
    });
  }

  set visible(v: boolean) {
    this.container.visible = v;
  }

  get visible(): boolean {
    return this.container.visible;
  }
}
