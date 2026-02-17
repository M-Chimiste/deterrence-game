/**
 * WorldScene — main 3D world view Preact component.
 *
 * Renders a Three.js perspective scene with terrain, ocean, entity markers,
 * missile trails, intercept effects, and orbit camera controls.
 *
 * Coordinate mapping: sim(x,y,z) → three.js(x=East, y=Up, z=-North).
 */

import { useEffect, useRef } from "preact/hooks";
import * as THREE from "three";
import { useGameStore } from "../store/gameState";
import { getInterpolatedTracks } from "../store/interpolation";
import { createSky } from "./Sky";
import { createOceanPlane } from "./OceanPlane";
import { createTerrainMesh } from "./TerrainMesh";
import { CameraController } from "./CameraController";
import { EntityMarkers } from "./EntityMarkers";
import { MissileTrails } from "./MissileTrails";
import { InterceptEffects } from "./InterceptEffects";

export function WorldScene() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sceneRef = useRef<WorldSceneImpl | null>(null);

  useEffect(() => {
    if (!canvasRef.current) return;
    const scene = new WorldSceneImpl(canvasRef.current);
    sceneRef.current = scene;

    let animFrame: number;
    const animate = () => {
      scene.update();
      scene.render();
      animFrame = requestAnimationFrame(animate);
    };
    animFrame = requestAnimationFrame(animate);

    const onResize = () => scene.resize();
    window.addEventListener("resize", onResize);

    return () => {
      cancelAnimationFrame(animFrame);
      window.removeEventListener("resize", onResize);
      scene.dispose();
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      style={{
        width: "100%",
        height: "100%",
        display: "block",
      }}
    />
  );
}

class WorldSceneImpl {
  private renderer: THREE.WebGLRenderer;
  private camera: THREE.PerspectiveCamera;
  private scene: THREE.Scene;
  private cameraController: CameraController;
  private canvas: HTMLCanvasElement;

  // Dynamic groups
  private dynamicGroup: THREE.Group;
  private entityMarkers: EntityMarkers;
  private missileTrails: MissileTrails;
  private interceptEffects: InterceptEffects;

  // Terrain (built once per mission)
  private terrainMesh: THREE.Mesh | null = null;
  private terrainBuilt = false;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;

    // Renderer
    this.renderer = new THREE.WebGLRenderer({
      canvas,
      antialias: true,
      alpha: false,
    });
    this.renderer.setClearColor(0x000008, 1);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

    // Scene
    this.scene = new THREE.Scene();

    // Camera
    const aspect = canvas.clientWidth / Math.max(canvas.clientHeight, 1);
    this.camera = new THREE.PerspectiveCamera(60, aspect, 100, 600_000);

    // Camera controller
    this.cameraController = new CameraController(this.camera, canvas);

    // Lighting
    const ambient = new THREE.AmbientLight(0x404050, 0.4);
    this.scene.add(ambient);

    const sun = new THREE.DirectionalLight(0xffffff, 0.8);
    sun.position.set(100_000, 150_000, -50_000); // SE, angled down
    this.scene.add(sun);

    // Sky
    this.scene.add(createSky());

    // Ocean
    this.scene.add(createOceanPlane());

    // Dynamic group for entities
    this.dynamicGroup = new THREE.Group();
    this.scene.add(this.dynamicGroup);

    // Entity systems
    this.entityMarkers = new EntityMarkers(this.dynamicGroup);
    this.missileTrails = new MissileTrails(this.dynamicGroup);
    this.interceptEffects = new InterceptEffects(this.dynamicGroup);

    this.resize();
  }

  update(): void {
    const state = useGameStore.getState();
    const snapshot = state.snapshot;
    if (!snapshot) return;

    // Build terrain mesh once when terrain data arrives
    if (!this.terrainBuilt && state.terrainData) {
      this.terrainMesh = createTerrainMesh(state.terrainData);
      if (this.terrainMesh) {
        this.scene.add(this.terrainMesh);
      }
      this.terrainBuilt = true;
    }

    // Clear terrain when returning to menu
    if (snapshot.phase === "MainMenu" && this.terrainBuilt) {
      if (this.terrainMesh) {
        this.scene.remove(this.terrainMesh);
        this.terrainMesh.geometry.dispose();
        (this.terrainMesh.material as THREE.Material).dispose();
        this.terrainMesh = null;
      }
      this.terrainBuilt = false;
    }

    // Update entities with interpolated tracks
    const tracks = getInterpolatedTracks();
    this.entityMarkers.update(tracks ?? snapshot.tracks);
    this.missileTrails.update(tracks ?? snapshot.tracks);

    // Process intercept effects from audio events
    if (snapshot.audio_events.length > 0) {
      const trackPositions = new Map<
        number,
        { x: number; y: number; z: number }
      >();
      for (const track of snapshot.tracks) {
        trackPositions.set(track.track_number, track.position);
      }
      this.interceptEffects.processEvents(
        snapshot.audio_events,
        trackPositions,
      );
    }
    this.interceptEffects.update();
  }

  render(): void {
    this.renderer.render(this.scene, this.camera);
  }

  resize(): void {
    const w = this.canvas.clientWidth;
    const h = this.canvas.clientHeight;
    if (w === 0 || h === 0) return;

    this.renderer.setSize(w, h, false);
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
  }

  dispose(): void {
    this.cameraController.dispose(this.canvas);
    this.entityMarkers.dispose();
    this.missileTrails.dispose();
    this.interceptEffects.dispose();
    if (this.terrainMesh) {
      this.terrainMesh.geometry.dispose();
      (this.terrainMesh.material as THREE.Material).dispose();
    }
    this.renderer.dispose();
    // Traverse and dispose all remaining geometries/materials
    this.scene.traverse((obj) => {
      if (
        obj instanceof THREE.Mesh ||
        obj instanceof THREE.Line ||
        obj instanceof THREE.Points
      ) {
        obj.geometry.dispose();
        if (Array.isArray(obj.material)) {
          obj.material.forEach((m: THREE.Material) => m.dispose());
        } else {
          (obj.material as THREE.Material).dispose();
        }
      }
    });
  }
}
