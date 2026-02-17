/**
 * EntityMarkers — 3D representations of tracks, missiles, and own-ship.
 *
 * Coordinate mapping: sim(x,y,z) -> three.js(x, z, -y).
 */

import * as THREE from "three";
import type { TrackView } from "../ipc/state";

const MARKER_SIZE = 800; // meters — visible at world scale

// Classification colors (match NTDS)
const COLOR_HOSTILE = 0xff3333;
const COLOR_SUSPECT = 0xffaa00;
const COLOR_UNKNOWN = 0x00ff41;
const COLOR_FRIENDLY = 0x3366ff;
const COLOR_OWNSHIP = 0x00ff41;

function classificationColor(classification: string): number {
  switch (classification) {
    case "Hostile":
      return COLOR_HOSTILE;
    case "Suspect":
      return COLOR_SUSPECT;
    case "Friend":
    case "AssumedFriend":
      return COLOR_FRIENDLY;
    default:
      return COLOR_UNKNOWN;
  }
}

export class EntityMarkers {
  private group: THREE.Group;
  private trackMarkers: Map<number, THREE.Mesh> = new Map();
  private ownShipMarker: THREE.Mesh;

  // Shared geometries for pooling
  private diamondGeo: THREE.BufferGeometry;
  private sphereGeo: THREE.BufferGeometry;

  constructor(parent: THREE.Group) {
    this.group = new THREE.Group();
    parent.add(this.group);

    // Diamond geometry for hostile tracks
    this.diamondGeo = new THREE.OctahedronGeometry(MARKER_SIZE, 0);
    // Sphere geometry for other tracks
    this.sphereGeo = new THREE.SphereGeometry(MARKER_SIZE * 0.7, 8, 6);

    // Own-ship marker (green cone)
    const ownGeo = new THREE.ConeGeometry(MARKER_SIZE, MARKER_SIZE * 2, 4);
    const ownMat = new THREE.MeshPhongMaterial({
      color: COLOR_OWNSHIP,
      emissive: COLOR_OWNSHIP,
      emissiveIntensity: 0.3,
    });
    this.ownShipMarker = new THREE.Mesh(ownGeo, ownMat);
    this.ownShipMarker.position.set(0, MARKER_SIZE, 0); // slightly above ground
    this.group.add(this.ownShipMarker);
  }

  update(tracks: TrackView[]): void {
    const activeTracks = new Set<number>();

    for (const track of tracks) {
      activeTracks.add(track.track_number);

      let marker = this.trackMarkers.get(track.track_number);
      if (!marker) {
        // Create new marker
        const isHostile = track.classification === "Hostile";
        const geo = isHostile ? this.diamondGeo : this.sphereGeo;
        const color = classificationColor(track.classification);
        const mat = new THREE.MeshPhongMaterial({
          color,
          emissive: color,
          emissiveIntensity: 0.4,
        });
        marker = new THREE.Mesh(geo, mat);
        this.trackMarkers.set(track.track_number, marker);
        this.group.add(marker);
      } else {
        // Update color if classification changed
        const color = classificationColor(track.classification);
        const mat = marker.material as THREE.MeshPhongMaterial;
        if (mat.color.getHex() !== color) {
          mat.color.setHex(color);
          mat.emissive.setHex(color);
        }
      }

      // Update position: sim(x,y,z) -> three.js(x, z, -y)
      marker.position.set(
        track.position.x,
        track.position.z, // altitude
        -track.position.y,
      );
    }

    // Remove markers for tracks that no longer exist
    for (const [trackNum, marker] of this.trackMarkers) {
      if (!activeTracks.has(trackNum)) {
        this.group.remove(marker);
        (marker.material as THREE.Material).dispose();
        this.trackMarkers.delete(trackNum);
      }
    }
  }

  dispose(): void {
    for (const [, marker] of this.trackMarkers) {
      (marker.material as THREE.Material).dispose();
    }
    this.trackMarkers.clear();
    this.diamondGeo.dispose();
    this.sphereGeo.dispose();
    (this.ownShipMarker.material as THREE.Material).dispose();
    this.ownShipMarker.geometry.dispose();
  }
}
