/**
 * Sky — simple gradient background for the 3D world view.
 *
 * Creates a hemisphere gradient: dark navy top, haze at horizon, dark below.
 */

import * as THREE from "three";

export function createSky(): THREE.Mesh {
  const geo = new THREE.SphereGeometry(500_000, 32, 16);
  // Invert normals so we see inside
  geo.scale(-1, 1, 1);

  const vertShader = `
    varying vec3 vWorldPosition;
    void main() {
      vec4 worldPosition = modelMatrix * vec4(position, 1.0);
      vWorldPosition = worldPosition.xyz;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `;

  const fragShader = `
    varying vec3 vWorldPosition;
    void main() {
      float h = normalize(vWorldPosition).y;
      // Above horizon: dark navy → deep blue
      vec3 topColor = vec3(0.02, 0.02, 0.08);
      vec3 horizonColor = vec3(0.08, 0.12, 0.18);
      vec3 bottomColor = vec3(0.01, 0.01, 0.03);

      vec3 color;
      if (h > 0.0) {
        color = mix(horizonColor, topColor, clamp(h * 3.0, 0.0, 1.0));
      } else {
        color = mix(horizonColor, bottomColor, clamp(-h * 5.0, 0.0, 1.0));
      }
      gl_FragColor = vec4(color, 1.0);
    }
  `;

  const mat = new THREE.ShaderMaterial({
    vertexShader: vertShader,
    fragmentShader: fragShader,
    side: THREE.BackSide,
    depthWrite: false,
  });

  return new THREE.Mesh(geo, mat);
}
