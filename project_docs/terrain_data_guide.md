# DETERRENCE — Terrain Data Sourcing & Processing Guide

## Final Recommendations

| Data Type | Source | Resolution | License | Use Case |
|---|---|---|---|---|
| **Land Elevation** | NASADEM (NASA JPL) | 30m (1 arc-second) | Public domain | Ground theaters: heightmaps, LOS, radar masking |
| **Ocean Bathymetry** | GEBCO 2025 Grid | ~450m (15 arc-second) | Public domain (attribution) | Naval theaters: seafloor rendering, water depth |
| **Coastlines & Vectors** | Natural Earth | 1:10m / 1:50m / 1:110m | Public domain | PPI overlays, coastlines, boundaries, shipping |
| **Terrain Textures** | Procedurally generated | N/A | N/A | Derived from elevation + slope data at build time |

All three primary sources are free for commercial use with no royalties or permission requests required. GEBCO asks for attribution in published materials. Natural Earth and NASADEM have zero restrictions.

---

## Source 1: NASADEM — Land Elevation

### What It Is

NASADEM is NASA's 2020 reprocessing of the original Shuttle Radar Topography Mission (SRTM) data from February 2000. It improves on the classic SRTM product through better phase-unwrapping algorithms, ICESat laser altimetry ground control, and void-filling from ASTER GDEM — without introducing any commercial-license-restricted data. Global coverage from 56°S to 60°N at 1 arc-second (~30m) resolution.

Vertical accuracy: ~6-7m RMSE. This is more than sufficient for radar LOS and terrain masking calculations where the relevant features (ridgelines, valleys, mesa edges) are tens to hundreds of meters in relief.

**Important:** NASADEM is a Digital Surface Model (DSM), not a bare-earth Digital Terrain Model (DTM). Elevation values include buildings, vegetation canopy, and infrastructure. For DETERRENCE this is actually correct behavior — real radar bounces off these surfaces too, so the DSM represents what a radar would "see" for line-of-sight purposes.

### License

Public domain. NASA's data use policy states: "Unless otherwise noted, all NASA-produced data may be used for any purpose without prior permission." NASADEM falls under this policy. No attribution legally required, though a credits screen acknowledgment is good practice.

**Do NOT use CGIAR-CSI SRTM V4** — it uses the same underlying radar data but its post-processing is restricted to non-commercial use without written permission from CIAT.

### Where to Download

**Option A: OpenTopography (Recommended for bulk download)**
- URL: https://portal.opentopography.org/raster?opentopoID=OT.032021.4326.2
- Format: GeoTIFF via web GUI or API
- API example for a specific region:
  ```
  https://portal.opentopography.org/API/globaldem?demtype=NASADEM&south=25&north=28&west=50&east=57&outputFormat=GTiff&API_Key=YOUR_KEY
  ```
- Requires free API key registration

**Option B: NASA LP DAAC (Original source)**
- URL: https://e4ftl01.cr.usgs.gov/MEASURES/NASADEM_HGT.001/2000.02.11/
- Format: HGT files (raw 16-bit integer arrays) in 1°×1° tiles
- File naming: `NASADEM_HGT_n25e051.zip` (southwest corner coordinates)
- Requires free NASA Earthdata login: https://urs.earthdata.nasa.gov/

**Option C: OpenTopography S3 Bulk Access**
- No authentication required for S3 access
  ```bash
  aws s3 ls s3://raster/NASADEM/ --recursive \
    --endpoint-url https://opentopography.s3.sdsc.edu \
    --no-sign-request
  ```

### Tiles Needed Per Theater

| Theater | Approx. Bounding Box | Tiles (~) | Raw Size |
|---|---|---|---|
| Korean Peninsula | 33-43°N, 124-132°E | ~80 | ~2 GB |
| Persian Gulf / Hormuz | 23-31°N, 47-59°E | ~96 | ~2.4 GB |
| Baltics (ground) | 53-60°N, 19-29°E | ~70 | ~1.8 GB |
| Taiwan | 21-26°N, 119-123°E | ~20 | ~500 MB |
| Israel | 29-34°N, 34-36°E | ~10 | ~250 MB |
| Saudi / UAE | 18-29°N, 36-57°E | ~230 | ~5.8 GB |
| Guam | 13-14°N, 144-145°E | ~2 | ~50 MB |

These are raw download sizes before resampling. Processed game-ready files will be dramatically smaller.

### HGT File Format Reference

The native HGT format is trivial to parse in Rust:

