/**
 * CoastlineRenderer — draws coastline polylines on the PPI scope.
 *
 * Receives coastline data in sim-space meters (x=East, y=North).
 * Renders as dim green lines on the PPI to show land/ocean boundaries.
 */

import * as THREE from "three";

const COASTLINE_COLOR = 0x003300;
const COASTLINE_OPACITY = 0.4;

export class CoastlineRenderer {
  private group: THREE.Group;
  private lines: THREE.Line[] = [];

  constructor(parent: THREE.Group) {
    this.group = new THREE.Group();
    parent.add(this.group);
  }

  /** Build coastline geometry from flat polyline arrays [x0,y0, x1,y1, ...]. */
  setCoastlines(coastlines: number[][]): void {
    this.clear();

    const mat = new THREE.LineBasicMaterial({
      color: COASTLINE_COLOR,
      transparent: true,
      opacity: COASTLINE_OPACITY,
    });

    for (const flat of coastlines) {
      if (flat.length < 4) continue; // Need at least 2 points

      const points: THREE.Vector3[] = [];
      for (let i = 0; i < flat.length; i += 2) {
        // sim-space: x=East, y=North — maps directly to PPI (x, y, 0)
        points.push(new THREE.Vector3(flat[i], flat[i + 1], 0));
      }

      const geo = new THREE.BufferGeometry().setFromPoints(points);
      const line = new THREE.Line(geo, mat);
      this.lines.push(line);
      this.group.add(line);
    }
  }

  /** Remove all coastline geometry. */
  clear(): void {
    for (const line of this.lines) {
      line.geometry.dispose();
      this.group.remove(line);
    }
    this.lines = [];
  }

  dispose(): void {
    this.clear();
    if (this.group.parent) {
      this.group.parent.remove(this.group);
    }
  }
}
