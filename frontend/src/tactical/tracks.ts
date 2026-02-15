/**
 * Track rendering for the PPI display.
 * Manages Three.js objects for each track: symbol, velocity leader, history trail, label, hook ring.
 */

import * as THREE from "three";
import type { TrackView } from "../ipc/state";
import { createNTDSSymbol, classificationColor, NTDS_COLORS } from "./symbology";

const VELOCITY_LEADER_SECONDS = 60;
const HISTORY_DOT_SIZE = 2.0;

/** Manages all track objects on the PPI. */
export class TrackRenderer {
  private group: THREE.Group;
  private trackObjects = new Map<number, TrackObject>();
  private symbolScale: number;

  constructor(parent: THREE.Group, symbolScale: number) {
    this.group = new THREE.Group();
    this.symbolScale = symbolScale;
    parent.add(this.group);
  }

  update(tracks: TrackView[]): void {
    const activeNumbers = new Set<number>();

    for (const track of tracks) {
      activeNumbers.add(track.track_number);
      let obj = this.trackObjects.get(track.track_number);
      if (!obj) {
        obj = new TrackObject(this.group, this.symbolScale);
        this.trackObjects.set(track.track_number, obj);
      }
      obj.update(track);
    }

    // Remove stale tracks
    for (const [num, obj] of this.trackObjects) {
      if (!activeNumbers.has(num)) {
        obj.dispose(this.group);
        this.trackObjects.delete(num);
      }
    }
  }

  dispose(): void {
    for (const obj of this.trackObjects.values()) {
      obj.dispose(this.group);
    }
    this.trackObjects.clear();
  }
}

/** Represents a single track's visual objects. */
class TrackObject {
  private container: THREE.Group;
  private symbol: THREE.Line | null = null;
  private leader: THREE.Line;
  private historyPoints: THREE.Points;
  private hookRing: THREE.Line;
  private label: THREE.Sprite;
  private symbolScale: number;
  private currentClassification: string | null = null;

  constructor(parent: THREE.Group, symbolScale: number) {
    this.symbolScale = symbolScale;
    this.container = new THREE.Group();
    parent.add(this.container);

    // Velocity leader
    const leaderGeo = new THREE.BufferGeometry().setFromPoints([
      new THREE.Vector3(0, 0, 0),
      new THREE.Vector3(0, 0, 0),
    ]);
    const leaderMat = new THREE.LineBasicMaterial({
      color: NTDS_COLORS.scope_green_dim,
      transparent: true,
      opacity: 0.6,
    });
    this.leader = new THREE.Line(leaderGeo, leaderMat);
    this.container.add(this.leader);

    // History trail
    const histGeo = new THREE.BufferGeometry();
    const histMat = new THREE.PointsMaterial({
      color: NTDS_COLORS.scope_green_dim,
      size: HISTORY_DOT_SIZE,
      sizeAttenuation: false,
      transparent: true,
      opacity: 0.4,
    });
    this.historyPoints = new THREE.Points(histGeo, histMat);
    this.container.add(this.historyPoints);

    // Hook ring (hidden by default)
    const hookGeo = createNTDSSymbol("Unknown"); // circle
    hookGeo.scale(symbolScale * 1.5, symbolScale * 1.5, 1);
    const hookMat = new THREE.LineDashedMaterial({
      color: NTDS_COLORS.hook,
      dashSize: symbolScale * 0.3,
      gapSize: symbolScale * 0.2,
      transparent: true,
      opacity: 0.8,
    });
    this.hookRing = new THREE.Line(hookGeo, hookMat);
    this.hookRing.computeLineDistances();
    this.hookRing.visible = false;
    this.container.add(this.hookRing);

    // Track number label
    this.label = createTextSprite("", NTDS_COLORS.scope_green, symbolScale);
    this.label.position.set(symbolScale * 1.5, symbolScale * 1.5, 0);
    this.container.add(this.label);
  }

