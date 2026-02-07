import { Container, Graphics } from "pixi.js";
import { EXPLOSION_COLORS, NEON_CYAN, NEON_MAGENTA, SOLAR_YELLOW, NEON_ORANGE, HOT_PINK } from "../ui/Theme";

const WORLD_HEIGHT = 720;
const GRAVITY = 120; // pixels/s^2 for visual particles

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  maxLife: number;
  size: number;
  color: number;
  type: "spark" | "ember" | "flash" | "ring" | "smoke";
  gravity: boolean;
}

export class ParticleManager {
  private gfx: Graphics;
  private particles: Particle[] = [];

  constructor(stage: Container) {
    this.gfx = new Graphics();
    stage.addChild(this.gfx);
  }

  spawnExplosion(x: number, y: number, intensity: number) {
    const screenY = WORLD_HEIGHT - y;
    const count = Math.floor(25 + intensity * 20);
    const speed = 200 + intensity * 150;

    // Sparks — fast bright shrapnel
    for (let i = 0; i < count; i++) {
      const angle = Math.random() * Math.PI * 2;
      const v = (0.3 + Math.random() * 0.7) * speed;
      this.particles.push({
        x, y: screenY,
        vx: Math.cos(angle) * v,
        vy: Math.sin(angle) * v,
        life: 0.3 + Math.random() * 0.6,
        maxLife: 0.3 + Math.random() * 0.6,
        size: 1 + Math.random() * 2.5,
        color: EXPLOSION_COLORS[Math.floor(Math.random() * EXPLOSION_COLORS.length)],
        type: "spark",
        gravity: false,
      });
    }

    // Embers — heavier, gravity-affected glowing debris
    const emberCount = Math.floor(6 + intensity * 6);
    for (let i = 0; i < emberCount; i++) {
      const angle = Math.random() * Math.PI * 2;
      const v = (0.2 + Math.random() * 0.5) * speed * 0.4;
      this.particles.push({
        x: x + (Math.random() - 0.5) * 8,
        y: screenY + (Math.random() - 0.5) * 8,
        vx: Math.cos(angle) * v,
        vy: Math.sin(angle) * v,
        life: 0.8 + Math.random() * 1.5,
        maxLife: 0.8 + Math.random() * 1.5,
        size: 2 + Math.random() * 3,
        color: [NEON_ORANGE, SOLAR_YELLOW, HOT_PINK][Math.floor(Math.random() * 3)],
        type: "ember",
        gravity: true,
      });
    }

    // Smoke puffs — large, slow, lingering
    const smokeCount = Math.floor(4 + intensity * 4);
    for (let i = 0; i < smokeCount; i++) {
      const angle = Math.random() * Math.PI * 2;
      const v = 20 + Math.random() * 40;
      this.particles.push({
        x: x + (Math.random() - 0.5) * 15,
        y: screenY + (Math.random() - 0.5) * 15,
        vx: Math.cos(angle) * v,
        vy: Math.sin(angle) * v - 15, // drift upward
        life: 0.8 + Math.random() * 1.2,
        maxLife: 0.8 + Math.random() * 1.2,
        size: 8 + intensity * 6 + Math.random() * 10,
        color: EXPLOSION_COLORS[Math.floor(Math.random() * EXPLOSION_COLORS.length)],
        type: "smoke",
        gravity: false,
      });
    }

    // Expanding shockwave ring
    this.particles.push({
      x, y: screenY,
      vx: 0, vy: 0,
      life: 0.4 + intensity * 0.15,
      maxLife: 0.4 + intensity * 0.15,
      size: 20 + intensity * 30,
      color: SOLAR_YELLOW,
      type: "ring",
      gravity: false,
    });

    // Second inner ring (different color, slightly delayed feel)
    this.particles.push({
      x, y: screenY,
      vx: 0, vy: 0,
      life: 0.3 + intensity * 0.1,
      maxLife: 0.3 + intensity * 0.1,
      size: 12 + intensity * 20,
      color: NEON_ORANGE,
      type: "ring",
      gravity: false,
    });

    // Central white flash (bigger)
    this.particles.push({
      x, y: screenY,
      vx: 0, vy: 0,
      life: 0.15 + intensity * 0.05,
      maxLife: 0.15 + intensity * 0.05,
      size: 18 + intensity * 12,
      color: 0xffffff,
      type: "flash",
      gravity: false,
    });

    // Secondary colored flash
    this.particles.push({
      x, y: screenY,
      vx: 0, vy: 0,
      life: 0.2 + intensity * 0.05,
      maxLife: 0.2 + intensity * 0.05,
      size: 25 + intensity * 15,
      color: EXPLOSION_COLORS[Math.floor(Math.random() * EXPLOSION_COLORS.length)],
      type: "flash",
      gravity: false,
    });
  }

