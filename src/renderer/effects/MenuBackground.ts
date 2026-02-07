import { Application, Graphics } from "pixi.js";
import {
  HOT_PINK, NEON_ORANGE, NEON_CYAN, NEON_GREEN, NEON_MAGENTA,
  ELECTRIC_BLUE, SOLAR_YELLOW, PANEL_BORDER,
} from "../ui/Theme";

const MISSILE_COLORS = [HOT_PINK, 0xff0044, NEON_ORANGE];
const INTERCEPTOR_COLORS = [NEON_CYAN, NEON_GREEN, NEON_MAGENTA, ELECTRIC_BLUE];
const EXPLOSION_COLORS = [SOLAR_YELLOW, NEON_ORANGE, NEON_CYAN];

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

export class MenuBackground {
  private gfx: Graphics;
  private gridGfx: Graphics;
  private particles: Particle[] = [];
  private running: boolean = false;
  private spawnTimerMissile: number = 0;
  private spawnTimerInterceptor: number = 0;
  private width: number;
  private height: number;

  constructor(
    app: Application,
    width: number,
    height: number
  ) {
    this.width = width;
    this.height = height;

    // Grid lines (behind everything)
    this.gridGfx = new Graphics();
    this.drawGrid();
    app.stage.addChild(this.gridGfx);

    this.gfx = new Graphics();
    app.stage.addChild(this.gfx);
  }

  private drawGrid() {
    this.gridGfx.clear();
    const spacing = 80;

    // Vertical lines
    for (let x = spacing; x < this.width; x += spacing) {
      this.gridGfx.moveTo(x, 0);
      this.gridGfx.lineTo(x, this.height);
      this.gridGfx.stroke({ width: 1, color: PANEL_BORDER, alpha: 0.2 });
    }

    // Horizontal lines
    for (let y = spacing; y < this.height; y += spacing) {
      this.gridGfx.moveTo(0, y);
      this.gridGfx.lineTo(this.width, y);
      this.gridGfx.stroke({ width: 1, color: PANEL_BORDER, alpha: 0.2 });
    }
  }

  start() {
    this.running = true;
    this.particles = [];
    this.spawnTimerMissile = 0;
    this.spawnTimerInterceptor = 0;
  }

  stop() {
    this.running = false;
    this.particles = [];
    this.gfx.clear();
  }

  update(dt: number) {
    if (!this.running) return;

    const delta = dt * 0.016; // normalize to ~seconds

    // Spawn missiles from top
    this.spawnTimerMissile += delta;
    if (this.spawnTimerMissile > 1.5) {
      this.spawnTimerMissile = 0;
      this.spawnMissile();
    }

    // Spawn interceptors from bottom
    this.spawnTimerInterceptor += delta;
    if (this.spawnTimerInterceptor > 2.0) {
      this.spawnTimerInterceptor = 0;
      this.spawnInterceptor();
    }

    // Update particles
    for (const p of this.particles) {
      p.life += delta;

      if (p.type === "missile") {
        p.x += p.vx * delta;
        p.y += p.vy * delta;
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
        p.x += p.vx * delta;
        p.y += p.vy * delta;

        // Check if interceptor reached near a missile — trigger explosion
        if (dist < 20 && p.life > 0.3) {
          this.spawnExplosion(p.x, p.y);
          p.life = p.maxLife; // mark for removal
        }
      } else if (p.type === "explosion") {
        p.radius += 80 * delta;
      }
    }

    // Remove dead particles
    this.particles = this.particles.filter((p) => p.life < p.maxLife);

    // Occasionally spawn random explosions
    if (Math.random() < 0.003) {
      this.spawnExplosion(
        100 + Math.random() * (this.width - 200),
        100 + Math.random() * (this.height - 200)
      );
    }

    this.draw();
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

  private draw() {
    this.gfx.clear();

    for (const p of this.particles) {
      const alpha = Math.max(0, 1 - p.life / p.maxLife);

      if (p.type === "missile") {
        // Neon dot with short trail
        const trailLen = 4;
        for (let i = 0; i < trailLen; i++) {
          const t = i / trailLen;
          const tx = p.x - p.vx * t * 0.05;
          const ty = p.y - p.vy * t * 0.05;
          this.gfx.circle(tx, ty, 2.5 - i * 0.4);
          this.gfx.fill({ color: p.color, alpha: alpha * (1 - t * 0.7) });
        }
      } else if (p.type === "interceptor") {
        // Neon dot with trail
        const trailLen = 3;
        for (let i = 0; i < trailLen; i++) {
          const t = i / trailLen;
          const tx = p.x - p.vx * t * 0.03;
          const ty = p.y - p.vy * t * 0.03;
          this.gfx.circle(tx, ty, 2 - i * 0.4);
          this.gfx.fill({ color: p.color, alpha: alpha * (1 - t * 0.6) });
        }
        // Glow ring on head
        this.gfx.circle(p.x, p.y, 4);
        this.gfx.fill({ color: p.color, alpha: alpha * 0.15 });
      } else if (p.type === "explosion") {
        // Expanding neon circle that fades
        this.gfx.circle(p.x, p.y, p.radius);
        this.gfx.stroke({ color: p.color, alpha: alpha * 0.8, width: 2 });
        // Inner glow
        this.gfx.circle(p.x, p.y, p.radius * 0.5);
        this.gfx.fill({ color: 0xffffff, alpha: alpha * 0.2 });
      }
    }
  }

  set visible(v: boolean) {
    this.gfx.visible = v;
    this.gridGfx.visible = v;
  }

  get visible(): boolean {
    return this.gfx.visible;
  }
}
