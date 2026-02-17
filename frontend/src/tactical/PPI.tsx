/**
 * Plan Position Indicator (PPI) — the main radar scope display.
 *
 * Renders a Three.js scene with:
 * - Dark green circle background
 * - Range rings and bearing tick marks
 * - Rotating sweep line with phosphor trail
 * - Track symbols (NTDS) via TrackRenderer
 * - Click-to-hook interaction
 */

import { useEffect, useRef } from "preact/hooks";
import * as THREE from "three";
import { useGameStore } from "../store/gameState";
import { getInterpolatedTracks } from "../store/interpolation";
import { TrackRenderer } from "./tracks";
import { CoastlineRenderer } from "./CoastlineRenderer";
import { NTDS_COLORS } from "./symbology";
import { hookTrack, unhookTrack } from "../ipc/bridge";

const DISPLAY_RANGE = 185_000; // meters (100nm)
const RANGE_RING_INTERVAL = 46_300; // ~25nm
const NUM_RANGE_RINGS = 4;
const BEARING_TICK_COUNT = 36; // every 10 degrees
const SWEEP_TRAIL_ANGLE = 0.5; // radians (~30°) of phosphor trail
const SYMBOL_SCALE = 2500; // meters — NTDS symbol size on scope

export function PPI() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sceneRef = useRef<PPIScene | null>(null);

  useEffect(() => {
    if (!canvasRef.current) return;
    const scene = new PPIScene(canvasRef.current);
    sceneRef.current = scene;

    let animFrame: number;
    const animate = () => {
      const state = useGameStore.getState();
      if (state.snapshot) {
        scene.update(state.snapshot);
      }
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

  const handleClick = (e: MouseEvent) => {
    const scene = sceneRef.current;
    if (!scene || !canvasRef.current) return;
    scene.handleClick(e, canvasRef.current);
  };

  return (
    <canvas
      ref={canvasRef}
      onClick={handleClick}
      style={{
        width: "100%",
        height: "100%",
        display: "block",
        cursor: "crosshair",
      }}
    />
  );
}

class PPIScene {
  private renderer: THREE.WebGLRenderer;
  private camera: THREE.OrthographicCamera;
  private scene: THREE.Scene;
  private staticGroup: THREE.Group;
  private dynamicGroup: THREE.Group;
  private sweepLine: THREE.Line;
  private sweepTrail: THREE.Mesh;
  private trackRenderer: TrackRenderer;
  private coastlineRenderer: CoastlineRenderer;
  private coastlinesLoaded = false;

  constructor(canvas: HTMLCanvasElement) {
    // Renderer
    this.renderer = new THREE.WebGLRenderer({
      canvas,
      antialias: true,
      alpha: false,
    });
    this.renderer.setClearColor(NTDS_COLORS.background, 1);
    this.renderer.setPixelRatio(window.devicePixelRatio);

    // Scene
    this.scene = new THREE.Scene();

    // Camera (orthographic, centered at origin)
    const aspect = canvas.clientWidth / canvas.clientHeight;
    const halfH = DISPLAY_RANGE;
    const halfW = halfH * aspect;
    this.camera = new THREE.OrthographicCamera(
      -halfW, halfW, halfH, -halfH, -1, 1,
    );

    // Groups
    this.staticGroup = new THREE.Group();
    this.dynamicGroup = new THREE.Group();
    this.scene.add(this.staticGroup);
    this.scene.add(this.dynamicGroup);

    // Static geometry
    this.createRangeRings();
    this.createBearingTicks();
    this.createCardinalLabels();
    this.createScopeCircle();

    // Sweep line
    this.sweepLine = this.createSweepLine();
    this.dynamicGroup.add(this.sweepLine);

    // Sweep trail (phosphor decay)
    this.sweepTrail = this.createSweepTrail();
    this.dynamicGroup.add(this.sweepTrail);

    // Track renderer
    this.trackRenderer = new TrackRenderer(this.dynamicGroup, SYMBOL_SCALE);

    // Coastline renderer (added to static group — coastlines don't change)
    this.coastlineRenderer = new CoastlineRenderer(this.staticGroup);

    this.resize();
  }

  update(snapshot: ReturnType<typeof useGameStore.getState>["snapshot"]): void {
    if (!snapshot) return;

    // Update sweep line rotation (bearing 0=North → Three.js rotation)
    // In our coord system: sweep_angle is radians from North clockwise
    // Three.js rotation is counter-clockwise from +x axis
    // bearing_to_rotation: rotation = PI/2 - sweep_angle
    const rotation = Math.PI / 2 - snapshot.radar.sweep_angle;
    this.sweepLine.rotation.z = rotation;
    this.sweepTrail.rotation.z = rotation;

    // Load coastlines once when terrain data arrives
    if (!this.coastlinesLoaded) {
      const terrainData = useGameStore.getState().terrainData;
      if (terrainData && terrainData.coastlines.length > 0) {
        this.coastlineRenderer.setCoastlines(terrainData.coastlines);
        this.coastlinesLoaded = true;
      }
    }

    // Clear coastlines when returning to menu
    if (snapshot.phase === "MainMenu" && this.coastlinesLoaded) {
      this.coastlineRenderer.clear();
      this.coastlinesLoaded = false;
    }

    // Update tracks with interpolation
    const tracks = getInterpolatedTracks();
    this.trackRenderer.update(tracks ?? snapshot.tracks);
  }

  render(): void {
    this.renderer.render(this.scene, this.camera);
  }

  resize(): void {
    const canvas = this.renderer.domElement;
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    if (w === 0 || h === 0) return;

    this.renderer.setSize(w, h, false);
    const aspect = w / h;
    const halfH = DISPLAY_RANGE;
    const halfW = halfH * aspect;
    this.camera.left = -halfW;
    this.camera.right = halfW;
    this.camera.top = halfH;
    this.camera.bottom = -halfH;
    this.camera.updateProjectionMatrix();
  }

  handleClick(event: MouseEvent, canvas: HTMLCanvasElement): void {
    const rect = canvas.getBoundingClientRect();
    const mouse = new THREE.Vector2(
      ((event.clientX - rect.left) / rect.width) * 2 - 1,
      -((event.clientY - rect.top) / rect.height) * 2 + 1,
    );

    // Convert mouse to world coordinates
    const worldX = mouse.x * (this.camera.right - this.camera.left) / 2
      + (this.camera.right + this.camera.left) / 2;
    const worldY = mouse.y * (this.camera.top - this.camera.bottom) / 2
      + (this.camera.top + this.camera.bottom) / 2;

    // Find nearest track
    const snapshot = useGameStore.getState().snapshot;
    if (!snapshot) return;

    const clickThreshold = SYMBOL_SCALE * 3;
    let nearestTrack: number | null = null;
    let nearestDist = Infinity;

    for (const track of snapshot.tracks) {
      const dx = track.position.x - worldX;
      const dy = track.position.y - worldY;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < clickThreshold && dist < nearestDist) {
        nearestDist = dist;
        nearestTrack = track.track_number;
      }
    }

    if (nearestTrack !== null) {
      hookTrack(nearestTrack);
    } else {
      unhookTrack();
    }
  }

  dispose(): void {
    this.trackRenderer.dispose();
    this.coastlineRenderer.dispose();
    this.renderer.dispose();
    // Traverse and dispose all geometries/materials
    this.scene.traverse((obj) => {
      if (obj instanceof THREE.Mesh || obj instanceof THREE.Line || obj instanceof THREE.Points) {
        obj.geometry.dispose();
        if (Array.isArray(obj.material)) {
          obj.material.forEach((m: THREE.Material) => m.dispose());
        } else {
          (obj.material as THREE.Material).dispose();
        }
      }
    });
  }

  private createScopeCircle(): void {
    const points: THREE.Vector3[] = [];
    const segments = 128;
    for (let i = 0; i <= segments; i++) {
      const angle = (i / segments) * Math.PI * 2;
      points.push(
        new THREE.Vector3(
          Math.cos(angle) * DISPLAY_RANGE,
          Math.sin(angle) * DISPLAY_RANGE,
          0,
        ),
      );
    }
    const geo = new THREE.BufferGeometry().setFromPoints(points);
    const mat = new THREE.LineBasicMaterial({
      color: NTDS_COLORS.scope_green_dim,
      transparent: true,
      opacity: 0.5,
    });
    this.staticGroup.add(new THREE.Line(geo, mat));
  }

  private createRangeRings(): void {
    for (let i = 1; i <= NUM_RANGE_RINGS; i++) {
      const radius = RANGE_RING_INTERVAL * i;
      const points: THREE.Vector3[] = [];
      const segments = 128;
      for (let j = 0; j <= segments; j++) {
        const angle = (j / segments) * Math.PI * 2;
        points.push(
          new THREE.Vector3(
            Math.cos(angle) * radius,
            Math.sin(angle) * radius,
            0,
          ),
        );
      }
      const geo = new THREE.BufferGeometry().setFromPoints(points);
      const mat = new THREE.LineBasicMaterial({
        color: NTDS_COLORS.range_ring,
        transparent: true,
        opacity: 0.4,
      });
      this.staticGroup.add(new THREE.Line(geo, mat));
    }
  }

  private createBearingTicks(): void {
    const innerRadius = DISPLAY_RANGE * 0.97;
    const outerRadius = DISPLAY_RANGE;

    for (let i = 0; i < BEARING_TICK_COUNT; i++) {
      const angle = (i / BEARING_TICK_COUNT) * Math.PI * 2;
      const points = [
        new THREE.Vector3(Math.cos(angle) * innerRadius, Math.sin(angle) * innerRadius, 0),
        new THREE.Vector3(Math.cos(angle) * outerRadius, Math.sin(angle) * outerRadius, 0),
      ];
      const geo = new THREE.BufferGeometry().setFromPoints(points);
      const mat = new THREE.LineBasicMaterial({
        color: NTDS_COLORS.bearing_tick,
        transparent: true,
        opacity: 0.5,
      });
      this.staticGroup.add(new THREE.Line(geo, mat));
    }
  }

  private createCardinalLabels(): void {
    const labels = [
      { text: "N", angle: Math.PI / 2 },     // North = +y
      { text: "E", angle: 0 },                // East = +x
      { text: "S", angle: -Math.PI / 2 },     // South = -y
      { text: "W", angle: Math.PI },           // West = -x
    ];

    for (const { text, angle } of labels) {
      const radius = DISPLAY_RANGE * 0.92;
      const sprite = this.createLabel(text, NTDS_COLORS.scope_green_dim);
      sprite.position.set(
        Math.cos(angle) * radius,
        Math.sin(angle) * radius,
        0,
      );
      this.staticGroup.add(sprite);
    }
  }

  private createLabel(text: string, color: number): THREE.Sprite {
    const canvas = document.createElement("canvas");
    canvas.width = 64;
    canvas.height = 64;
    const ctx = canvas.getContext("2d")!;
    ctx.font = "bold 48px monospace";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillStyle = `#${color.toString(16).padStart(6, "0")}`;
    ctx.fillText(text, 32, 32);

    const texture = new THREE.CanvasTexture(canvas);
    texture.minFilter = THREE.LinearFilter;
    const mat = new THREE.SpriteMaterial({
      map: texture,
      transparent: true,
    });
    const sprite = new THREE.Sprite(mat);
    const s = DISPLAY_RANGE * 0.06;
    sprite.scale.set(s, s, 1);
    return sprite;
  }

  private createSweepLine(): THREE.Line {
    const points = [
      new THREE.Vector3(0, 0, 0),
      new THREE.Vector3(DISPLAY_RANGE, 0, 0),
    ];
    const geo = new THREE.BufferGeometry().setFromPoints(points);
    const mat = new THREE.LineBasicMaterial({
      color: NTDS_COLORS.sweep,
      transparent: true,
      opacity: 0.8,
    });
    return new THREE.Line(geo, mat);
  }

  private createSweepTrail(): THREE.Mesh {
    // Create a ring sector mesh for the phosphor decay trail
    const segments = 32;
    const positions: number[] = [];
    const colors: number[] = [];

    const color = new THREE.Color(NTDS_COLORS.sweep);

    for (let i = 0; i <= segments; i++) {
      const t = i / segments;
      // Trail goes from current angle backwards
      const angle = -t * SWEEP_TRAIL_ANGLE;
      const alpha = 1.0 - t; // Fade from bright to transparent

      // Inner point (at origin)
      positions.push(0, 0, 0);
      colors.push(color.r, color.g, color.b, 0);

      // Outer point (at display range)
      positions.push(
        Math.cos(angle) * DISPLAY_RANGE,
        Math.sin(angle) * DISPLAY_RANGE,
        0,
      );
      colors.push(color.r, color.g, color.b, alpha * 0.15);
    }

    // Build indices for triangle strip
    const indices: number[] = [];
    for (let i = 0; i < segments; i++) {
      const a = i * 2;
      const b = i * 2 + 1;
      const c = (i + 1) * 2;
      const d = (i + 1) * 2 + 1;
      indices.push(a, b, c);
      indices.push(b, d, c);
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
    geo.setAttribute("color", new THREE.Float32BufferAttribute(colors, 4));
    geo.setIndex(indices);

    const mat = new THREE.MeshBasicMaterial({
      vertexColors: true,
      transparent: true,
      side: THREE.DoubleSide,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });

    return new THREE.Mesh(geo, mat);
  }
}
