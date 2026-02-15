/**
 * NTDS (Naval Tactical Data System) symbol definitions and colors.
 * Symbols are defined as Three.js line geometries for rendering on the PPI scope.
 */

import * as THREE from "three";
import type { Classification } from "../ipc/state";

export const NTDS_COLORS = {
  scope_green: 0x00ff41,
  scope_green_dim: 0x006618,
  hostile: 0xff3333,
  friendly: 0x3399ff,
  neutral: 0x33ff33,
  unknown: 0xffff33,
  suspect: 0xff9933,
  background: 0x001a00,
  sweep: 0x00ff41,
  range_ring: 0x003300,
  bearing_tick: 0x002800,
  hook: 0x00ffff,
} as const;

/** Create NTDS symbol line geometry for a classification. */
export function createNTDSSymbol(
  classification: Classification,
): THREE.BufferGeometry {
  switch (classification) {
    case "Unknown":
    case "Pending":
      return createCircle(1.0);
    case "Hostile":
      return createDiamond(1.0);
    case "Friend":
    case "AssumedFriend":
      return createSemicircle(1.0);
    case "Neutral":
      return createSquare(1.0);
    case "Suspect":
      return createQuatrefoil(1.0);
    default:
      return createCircle(1.0);
  }
}

/** Get the display color for a classification. */
export function classificationColor(classification: Classification): number {
  switch (classification) {
    case "Hostile":
      return NTDS_COLORS.hostile;
    case "Friend":
    case "AssumedFriend":
      return NTDS_COLORS.friendly;
    case "Neutral":
      return NTDS_COLORS.neutral;
    case "Suspect":
      return NTDS_COLORS.suspect;
    case "Unknown":
    case "Pending":
    default:
      return NTDS_COLORS.unknown;
  }
}

function createCircle(radius: number): THREE.BufferGeometry {
  const points: THREE.Vector3[] = [];
  const segments = 24;
  for (let i = 0; i <= segments; i++) {
    const angle = (i / segments) * Math.PI * 2;
    points.push(
      new THREE.Vector3(Math.cos(angle) * radius, Math.sin(angle) * radius, 0),
    );
  }
  return new THREE.BufferGeometry().setFromPoints(points);
}

function createDiamond(size: number): THREE.BufferGeometry {
  const points = [
    new THREE.Vector3(0, size, 0),
    new THREE.Vector3(size, 0, 0),
    new THREE.Vector3(0, -size, 0),
    new THREE.Vector3(-size, 0, 0),
    new THREE.Vector3(0, size, 0),
  ];
  return new THREE.BufferGeometry().setFromPoints(points);
}

function createSemicircle(radius: number): THREE.BufferGeometry {
  const points: THREE.Vector3[] = [];
  const segments = 12;
  for (let i = 0; i <= segments; i++) {
    const angle = (i / segments) * Math.PI;
    points.push(
      new THREE.Vector3(Math.cos(angle) * radius, Math.sin(angle) * radius, 0),
    );
  }
  points.push(new THREE.Vector3(-radius, 0, 0));
  return new THREE.BufferGeometry().setFromPoints(points);
}

function createSquare(size: number): THREE.BufferGeometry {
  const points = [
    new THREE.Vector3(-size, size, 0),
    new THREE.Vector3(size, size, 0),
    new THREE.Vector3(size, -size, 0),
    new THREE.Vector3(-size, -size, 0),
    new THREE.Vector3(-size, size, 0),
  ];
  return new THREE.BufferGeometry().setFromPoints(points);
}

function createQuatrefoil(radius: number): THREE.BufferGeometry {
  const points: THREE.Vector3[] = [];
  const segments = 8;
  const r = radius * 0.5;
  const offsets = [
    { x: r, y: 0 },
    { x: 0, y: r },
    { x: -r, y: 0 },
    { x: 0, y: -r },
  ];
  for (const off of offsets) {
    for (let i = 0; i <= segments; i++) {
      const angle = (i / segments) * Math.PI * 2;
      points.push(
        new THREE.Vector3(off.x + Math.cos(angle) * r, off.y + Math.sin(angle) * r, 0),
      );
    }
  }
  return new THREE.BufferGeometry().setFromPoints(points);
}
