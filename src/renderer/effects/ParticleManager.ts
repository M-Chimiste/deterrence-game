import { Container, Graphics } from "pixi.js";
import { EXPLOSION_COLORS, NEON_CYAN, NEON_MAGENTA, SOLAR_YELLOW, NEON_ORANGE } from "../ui/Theme";

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
  type: "spark" | "ember" | "flash";
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
    const count = Math.floor(15 + intensity * 12);
    const speed = 150 + intensity * 100;

    // Sparks
    for (let i = 0; i < count; i++) {
      const angle = Math.random() * Math.PI * 2;
      const v = (0.3 + Math.random() * 0.7) * speed;
      this.particles.push({
        x, y: screenY,
        vx: Math.cos(angle) * v,
        vy: Math.sin(angle) * v,
        life: 0.2 + Math.random() * 0.5,
        maxLife: 0.2 + Math.random() * 0.5,
        size: 1 + Math.random() * 2,
        color: EXPLOSION_COLORS[Math.floor(Math.random() * EXPLOSION_COLORS.length)],
        type: "spark",
        gravity: false,
      });
    }

    // Embers (gravity-affected)
    const emberCount = Math.floor(3 + intensity * 4);
    for (let i = 0; i < emberCount; i++) {
      const angle = Math.random() * Math.PI * 2;
      const v = (0.2 + Math.random() * 0.5) * speed * 0.5;
      this.particles.push({
        x, y: screenY,
        vx: Math.cos(angle) * v,
        vy: Math.sin(angle) * v,
        life: 0.5 + Math.random() * 1.0,
        maxLife: 0.5 + Math.random() * 1.0,
        size: 2 + Math.random() * 3,
        color: NEON_ORANGE,
        type: "ember",
        gravity: true,
      });
    }

    // Flash (central white burst)
    this.particles.push({
      x, y: screenY,
      vx: 0, vy: 0,
      life: 0.12 + intensity * 0.03,
      maxLife: 0.12 + intensity * 0.03,
      size: 12 + intensity * 8,
      color: 0xffffff,
      type: "flash",
      gravity: false,
    });
  }

  spawnImpact(x: number, y: number) {
    const screenY = WORLD_HEIGHT - y;
    // Debris shooting upward from ground
    for (let i = 0; i < 12; i++) {
      const angle = -Math.PI / 2 + (Math.random() - 0.5) * Math.PI * 0.7;
      const v = 80 + Math.random() * 180;
      this.particles.push({
        x: x + (Math.random() - 0.5) * 10,
        y: screenY,
        vx: Math.cos(angle) * v,
        vy: Math.sin(angle) * v,
        life: 0.3 + Math.random() * 0.5,
        maxLife: 0.3 + Math.random() * 0.5,
        size: 1 + Math.random() * 2,
        color: [0x666666, 0x886644, SOLAR_YELLOW][Math.floor(Math.random() * 3)],
        type: "spark",
        gravity: true,
      });
    }

    // Ground flash
    this.particles.push({
      x, y: screenY,
      vx: 0, vy: 0,
      life: 0.15,
      maxLife: 0.15,
      size: 20,
      color: SOLAR_YELLOW,
      type: "flash",
      gravity: false,
    });
  }

  spawnMirvSplit(x: number, y: number) {
    const screenY = WORLD_HEIGHT - y;
    const colors = [NEON_CYAN, NEON_MAGENTA, 0xffffff];
    for (let i = 0; i < 10; i++) {
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
        p.vx *= 1 - 2 * dtSec;
        p.vy *= 1 - 2 * dtSec;
      }
    }

    this.drawParticles();
  }

  private drawParticles() {
    this.gfx.clear();

    for (const p of this.particles) {
      const alpha = Math.max(0, p.life / p.maxLife);

      if (p.type === "flash") {
        // Expanding/fading circle
        const progress = 1 - p.life / p.maxLife;
        const radius = p.size * (0.5 + progress * 0.5);
        this.gfx.circle(p.x, p.y, radius);
        this.gfx.fill({ color: p.color, alpha: alpha * 0.6 });
      } else if (p.type === "ember") {
        // Glow + core
        this.gfx.circle(p.x, p.y, p.size * 1.5);
        this.gfx.fill({ color: p.color, alpha: alpha * 0.15 });
        this.gfx.circle(p.x, p.y, p.size);
        this.gfx.fill({ color: p.color, alpha: alpha * 0.7 });
      } else {
        // Spark: small bright dot
        this.gfx.circle(p.x, p.y, p.size);
        this.gfx.fill({ color: p.color, alpha: alpha * 0.9 });
      }
    }
  }

  get particleCount(): number {
    return this.particles.length;
  }
}
