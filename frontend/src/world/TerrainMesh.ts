/**
 * TerrainMesh — builds 3D terrain geometry from heightmap data.
 *
 * Creates a displaced PlaneGeometry with vertex coloring by elevation.
 * Coordinate mapping: sim(x,y,z) -> three.js(x, z_up, -y).
 */

import * as THREE from "three";
import type { TerrainDataPayload } from "../ipc/state";

/** Elevation color ramp: ocean=blue, shore=sand, low=green, high=brown, peak=white. */
function elevationColor(elevation: number, maxElev: number): THREE.Color {
  if (elevation <= 0) return new THREE.Color(0x001830); // ocean

  const t = Math.min(elevation / Math.max(maxElev, 1), 1.0);

  if (t < 0.05) return new THREE.Color(0x8b7355); // shore/sand
  if (t < 0.25) return new THREE.Color(0x2d5a1e); // low green
  if (t < 0.5) return new THREE.Color(0x3a6b28); // green
  if (t < 0.75) return new THREE.Color(0x6b4423); // brown
  if (t < 0.9) return new THREE.Color(0x8b6b43); // light brown
  return new THREE.Color(0xcccccc); // snow/peak
}

const METERS_PER_DEGREE = 111_320;

export function createTerrainMesh(
  data: TerrainDataPayload,
): THREE.Mesh | null {
  if (data.elevations.length === 0) return null;

  const w = data.width;
  const h = data.height;

  // Compute physical dimensions of the terrain in meters
  const cosLat = Math.cos((data.center_lat * Math.PI) / 180);
  const lonSpanDeg = (w * data.cell_size_arcsec) / 3600;
  const latSpanDeg = (h * data.cell_size_arcsec) / 3600;
  const widthM = lonSpanDeg * METERS_PER_DEGREE * cosLat;
  const heightM = latSpanDeg * METERS_PER_DEGREE;

  // Offset of grid SW corner from projection center (in sim-space meters)
  const swOffsetX =
    (data.origin_lon - data.center_lon) * METERS_PER_DEGREE * cosLat;
  const swOffsetY = (data.origin_lat - data.center_lat) * METERS_PER_DEGREE;

  // Create plane: widthM x heightM, subdivided to match grid resolution
  const geo = new THREE.PlaneGeometry(widthM, heightM, w - 1, h - 1);

  const posAttr = geo.attributes.position;
  const colors = new Float32Array(posAttr.count * 3);
  const maxElev = data.max_elevation;

  for (let i = 0; i < posAttr.count; i++) {
    // PlaneGeometry default is in XY plane. Vertices arranged row-by-row.
    // grid row 0 = north edge (top of image), row h-1 = south edge.
    const gridRow = Math.floor(i / w);
    const gridCol = i % w;

    const elevation = data.elevations[gridRow * w + gridCol];

    // PlaneGeometry generates vertices in row-major order (top-left to bottom-right)
    // We need to set X and Y on the plane, then later rotate to XZ.
    // But it's simpler to just directly set X, Y, Z in world space:
    //   three.x = sim.x (East)
    //   three.y = sim.z (Up = elevation)
    //   three.z = -sim.y (South, since Three.js z goes toward camera)

    // Grid col 0 = west edge, col w-1 = east edge
    const simX = swOffsetX + (gridCol / (w - 1)) * widthM;
    // Grid row 0 = north edge, row h-1 = south edge
    const simY = swOffsetY + heightM - (gridRow / (h - 1)) * heightM;

    posAttr.setXYZ(
      i,
      simX, // three.x = sim East
      Math.max(elevation, 0), // three.y = altitude (clamp ocean to 0)
      -simY, // three.z = -sim North
    );

    // Vertex color
    const c = elevationColor(elevation, maxElev);
    colors[i * 3] = c.r;
    colors[i * 3 + 1] = c.g;
    colors[i * 3 + 2] = c.b;
  }

  geo.setAttribute("color", new THREE.Float32BufferAttribute(colors, 3));
  geo.computeVertexNormals();

  // Remove the default UV-based index pattern since we repositioned vertices
  // (PlaneGeometry indices are still valid — they connect adjacent vertices)

  const mat = new THREE.MeshPhongMaterial({
    vertexColors: true,
    flatShading: true,
    shininess: 5,
  });

  return new THREE.Mesh(geo, mat);
}