- Each file covers exactly 1° latitude × 1° longitude
- Contains a flat array of signed 16-bit big-endian integers
- 1 arc-second: 3601 × 3601 samples (25,934,402 bytes per file)
- 3 arc-second: 1201 × 1201 samples (2,884,802 bytes per file)
- Values are meters above EGM96 geoid (approximately sea level)
- No header — the filename encodes the georeferencing
- -32768 indicates a void (no data)
- Row order: north to south. Column order: west to east.
- Adjacent tiles share edge rows/columns (hence 3601 not 3600)

```rust
// Pseudocode for loading an HGT tile
fn load_hgt(path: &Path) -> Vec<i16> {
    let bytes = std::fs::read(path).unwrap();
    let sample_count = bytes.len() / 2;
    let grid_size = (sample_count as f64).sqrt() as usize; // 3601 or 1201
    bytes.chunks(2)
        .map(|chunk| i16::from_be_bytes([chunk[0], chunk[1]]))
        .collect()
}
```

---

## Source 2: GEBCO 2025 Grid — Ocean Bathymetry

### What It Is

The General Bathymetric Chart of the Oceans, produced by the Nippon Foundation-GEBCO Seabed 2030 Project. A continuous global terrain model covering both ocean and land at 15 arc-second resolution (~450m). The ocean data is a fusion of direct sonar soundings and satellite-altimetry-derived predicted bathymetry. Land areas use SRTM15+ as a base.

For DETERRENCE, GEBCO provides:
- Ocean floor topography for 3D naval theater rendering
- Accurate coastline geometry derived from the elevation data
- Water depth values for any gameplay mechanics that care about depth (shallow water navigation, submarine launch points)
- A unified land+ocean grid that simplifies rendering in littoral theaters where land and sea meet

### License

Public domain with attribution. Users are free to copy, distribute, adapt, and commercially exploit the data. The only requirements are:
1. Acknowledge the source (a line in your credits screen is sufficient)
2. Don't imply official GEBCO/IHO/IOC endorsement
3. Don't misrepresent the data or its source

Suggested attribution: *"Bathymetry data from GEBCO Compilation Group (2025) GEBCO 2025 Grid (doi:10.5285/1c44ce99-0a0d-5f4f-e063-7086abc0ea0f)"*

### Where to Download

**Option A: GEBCO Download App (Best for specific regions)**
- URL: https://download.gebco.net/
- Select grid version → define bounding box → choose format (GeoTIFF recommended)
- No registration required

**Option B: Full Global Grid Download**
- URL: https://www.gebco.net/data-products/gridded-bathymetry-data
- Available as single netCDF file (~8 GB) or 8 × GeoTIFF tiles (90°×90° each)
- Includes Type Identifier (TID) grid showing data source quality per cell

**Option C: OpenTopography**
- URL: https://portal.opentopography.org/raster?opentopoID=OTSDEM.122023.4326.1
- Same API-based subsetting as NASADEM

### Coverage for Naval Theaters

| Theater | Key Feature | GEBCO Useful For |
|---|---|---|
| Strait of Hormuz | Narrow shipping lane, shallow waters | Seafloor depth, island rendering |
| South China Sea | Contested shoals, deep basin | Reef/shoal positions, deep ocean floor |
| Baltic Sea | Shallow, island-studded | Island terrain, shallow-water rendering |
| Norwegian Sea | Deep fjords, continental shelf | Dramatic depth contrast for 3D view |
| Eastern Mediterranean | Moderate depth, island chains | Island silhouettes, depth variation |
| Arabian Sea | Continental shelf to deep ocean | Open ocean depth gradients |

---

## Source 3: Natural Earth — Vector Overlays

### What It Is

A public domain map dataset maintained by cartographers and GIS volunteers. Provides curated, ready-to-use vector data at three scales. Unlike raw GIS data, Natural Earth features are editorially cleaned and aesthetically considered.

For DETERRENCE, Natural Earth provides:
- **Coastlines** — For PPI radar display overlay and land/water masking
- **Country boundaries** — ROE visualization, theater context
- **Disputed territories** — Relevant for several theaters (Kashmir, South China Sea, etc.)
- **Ocean polygons** — Water body identification
- **Bathymetric contours** — Depth lines for PPI display (0, -200, -1000, -2000m, etc.)
- **Ports** — Defended asset placement, civilian shipping origin points
- **Airports** — Civilian air traffic origin/destination for the identification dilemma
- **Urban areas** — Defended population centers
- **Reefs** — Navigation hazards, radar clutter sources
- **Rivers/lakes** — Ground theater geographic context

