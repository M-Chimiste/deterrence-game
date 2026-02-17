/**
 * InterceptEffects — visual flash effects for engagement outcomes.
 *
 * Hit: bright white flash (expanding sphere)
 * Miss: brief dim flash
 * Vampire impact: red flash at ground level
 */

import * as THREE from "three";
import type { AudioEvent } from "../ipc/state";

const EFFECT_DURATION = 0.5; // seconds
const FLASH_SIZE = 3000; // meters — visible at world scale

interface ActiveEffect {
  mesh: THREE.Mesh;
  light: THREE.PointLight;
  startTime: number;
  duration: number;
}

export class InterceptEffects {
  private group: THREE.Group;
  private effects: ActiveEffect[] = [];
  private flashGeo: THREE.SphereGeometry;

  constructor(parent: THREE.Group) {
    this.group = new THREE.Group();
    parent.add(this.group);
    this.flashGeo = new THREE.SphereGeometry(FLASH_SIZE, 8, 6);
  }

  /** Process audio events to trigger effects at track positions. */
  processEvents(
    events: AudioEvent[],
    trackPositions: Map<number, { x: number; y: number; z: number }>,
  ): void {
    for (const event of events) {
      if (event.type === "Splash") {
        const pos = trackPositions.get(event.track_number);
        if (pos) {
          const isHit = event.result === "Hit";
          this.spawnFlash(
            pos.x,
            pos.z,
            -pos.y,
            isHit ? 0xffffff : 0x888888,
            isHit ? 2.0 : 0.5,
          );
        }
      } else if (event.type === "VampireImpact") {
        // Impact at origin (own-ship position), ground level
        this.spawnFlash(0, 500, 0, 0xff2200, 3.0);
      }
    }
  }

  /** Spawn a flash effect at the given Three.js position. */
  private spawnFlash(
    x: number,
    y: number,
    z: number,
    color: number,
    intensity: number,
  ): void {
    const mat = new THREE.MeshBasicMaterial({
      color,
      transparent: true,
      opacity: 0.8,
    });
    const mesh = new THREE.Mesh(this.flashGeo, mat);
    mesh.position.set(x, y, z);
    this.group.add(mesh);

    const light = new THREE.PointLight(color, intensity, FLASH_SIZE * 10);
    light.position.set(x, y, z);
    this.group.add(light);

    this.effects.push({
      mesh,
      light,
      startTime: performance.now() / 1000,
      duration: EFFECT_DURATION,
    });
  }

  /** Update effects each frame — expand and fade out. */
  update(): void {
    const now = performance.now() / 1000;
    const toRemove: number[] = [];

    for (let i = 0; i < this.effects.length; i++) {
      const effect = this.effects[i];
      const elapsed = now - effect.startTime;
      const t = elapsed / effect.duration;

      if (t >= 1.0) {
        toRemove.push(i);
        continue;
      }

      // Expand and fade
      const scale = 1.0 + t * 2.0;
      effect.mesh.scale.setScalar(scale);
      (effect.mesh.material as THREE.MeshBasicMaterial).opacity =
        0.8 * (1.0 - t);
      effect.light.intensity = effect.light.intensity * (1.0 - t);
    }

    // Remove expired effects (iterate in reverse to preserve indices)
    for (let i = toRemove.length - 1; i >= 0; i--) {
      const idx = toRemove[i];
      const effect = this.effects[idx];
      this.group.remove(effect.mesh);
      this.group.remove(effect.light);
      (effect.mesh.material as THREE.Material).dispose();
      effect.light.dispose();
      this.effects.splice(idx, 1);
    }
  }

  dispose(): void {
    for (const effect of this.effects) {
      this.group.remove(effect.mesh);
      this.group.remove(effect.light);
      (effect.mesh.material as THREE.Material).dispose();
      effect.light.dispose();
    }
    this.effects = [];
    this.flashGeo.dispose();
  }
}