  update(track: TrackView): void {
    // Position (simulation x=East, y=North → Three.js x=East, y=North)
    this.container.position.set(track.position.x, track.position.y, 0);

    // Symbol — recreate if classification changed
    if (this.currentClassification !== track.classification) {
      if (this.symbol) {
        this.container.remove(this.symbol);
        this.symbol.geometry.dispose();
        (this.symbol.material as THREE.Material).dispose();
      }
      const geo = createNTDSSymbol(track.classification);
      geo.scale(this.symbolScale, this.symbolScale, 1);
      const color = classificationColor(track.classification);
      const mat = new THREE.LineBasicMaterial({ color });
      this.symbol = new THREE.Line(geo, mat);
      this.container.add(this.symbol);
      this.currentClassification = track.classification;
    }

    // Velocity leader — project forward
    const leaderX = Math.sin(track.heading) * track.speed * VELOCITY_LEADER_SECONDS;
    const leaderY = Math.cos(track.heading) * track.speed * VELOCITY_LEADER_SECONDS;
    const positions = this.leader.geometry.attributes.position;
    if (positions) {
      const arr = positions.array as Float32Array;
      arr[3] = leaderX;
      arr[4] = leaderY;
      arr[5] = 0;
      positions.needsUpdate = true;
    }

    // History trail
    if (track.history.length > 0) {
      const histPositions = new Float32Array(track.history.length * 3);
      for (let i = 0; i < track.history.length; i++) {
        // History positions are absolute, but we need them relative to own ship (origin)
        histPositions[i * 3] = track.history[i].x - track.position.x;
        histPositions[i * 3 + 1] = track.history[i].y - track.position.y;
        histPositions[i * 3 + 2] = 0;
      }
      this.historyPoints.geometry.setAttribute(
        "position",
        new THREE.BufferAttribute(histPositions, 3),
      );
    }

    // Hook ring
    this.hookRing.visible = track.hooked;

    // Label
    updateTextSprite(this.label, `T${track.track_number}`);
  }

  dispose(parent: THREE.Group): void {
    parent.remove(this.container);

    if (this.symbol) {
      this.symbol.geometry.dispose();
      (this.symbol.material as THREE.Material).dispose();
    }
    this.leader.geometry.dispose();
    (this.leader.material as THREE.Material).dispose();
    this.historyPoints.geometry.dispose();
    (this.historyPoints.material as THREE.Material).dispose();
    this.hookRing.geometry.dispose();
    (this.hookRing.material as THREE.Material).dispose();
    if (this.label.material instanceof THREE.SpriteMaterial && this.label.material.map) {
      this.label.material.map.dispose();
    }
    (this.label.material as THREE.Material).dispose();
  }
}

/** Create a text sprite from a canvas texture. */
function createTextSprite(
  text: string,
  color: number,
  scale: number,
): THREE.Sprite {
  const canvas = document.createElement("canvas");
  canvas.width = 128;
  canvas.height = 32;
  const ctx = canvas.getContext("2d")!;
  ctx.font = "bold 20px monospace";
  ctx.fillStyle = `#${color.toString(16).padStart(6, "0")}`;
  ctx.fillText(text, 2, 22);

  const texture = new THREE.CanvasTexture(canvas);
  texture.minFilter = THREE.LinearFilter;
  const mat = new THREE.SpriteMaterial({
    map: texture,
    transparent: true,
  });
  const sprite = new THREE.Sprite(mat);
  sprite.scale.set(scale * 4, scale, 1);
  return sprite;
}

/** Update the text on an existing sprite. */
function updateTextSprite(sprite: THREE.Sprite, text: string): void {
  const mat = sprite.material as THREE.SpriteMaterial;
  if (!mat.map) return;
  const canvas = (mat.map as THREE.CanvasTexture).image as HTMLCanvasElement;
  const ctx = canvas.getContext("2d")!;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.font = "bold 20px monospace";
  ctx.fillStyle = "#00ff41";
  ctx.fillText(text, 2, 22);
  mat.map.needsUpdate = true;
}