### License

Fully public domain. No restrictions of any kind. No attribution required. You can modify, redistribute, and commercially exploit without limitation.

### Where to Download

**GitHub Repository (All data, version-controlled):**
- URL: https://github.com/nvkelso/natural-earth-vector
- Includes shapefiles and GeoJSON at all three scales
- Clone the repo or download specific files

**Direct Downloads by Category:**
- 10m coastline: `https://www.naturalearthdata.com/download/10m/physical/ne_10m_coastline.zip` (~800 KB)
- 10m land polygons: `https://www.naturalearthdata.com/download/10m/physical/ne_10m_land.zip` (~4 MB)
- 10m ocean: `https://www.naturalearthdata.com/download/10m/physical/ne_10m_ocean.zip` (~4 MB)
- 10m bathymetry contours: `https://www.naturalearthdata.com/download/10m/physical/ne_10m_bathymetry_all.zip` (~16 MB)
- 10m countries: `https://www.naturalearthdata.com/download/10m/cultural/ne_10m_admin_0_countries.zip` (~5 MB)
- 10m airports: `https://www.naturalearthdata.com/download/10m/cultural/ne_10m_airports.zip` (~300 KB)
- 10m ports: `https://www.naturalearthdata.com/download/10m/cultural/ne_10m_ports.zip` (~300 KB)

**GeoJSON versions (easier to parse, slightly larger):**
- URL: https://github.com/martynafford/natural-earth-geojson

### Scale Selection Guide

| Scale | Best For | Detail Level |
|---|---|---|
| 1:10m | Zoomed-in PPI display, littoral scenarios | Full coastline detail, minor islands |
| 1:50m | Standard PPI display, theater overview | Major coastline features, significant islands |
| 1:110m | Campaign map, world overview | Continental outlines only |

