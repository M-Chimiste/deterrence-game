/**
 * CameraController — orbit camera for the 3D world view.
 *
 * Overhead mode (default): high up, looking down at ~60 degrees.
 * Mouse drag to orbit, scroll to zoom.
 */

import * as THREE from "three";

const DEFAULT_DISTANCE = 200_000; // meters from target
const MIN_DISTANCE = 10_000;
const MAX_DISTANCE = 400_000;
const DEFAULT_POLAR = Math.PI / 6; // 30 degrees from vertical (looking down at 60deg)
const MIN_POLAR = 0.05; // almost top-down
const MAX_POLAR = Math.PI / 2.2; // near horizon
const DRAG_SPEED = 0.003;
const ZOOM_SPEED = 0.1;

export class CameraController {
  private camera: THREE.PerspectiveCamera;
  private target = new THREE.Vector3(0, 0, 0);
  private azimuth = 0; // radians around Y axis
  private polar = DEFAULT_POLAR; // radians from vertical
  private distance = DEFAULT_DISTANCE;
  private isDragging = false;
  private lastMouse = { x: 0, y: 0 };

  constructor(camera: THREE.PerspectiveCamera, canvas: HTMLCanvasElement) {
    this.camera = camera;
    this.updateCameraPosition();

    canvas.addEventListener("mousedown", this.onMouseDown);
    canvas.addEventListener("mousemove", this.onMouseMove);
    canvas.addEventListener("mouseup", this.onMouseUp);
    canvas.addEventListener("mouseleave", this.onMouseUp);
    canvas.addEventListener("wheel", this.onWheel, { passive: true });
  }

  updateCameraPosition(): void {
    const x =
      this.target.x +
      this.distance * Math.sin(this.polar) * Math.sin(this.azimuth);
    const y = this.target.y + this.distance * Math.cos(this.polar);
    const z =
      this.target.z +
      this.distance * Math.sin(this.polar) * Math.cos(this.azimuth);

    this.camera.position.set(x, y, z);
    this.camera.lookAt(this.target);
  }

  dispose(canvas: HTMLCanvasElement): void {
    canvas.removeEventListener("mousedown", this.onMouseDown);
    canvas.removeEventListener("mousemove", this.onMouseMove);
    canvas.removeEventListener("mouseup", this.onMouseUp);
    canvas.removeEventListener("mouseleave", this.onMouseUp);
    canvas.removeEventListener("wheel", this.onWheel);
  }

  private onMouseDown = (e: MouseEvent): void => {
    this.isDragging = true;
    this.lastMouse.x = e.clientX;
    this.lastMouse.y = e.clientY;
  };

  private onMouseMove = (e: MouseEvent): void => {
    if (!this.isDragging) return;

    const dx = e.clientX - this.lastMouse.x;
    const dy = e.clientY - this.lastMouse.y;
    this.lastMouse.x = e.clientX;
    this.lastMouse.y = e.clientY;

    this.azimuth -= dx * DRAG_SPEED;
    this.polar = Math.max(
      MIN_POLAR,
      Math.min(MAX_POLAR, this.polar + dy * DRAG_SPEED),
    );

    this.updateCameraPosition();
  };

  private onMouseUp = (): void => {
    this.isDragging = false;
  };

  private onWheel = (e: WheelEvent): void => {
    const factor = 1 + Math.sign(e.deltaY) * ZOOM_SPEED;
    this.distance = Math.max(
      MIN_DISTANCE,
      Math.min(MAX_DISTANCE, this.distance * factor),
    );
    this.updateCameraPosition();
  };
}
