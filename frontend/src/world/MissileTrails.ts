/**
 * MissileTrails — trail lines for tracks showing recent position history.
 *
 * Blue trails for friendly interceptors, red for hostile threats.
 * Coordinate mapping: sim(x,y,z) -> three.js(x, z, -y).
 */

import * as THREE from "three";
import type { TrackView } from "../ipc/state";

const TRAIL_COLOR_HOSTILE = 0xff3333;
const TRAIL_COLOR_FRIENDLY = 0x3388ff;
const TRAIL_OPACITY = 0.5;

export class MissileTrails {
  private group: THREE.Group;
  private trails: Map<number, THREE.Line> = new Map();

  constructor(parent: THREE.Group) {
    this.group = new THREE.Group();
    parent.add(this.group);
  }

  update(tracks: TrackView[]): void {
    const activeTracks = new Set<number>();

    for (const track of tracks) {
      if (track.history.length < 2) continue;
      activeTracks.add(track.track_number);

      const isHostile =
        track.classification === "Hostile" ||
        track.classification === "Suspect";
      const color = isHostile ? TRAIL_COLOR_HOSTILE : TRAIL_COLOR_FRIENDLY;

      const points = track.history.map(
        (p) => new THREE.Vector3(p.x, p.z, -p.y),
      );
      // Add current position as the trail head
      points.push(
        new THREE.Vector3(
          track.position.x,
          track.position.z,
          -track.position.y,
        ),
      );

      let trail = this.trails.get(track.track_number);
      if (trail) {
        // Update existing trail geometry
        trail.geometry.dispose();
        trail.geometry = new THREE.BufferGeometry().setFromPoints(points);
        const mat = trail.material as THREE.LineBasicMaterial;
        if (mat.color.getHex() !== color) {
          mat.color.setHex(color);
        }
      } else {
        // Create new trail
        const geo = new THREE.BufferGeometry().setFromPoints(points);
        const mat = new THREE.LineBasicMaterial({
          color,
          transparent: true,
          opacity: TRAIL_OPACITY,
        });
        trail = new THREE.Line(geo, mat);
        this.trails.set(track.track_number, trail);
        this.group.add(trail);
      }
    }

    // Remove trails for tracks that no longer exist
    for (const [trackNum, trail] of this.trails) {
      if (!activeTracks.has(trackNum)) {
        trail.geometry.dispose();
        (trail.material as THREE.Material).dispose();
        this.group.remove(trail);
        this.trails.delete(trackNum);
      }
    }
  }

  dispose(): void {
    for (const [, trail] of this.trails) {
      trail.geometry.dispose();
      (trail.material as THREE.Material).dispose();
    }
    this.trails.clear();
  }
}
