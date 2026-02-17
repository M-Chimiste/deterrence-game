/**
 * OceanPlane — water surface at sea level for the 3D world view.
 */

import * as THREE from "three";

const OCEAN_SIZE = 400_000; // meters, covers well beyond radar range

export function createOceanPlane(): THREE.Mesh {
  const geo = new THREE.PlaneGeometry(OCEAN_SIZE, OCEAN_SIZE);
  const mat = new THREE.MeshPhongMaterial({
    color: 0x001830,
    transparent: true,
    opacity: 0.85,
    shininess: 80,
    specular: 0x112244,
  });

  const mesh = new THREE.Mesh(geo, mat);
  // Plane defaults to XY facing +Z. Rotate to XZ ground plane (Y up).
  mesh.rotation.x = -Math.PI / 2;
  mesh.position.y = 0; // sea level
  return mesh;
}