  spawnImpact(x: number, y: number) {
    const screenY = WORLD_HEIGHT - y;
    // Debris shooting upward from ground
    for (let i = 0; i < 18; i++) {
      const angle = -Math.PI / 2 + (Math.random() - 0.5) * Math.PI * 0.7;
      const v = 80 + Math.random() * 200;
      this.particles.push({
        x: x + (Math.random() - 0.5) * 12,
        y: screenY,
        vx: Math.cos(angle) * v,
        vy: Math.sin(angle) * v,
        life: 0.4 + Math.random() * 0.6,
        maxLife: 0.4 + Math.random() * 0.6,
        size: 1 + Math.random() * 2,
        color: [0x666666, 0x886644, SOLAR_YELLOW, HOT_PINK][Math.floor(Math.random() * 4)],
        type: "spark",
        gravity: true,
      });
    }

    // Impact smoke
    for (let i = 0; i < 5; i++) {
      this.particles.push({
        x: x + (Math.random() - 0.5) * 20,
        y: screenY - Math.random() * 10,
        vx: (Math.random() - 0.5) * 30,
        vy: -20 - Math.random() * 30,
        life: 0.6 + Math.random() * 0.8,
        maxLife: 0.6 + Math.random() * 0.8,
        size: 10 + Math.random() * 15,
        color: [NEON_ORANGE, SOLAR_YELLOW][Math.floor(Math.random() * 2)],
        type: "smoke",
        gravity: false,
      });
    }

    // Ground flash
    this.particles.push({
      x, y: screenY,
      vx: 0, vy: 0,
      life: 0.2,
      maxLife: 0.2,
      size: 30,
      color: SOLAR_YELLOW,
      type: "flash",
      gravity: false,
    });

    // Impact ring
    this.particles.push({
      x, y: screenY,
      vx: 0, vy: 0,
      life: 0.3,
      maxLife: 0.3,
      size: 25,
      color: HOT_PINK,
      type: "ring",
      gravity: false,
    });
  }

  spawnMirvSplit(x: number, y: number) {
    const screenY = WORLD_HEIGHT - y;
    const colors = [NEON_CYAN, NEON_MAGENTA, 0xffffff];
    for (let i = 0; i < 14; i++) {
      const angle = Math.random() * Math.PI * 2;
      const v = 60 + Math.random() * 120;
      this.particles.push({
        x, y: screenY,
        vx: Math.cos(angle) * v,
        vy: Math.sin(angle) * v,
        life: 0.2 + Math.random() * 0.3,
        maxLife: 0.2 + Math.random() * 0.3,
        size: 1 + Math.random() * 1.5,
        color: colors[Math.floor(Math.random() * colors.length)],
        type: "spark",
        gravity: false,
      });
    }

    // Small flash
    this.particles.push({
      x, y: screenY,
      vx: 0, vy: 0,
      life: 0.1,
      maxLife: 0.1,
      size: 8,
      color: 0xffffff,
      type: "flash",
      gravity: false,
    });
  }

  update(dt: number) {
    const dtSec = dt * (1 / 60); // PixiJS ticker dt is in frames at 60fps

    // Update particles
    for (let i = this.particles.length - 1; i >= 0; i--) {
      const p = this.particles[i];
      p.life -= dtSec;
      if (p.life <= 0) {
        this.particles.splice(i, 1);
        continue;
      }

      p.x += p.vx * dtSec;
      p.y += p.vy * dtSec;

      if (p.gravity) {
        p.vy += GRAVITY * dtSec;
      }

      // Slow down sparks
      if (p.type === "spark") {
        p.vx *= 1 - 2.5 * dtSec;
        p.vy *= 1 - 2.5 * dtSec;
      }

      // Slow down smoke and let it expand
      if (p.type === "smoke") {
        p.vx *= 1 - 1.5 * dtSec;
        p.vy *= 1 - 1.0 * dtSec;
        p.size += 12 * dtSec; // expand over time
      }
    }

    this.drawParticles();
  }

  private drawParticles() {
    this.gfx.clear();

    for (const p of this.particles) {
      const lifeRatio = Math.max(0, p.life / p.maxLife);

      if (p.type === "flash") {
        // Expanding/fading bright circle
        const progress = 1 - lifeRatio;
        const radius = p.size * (0.5 + progress * 0.5);
        // Outer glow
        this.gfx.circle(p.x, p.y, radius * 1.5);
        this.gfx.fill({ color: p.color, alpha: lifeRatio * 0.15 });
        // Core
        this.gfx.circle(p.x, p.y, radius);
        this.gfx.fill({ color: p.color, alpha: lifeRatio * 0.6 });
      } else if (p.type === "ring") {
        // Expanding shockwave ring
        const progress = 1 - lifeRatio;
        const radius = p.size * (0.2 + progress * 0.8);
        // Thick ring
        this.gfx.setStrokeStyle({ width: 3 * lifeRatio, color: p.color, alpha: lifeRatio * 0.7 });
        this.gfx.circle(p.x, p.y, radius);
        this.gfx.stroke();
        // Outer glow ring
        this.gfx.setStrokeStyle({ width: 6 * lifeRatio, color: p.color, alpha: lifeRatio * 0.15 });
        this.gfx.circle(p.x, p.y, radius);
        this.gfx.stroke();
      } else if (p.type === "smoke") {
        // Soft expanding cloud
        this.gfx.circle(p.x, p.y, p.size);
        this.gfx.fill({ color: p.color, alpha: lifeRatio * 0.08 });
        this.gfx.circle(p.x, p.y, p.size * 0.6);
        this.gfx.fill({ color: p.color, alpha: lifeRatio * 0.12 });
      } else if (p.type === "ember") {
        // Glow + bright core
        this.gfx.circle(p.x, p.y, p.size * 2);
        this.gfx.fill({ color: p.color, alpha: lifeRatio * 0.1 });
        this.gfx.circle(p.x, p.y, p.size);
        this.gfx.fill({ color: p.color, alpha: lifeRatio * 0.7 });
        this.gfx.circle(p.x, p.y, p.size * 0.4);
        this.gfx.fill({ color: 0xffffff, alpha: lifeRatio * 0.3 });
      } else {
        // Spark: small bright dot with tiny glow
        this.gfx.circle(p.x, p.y, p.size * 1.5);
        this.gfx.fill({ color: p.color, alpha: lifeRatio * 0.15 });
        this.gfx.circle(p.x, p.y, p.size);
        this.gfx.fill({ color: p.color, alpha: lifeRatio * 0.9 });
      }
    }
  }

  get particleCount(): number {
    return this.particles.length;
  }
}