For DETERRENCE, ship the 10m data for the active theater region (it's small — all vectors for a theater fit in <1 MB) and 50m or 110m for any world overview map.

---

## Processing Pipeline

### Phase 1: Build-Time Preprocessing

This runs on the developer machine, not shipped with the game. Written as a standalone Rust CLI tool or Python script using GDAL.

```
Raw Data (NASADEM + GEBCO + Natural Earth)
    │
    ▼
┌─────────────────────────────┐
│  1. TILE SELECTION          │  Select tiles covering theater bounding box
│     & MERGE                 │  Merge adjacent tiles into single heightmap
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  2. RESAMPLE                │  Downsample from 30m to target resolution
│                             │  90m default (gameplay), 30m optional (high-fidelity)
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  3. VOID FILL               │  Interpolate any remaining -32768 void pixels
│                             │  (NASADEM has very few, but handle edge cases)
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  4. OCEAN MERGE             │  For littoral theaters: merge NASADEM land
│     (Littoral theaters)     │  with GEBCO ocean into unified heightmap
│                             │  Use coastline mask to blend at shoreline
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  5. COMPACT BINARY EXPORT   │  Write game-ready .dtrn binary file
│                             │  (see format spec below)
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  6. TEXTURE GENERATION      │  Procedurally generate terrain color/material
│                             │  maps from elevation + slope + theater biome
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  7. VECTOR EXTRACTION       │  Clip Natural Earth vectors to theater bounds
│                             │  Convert to lightweight binary polyline format
└─────────────────────────────┘
```

### Phase 2: Game-Ready Binary Format (.dtrn)

Custom compact format for fast loading in Rust. No external library dependencies for parsing.

```
DETERRENCE Terrain File (.dtrn)
================================

Header (64 bytes):
  magic:          [u8; 4]    = "DTRN"
  version:        u16        = 1
  flags:          u16        = bitflags (has_ocean, has_texture, etc.)
  origin_lat:     f64        = southwest corner latitude (degrees)
  origin_lon:     f64        = southwest corner longitude (degrees)
  cell_size:      f64        = arc-seconds per cell (e.g., 3.0 for 90m)
  width:          u32        = number of columns
  height:         u32        = number of rows
  min_elevation:  i16        = minimum value in grid (meters)
  max_elevation:  i16        = maximum value in grid (meters)
  reserved:       [u8; 16]   = zero-filled, future use

Elevation Data:
  i16[height × width]        = elevation values in meters, row-major
                                north-to-south, west-to-east
                                big-endian (matching HGT convention)

Optional Sections (indicated by flags):
  Texture Index:  u8[height × width]   = biome/material ID per cell
  Ocean Mask:     bitfield              = 1 bit per cell, land=1 ocean=0
```

### Phase 3: Runtime Loading in Rust

```rust
// In deterrence-terrain crate
pub struct TerrainGrid {
    origin: (f64, f64),      // (lat, lon) of SW corner
    cell_size: f64,          // arc-seconds per cell
    width: u32,
    height: u32,
    elevations: Vec<i16>,    // flat array, row-major
    ocean_mask: Option<BitVec>,
}

impl TerrainGrid {
    /// O(1) elevation query at a lat/lon coordinate
    pub fn elevation_at(&self, lat: f64, lon: f64) -> Option<f32> {
        let col = ((lon - self.origin.1) * 3600.0 / self.cell_size) as usize;
        let row = ((self.origin.0 + (self.height as f64 * self.cell_size / 3600.0) - lat)
                   * 3600.0 / self.cell_size) as usize;
        if col < self.width as usize && row < self.height as usize {
            Some(self.elevations[row * self.width as usize + col] as f32)
        } else {
            None
        }
    }

    /// Bresenham-style LOS check between two points
    pub fn has_line_of_sight(&self, from: (f64, f64, f32), to: (f64, f64, f32)) -> bool {
        // Step along ray, check elevation clearance at each cell
        // Account for Earth curvature over long distances
        // ...
    }
}
```

### Phase 4: Frontend Terrain Mesh (Three.js)

The GameStateSnapshot includes terrain metadata. The frontend loads the heightmap once per mission and builds a Three.js mesh:

```typescript
// Simplified terrain mesh generation
function buildTerrainMesh(heightmap: Int16Array, width: number, height: number): THREE.Mesh {
    const geometry = new THREE.PlaneGeometry(width, height, width - 1, height - 1);
    const positions = geometry.attributes.position;

    for (let i = 0; i < positions.count; i++) {
        positions.setZ(i, heightmap[i] * verticalScale);
    }

    geometry.computeVertexNormals();

    const material = new THREE.MeshStandardMaterial({
        vertexColors: true,  // Colors assigned per-vertex from texture index
        flatShading: false,
        side: THREE.FrontSide,
    });

    return new THREE.Mesh(geometry, material);
}
```

---

## Terrain Texturing Strategy

Since no openly licensed satellite imagery exists at game-quality resolution, terrain textures are **procedurally generated** from the elevation data itself. This is the standard approach in games using real heightmaps and has the advantage of being resolution-independent.

### Material Classification

Each heightmap cell is assigned a material/biome ID based on computed terrain attributes:

| Attribute | Derived From | Method |
|---|---|---|
| **Elevation** | Direct from heightmap | Value lookup |
| **Slope angle** | Height difference between adjacent cells | `atan2(dz, cell_size)` |
| **Slope aspect** | Direction of steepest descent | Gradient direction |
| **Curvature** | Rate of slope change | Second derivative |
| **Relative elevation** | Height vs. local neighborhood mean | Gaussian blur comparison |

### Theater Biome Profiles

Each theater YAML defines a biome profile that maps terrain attributes to materials:

```yaml
# theaters/persian_gulf.yaml
biome:
  name: "arid_desert"
  materials:
    - id: 0
      name: "sand_flat"
      color: [0.85, 0.78, 0.62]          # Base desert sand
      conditions:
        slope_max: 10                      # Degrees
        elevation_max: 200                 # Meters
    - id: 1
      name: "sand_dune"
      color: [0.90, 0.82, 0.65]
      conditions:
        slope_range: [5, 25]
        elevation_max: 300
        curvature: "convex"
    - id: 2
      name: "rock_exposed"
      color: [0.55, 0.48, 0.40]
      conditions:
        slope_min: 25                      # Steep = exposed rock
    - id: 3
      name: "gravel_wadi"
      color: [0.65, 0.58, 0.50]
      conditions:
        curvature: "concave"               # Valley/channel bottoms
        elevation_max: 500
    - id: 4
      name: "mountain_rock"
      color: [0.50, 0.45, 0.38]
      conditions:
        elevation_min: 800
    - id: 5
      name: "salt_flat"
      color: [0.92, 0.90, 0.85]
      conditions:
        slope_max: 2
        relative_elevation: "local_minimum"
```

```yaml
# theaters/korean_peninsula.yaml
biome:
  name: "temperate_mountainous"
  materials:
    - id: 0
      name: "lowland_green"
      color: [0.35, 0.50, 0.28]
      conditions:
        elevation_max: 200
        slope_max: 15
    - id: 1
      name: "forest"
      color: [0.22, 0.38, 0.18]
      conditions:
        elevation_range: [100, 1200]
        slope_range: [5, 35]
    - id: 2
      name: "alpine_rock"
      color: [0.55, 0.52, 0.48]
      conditions:
        elevation_min: 1200
    - id: 3
      name: "ridge_rock"
      color: [0.50, 0.47, 0.42]
      conditions:
        slope_min: 35
    - id: 4
      name: "valley_agricultural"
      color: [0.45, 0.55, 0.30]
      conditions:
        slope_max: 5
        curvature: "concave"
        elevation_max: 300
    - id: 5
      name: "urban"
      color: [0.55, 0.55, 0.52]
      conditions:
        slope_max: 8
        relative_elevation: "local_flat"
        elevation_max: 100
```

### Rendering Approach

The 3D terrain view uses **vertex coloring with detail textures**, not high-resolution texture maps. This keeps memory usage low and scales to any terrain size.

1. **Vertex color**: Each vertex gets a base color from its material ID (assigned at build time)
2. **Detail textures**: A small set of tileable textures (sand grain, rock surface, grass blades) are blended per-material using a shader, adding visual detail at close zoom
3. **Slope shading**: Normals from the heightmap provide natural lighting and shadow
4. **Elevation tinting**: Subtle color shift with altitude (atmospheric haze effect)
5. **Grid overlay** (optional): Faint coordinate grid for tactical reference

This approach matches the game's semi-stylized aesthetic — recognizably real terrain without trying to be photorealistic. The CIC radar console is the primary view; the 3D terrain is context and payoff, not the main experience.

### Detail Texture Set (Per Biome)

A small number of tileable textures per biome, stored as compressed PNG:

| Texture | Size | Purpose |
|---|---|---|
| `sand_detail.png` | 256×256 | Sand grain pattern for desert biomes |
| `rock_detail.png` | 256×256 | Rock surface for mountains/cliffs |
| `grass_detail.png` | 256×256 | Vegetation pattern for temperate biomes |
| `snow_detail.png` | 256×256 | Snow/ice for high altitude |
| `urban_detail.png` | 256×256 | Simplified building pattern |
| `water_detail.png` | 256×256 | Shallow water caustics |

Total texture memory per biome: ~1-2 MB. These can be shared across theaters with similar climates.

---

## Estimated File Sizes (Shipped with Game)

| Component | Per Theater | All 18 Theaters |
|---|---|---|
| Heightmap (.dtrn, 90m resolution) | 1-4 MB | ~30-50 MB |
| Texture index | 0.2-1 MB | ~5-10 MB |
| Coastline vectors | <100 KB | ~500 KB |
| Detail textures (shared) | — | ~5 MB |
| **Total terrain data** | **~2-5 MB each** | **~40-65 MB** |

This fits comfortably within the <50 MB base install target if you ship 4-6 theaters initially and offer the rest as downloadable packs, or slightly exceeds it if you bundle everything.

---

## Build Tool Requirements

The preprocessing pipeline needs these tools (developer machine only, not shipped):

| Tool | Purpose | Install |
|---|---|---|
| **GDAL** (3.x) | Raster merge, resample, format conversion, vector clipping | `apt install gdal-bin` / `brew install gdal` |
| **Python 3** + rasterio | Scripting the pipeline, NASADEM tile selection | `pip install rasterio numpy` |
| **ogr2ogr** (part of GDAL) | Natural Earth shapefile → GeoJSON conversion and clipping | Included with GDAL |

Alternatively, the entire pipeline can be written in Rust using the `gdal` crate bindings, keeping the toolchain unified. The tradeoff is more upfront work versus a simpler developer setup.

---

## Attribution & Legal Summary

| Source | License | Attribution Required | Commercial Use | Redistribution |
|---|---|---|---|---|
| NASADEM (NASA JPL) | Public domain | No (recommended) | Yes, unrestricted | Yes, unrestricted |
| GEBCO 2025 Grid | Public domain (terms) | Yes | Yes, including commercial | Yes |
| Natural Earth | Public domain | No | Yes, unrestricted | Yes, unrestricted |

**Recommended credits screen text:**

> Elevation data: NASADEM, NASA JPL (2020). NASADEM Merged DEM Global 1 arc second V001.
>
> Bathymetry data: GEBCO Compilation Group (2025). GEBCO 2025 Grid.
>
> Vector data: Made with Natural Earth. Free vector and raster map data at naturalearthdata.com.

No legal review needed. No license fees. No permission requests. No usage reporting. All three datasets are explicitly cleared for commercial game distribution.
