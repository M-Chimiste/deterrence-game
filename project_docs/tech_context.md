# DETERRENCE — Tech Context

## Development Environment Setup

### Prerequisites

| Tool | Version | Purpose |
|---|---|---|
| Rust (via rustup) | stable, 2021 edition | Backend compilation |
| Node.js | 20.x LTS | Frontend build tooling, Vite, npm |
| Tauri CLI | 2.x | Application build, dev server, bundling |
| Git | 2.40+ | Version control |
| VS Code (recommended) | latest | IDE with rust-analyzer + TypeScript extensions |

### System Dependencies (Platform-Specific)

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**macOS:**
```bash
xcode-select --install
```

**Windows:**
- Visual Studio Build Tools 2022 with C++ workload
- WebView2 (ships with Windows 10/11)

### Initial Setup

```bash
# Clone repository
git clone <repo-url> deterrence
cd deterrence

# Install Rust toolchain
rustup default stable
rustup component add clippy rustfmt

# Install Node dependencies
cd frontend && npm install && cd ..

# Install Tauri CLI
cargo install tauri-cli --version "^2"

# Verify setup
cargo tauri dev
```

### Development Workflow

```bash
# Full dev mode (Rust + frontend with HMR)
cargo tauri dev

# Rust only (headless simulation testing)
cargo test --workspace

# Frontend only (against mock state)
cd frontend && npm run dev

# Build release
cargo tauri build

# Lint
cargo clippy --workspace -- -D warnings
cd frontend && npm run lint

# Format
cargo fmt --all
cd frontend && npm run format
```

---

## Coding Standards

### Rust

**Style:**
- Follow `rustfmt` defaults (run `cargo fmt --all` before commit)
- `clippy` with `-D warnings` — all warnings are errors in CI
- 2021 edition idioms

**Naming:**
- Types: `PascalCase` — `TrackInfo`, `EngagementPhase`, `RadarSystem`
- Functions/methods: `snake_case` — `calculate_detection_probability`, `advance_tick`
- Constants: `SCREAMING_SNAKE_CASE` — `MAX_TRACKS`, `DEFAULT_TICK_RATE`
- Modules: `snake_case` — `fire_control`, `threat_ai`
- Crate names: `kebab-case` — `deterrence-core`, `deterrence-sim`

**Error Handling:**
- Use `thiserror` for library error types in each crate
- Use `anyhow` only in the `deterrence-app` binary crate for top-level error handling
- Never `unwrap()` in simulation code — use `?` or explicit error handling
- `unwrap()` is acceptable ONLY in tests and initialization code with clear invariant comments

**Documentation:**
- All public types and functions have `///` doc comments
- Module-level `//!` documentation for each module explaining its role in the system
- Complex algorithms (radar detection, Pk calculation, intercept geometry) get block comments explaining the math and referencing source material

**Testing:**
- Unit tests in `#[cfg(test)]` modules within each file
- Integration tests in `tests/` directory at workspace root
- Simulation tests should be deterministic — seed all RNG
- Test radar detection at known ranges/RCS values against expected Pd
- Test fire control state machine transitions exhaustively
- Test engagement Pk at boundary conditions (min/max range, edge of envelope)

**Performance:**
- Profile before optimizing — use `cargo flamegraph` or `perf`
- `rayon` for embarrassingly parallel iteration (radar sweeps over entities)
- Avoid allocations in the hot loop (simulation tick). Pre-allocate vectors, reuse buffers.
- Serialization budget: GameStateSnapshot must serialize in < 3ms at 30Hz

**ECS Conventions:**
- Components are plain structs with `pub` fields — no methods on components
- Systems are standalone functions that take `&World` or `&mut World`
- Queries should be as narrow as possible — don't fetch components you don't need
- Entity creation/destruction happens in a dedicated phase, not mid-system

### TypeScript

**Style:**
- Strict mode enabled (`"strict": true` in tsconfig)
- ESLint with recommended TypeScript rules
- Prettier for formatting (single quotes, no semicolons, 2-space indent)

**Naming:**
- Types/Interfaces: `PascalCase` — `GameStateSnapshot`, `TrackView`
- Functions/variables: `camelCase` — `updateTacticalDisplay`, `currentSnapshot`
- Constants: `SCREAMING_SNAKE_CASE` — `TICK_RATE`, `MAX_HISTORY_DOTS`
- Files: `PascalCase` for components (`VLSStatus.tsx`), `camelCase` for modules (`gameState.ts`)

**Type Safety:**
- No `any` types — ever. Use `unknown` and narrow with type guards.
- All IPC payloads have corresponding TypeScript types that mirror Rust structs
- Types for IPC are generated or manually maintained in `src/ipc/` — these are the source of truth for the frontend

**Component Conventions (Preact/Solid):**
- Functional components only, no classes
- Props interfaces explicitly defined
- State derived from Zustand store subscriptions, not local state (game state comes from Rust)
- Local UI state (panel position, scroll position, hover state) uses framework-native state

**Three.js Conventions:**
- Scene management in dedicated manager classes, not scattered across components
- Dispose all geometries, materials, and textures on cleanup
- Use `InstancedMesh` for repeated geometry (track symbols, history dots, missile trails)
- Shaders in separate `.glsl` files, imported via Vite raw import
- Performance budget: 60fps on integrated GPU with full PPI and 3D PIP. Profile with Chrome DevTools Performance panel.

### Shared (Rust ↔ TypeScript)

**IPC Contract Maintenance:**
- The Rust `PlayerCommand` enum and `GameStateSnapshot` struct are the canonical definitions
- TypeScript mirrors in `src/ipc/commands.ts` and `src/ipc/state.ts` must be kept in sync manually (or via codegen if tooling is added later)
- Any change to the IPC contract requires updating both sides AND updating the integration test in `tests/sim/ipc.rs`
- Version the IPC contract — include a `protocol_version: u32` field in the handshake so mismatches are caught early

---

## Dependencies

### Rust Crate Dependencies

**deterrence-core** (zero external dependencies beyond std):
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
```

**deterrence-sim:**
```toml
[dependencies]
deterrence-core = { path = "../deterrence-core" }
nalgebra = "0.33"        # or glam = "0.29" (benchmark both during Phase 1)
rand = "0.8"
rand_distr = "0.4"
noise = "0.9"
rayon = "1.10"
hecs = "0.10"
```

**deterrence-threat-ai:**
```toml
[dependencies]
deterrence-core = { path = "../deterrence-core" }
nalgebra = "0.33"
rand = "0.8"
```

**deterrence-terrain:**
```toml
[dependencies]
deterrence-core = { path = "../deterrence-core" }
nalgebra = "0.33"
byteorder = "1"          # Heightmap binary parsing
```

**deterrence-procgen:**
```toml
[dependencies]
deterrence-core = { path = "../deterrence-core" }
deterrence-terrain = { path = "../deterrence-terrain" }
rand = "0.8"
serde_yaml = "0.9"       # Theater/scenario data loading
```

**deterrence-campaign:**
```toml
[dependencies]
deterrence-core = { path = "../deterrence-core" }
redb = "2"               # Embedded database for save games
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**deterrence-app (Tauri binary):**
```toml
[dependencies]
deterrence-core = { path = "../deterrence-core" }
deterrence-sim = { path = "../deterrence-sim" }
deterrence-threat-ai = { path = "../deterrence-threat-ai" }
deterrence-terrain = { path = "../deterrence-terrain" }
deterrence-procgen = { path = "../deterrence-procgen" }
deterrence-campaign = { path = "../deterrence-campaign" }
tauri = { version = "2", features = ["all"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

### Frontend Dependencies

```json
{
  "dependencies": {
    "three": "^0.169",
    "@tauri-apps/api": "^2",
    "zustand": "^5",
    "preact": "^10",
    "howler": "^2.2",
    "@types/three": "^0.169"
  },
  "devDependencies": {
    "typescript": "^5.6",
    "vite": "^6",
    "@tauri-apps/cli": "^2",
    "eslint": "^9",
    "@typescript-eslint/parser": "^8",
    "@typescript-eslint/eslint-plugin": "^8",
    "prettier": "^3",
    "vitest": "^2",
    "tweakpane": "^4"
  }
}
```

### Dependency Policy

- **Pin major versions** in Cargo.toml and package.json. Use `cargo update` and `npm update` deliberately, not automatically.
- **Minimize dependency count.** Every dependency is an attack surface and maintenance burden. Prefer stdlib or hand-written solutions for simple tasks.
- **No runtime dependencies on Tauri plugins** unless absolutely necessary. Use standard web APIs (fetch, WebGL, Web Audio) where possible — Tauri plugins add platform-specific complexity.
- **Audit before adding:** Every new dependency must justify its inclusion. "It saves 50 lines of code" is usually not sufficient. "It solves a fundamentally hard problem correctly" is.
- **No C/C++ build dependencies** beyond what Tauri itself requires. Rust-native crates only. This keeps the build toolchain simple across platforms.

---

## Technical Constraints

### Performance Constraints

| Constraint | Limit | Rationale |
|---|---|---|
| Simulation tick budget | 33ms per tick (30Hz) | Fixed-rate simulation. If a tick takes longer, the sim falls behind real-time. |
| IPC serialization budget | 3ms per snapshot | Part of the tick budget. Snapshot must serialize and deserialize within this window. |
| Frontend frame budget | 16ms per frame (60fps) | Standard display refresh. 2D tactical display is the priority; 3D view can drop frames. |
| Entity count | 500+ simultaneous | Saturation attack scenarios with many threats, missiles, decoys, civilian traffic. |
| Memory budget | 500 MB total | Complex joint scenario with terrain, entity state, audio buffers, 3D scene graph. |
| Snapshot size | < 100KB per tick | 30 snapshots/sec × 100KB = 3MB/sec throughput across IPC. Manageable but worth monitoring. |

### Platform Constraints

- **Tauri WebView:** Rendering is inside a platform WebView (WebKit on macOS/Linux, WebView2/Chromium on Windows). WebGL 2.0 is supported on all platforms. WebGPU is NOT guaranteed — do not depend on it.
- **No Node.js in runtime:** Tauri frontend is a web context, not Node. No `fs`, no `path`, no Node APIs. All system access goes through Tauri `invoke()`.
- **Single window:** The application runs in a single Tauri window. Panel management is done within the web context (CSS/JS-based window manager), not native OS windows.
- **Offline-capable:** The game must function fully offline. No runtime network dependencies. Terrain data, audio, and all assets are bundled or downloadable.

### Simulation Constraints

- **Deterministic with seed:** Given the same RNG seed and input sequence, the simulation must produce identical output. Required for replay and testing. No `HashMap` iteration order dependencies, no system clock reads in simulation logic.
- **No floating-point nondeterminism:** Use `f64` for all simulation math. Avoid platform-dependent FP optimizations (`-ffast-math` equivalent). If cross-platform determinism is required (multiplayer), consider fixed-point math for critical paths.
- **Tick-aligned input:** Player commands are processed at tick boundaries, not between ticks. Commands arriving mid-tick are queued for the next tick.
- **Simulation time, not wall time:** The simulation has its own clock that advances per tick. Time compression/pause affects sim time, not tick rate.

---

## Build & Distribution Configuration

### Tauri Configuration (tauri.conf.json)

```jsonc
{
  "build": {
    "beforeBuildCommand": "cd frontend && npm run build",
    "beforeDevCommand": "cd frontend && npm run dev",
    "frontendDist": "../frontend/dist",
    "devUrl": "http://localhost:5173"
  },
  "app": {
    "windows": [
      {
        "title": "DETERRENCE",
        "width": 1920,
        "height": 1080,
        "minWidth": 1280,
        "minHeight": 720,
        "fullscreen": false,
        "resizable": true,
        "decorations": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; media-src 'self' data: blob:"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["nsis", "dmg", "appimage"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "resources": [
      "data/**/*",
      "assets/**/*"
    ]
  }
}
```

### Vite Configuration

```typescript
// frontend/vite.config.ts
import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'

export default defineConfig({
  plugins: [preact()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: ['es2021', 'chrome100', 'safari15'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  assetsInclude: ['**/*.glsl'],  // Import GLSL shaders as raw strings
})
```

### Cargo Workspace Configuration

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
  "crates/deterrence-core",
  "crates/deterrence-sim",
  "crates/deterrence-threat-ai",
  "crates/deterrence-terrain",
  "crates/deterrence-procgen",
  "crates/deterrence-campaign",
  "crates/deterrence-app",
]
resolver = "2"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
nalgebra = "0.33"
rand = "0.8"

[profile.dev]
opt-level = 1            # Simulation needs some optimization even in dev

[profile.dev.package."*"]
opt-level = 2            # Optimize dependencies fully in dev

[profile.release]
lto = "thin"             # Link-time optimization for smaller binary
strip = true             # Strip debug symbols
codegen-units = 1        # Single codegen unit for better optimization
```

---

## Third-Party Integrations

### Current: None

DETERRENCE has zero runtime third-party service dependencies. The game is fully offline-capable.

### Potential Future

| Integration | Purpose | Phase |
|---|---|---|
| Steam API (Steamworks) | Achievement tracking, cloud saves, workshop (mods) | Post-Phase 4 |
| Tauri auto-updater | Application updates for direct distribution | Phase 4 |
| Sentry / crash reporting | Error tracking for released builds | Phase 4 |
| Analytics (opt-in) | Gameplay telemetry for balancing (which threats cause most reloads, etc.) | Post-launch |

### Terrain Data Source

Terrain heightmaps are derived from SRTM (Shuttle Radar Topography Mission) data — public domain, no license restrictions. Data is pre-processed and bundled, not downloaded at runtime.

Processing pipeline (build-time, not runtime):
1. Download SRTM tiles for theater regions
2. Resample to game-appropriate resolution (e.g., 90m grid for gameplay, 30m for high-fidelity)
3. Convert to compact binary format (16-bit height values, custom header)
4. Bundle in `assets/terrain/` per theater

---

## Environment Variables & Configuration

### Development

| Variable | Purpose | Default |
|---|---|---|
| `TAURI_DEBUG` | Enable debug mode (sourcemaps, verbose logging) | Set by `cargo tauri dev` |
| `DETERRENCE_LOG` | Rust log level (trace, debug, info, warn, error) | `info` |
| `DETERRENCE_TICK_RATE` | Simulation tick rate override | `30` |
| `DETERRENCE_SEED` | RNG seed override for deterministic testing | Random |
| `DETERRENCE_DATA_DIR` | Override data directory path | `./data` |

### User Configuration (Runtime)

Stored in platform-appropriate config directory (`~/.config/deterrence/` on Linux, `~/Library/Application Support/deterrence/` on macOS, `%APPDATA%/deterrence/` on Windows):

```yaml
# settings.yaml
display:
  fullscreen: false
  resolution: [1920, 1080]
  vsync: true
  render_quality: high       # low, medium, high
  crt_effects: true          # Phosphor glow, scanlines
  symbology: ntds            # ntds or milstd2525

audio:
  master_volume: 0.8
  ambient_volume: 0.6
  alert_volume: 1.0
  voice_volume: 0.9

gameplay:
  default_doctrine: auto_special
  time_scale: 1.0
  auto_pause_on_alert: false
  colorblind_mode: none      # none, deuteranopia, protanopia, tritanopia

controls:
  hotkeys: default           # or path to custom hotkey YAML
  mouse_sensitivity: 1.0

debug:
  show_fps: false
  show_tick_time: false
  tweakpane: false           # Enable runtime debug panel
```

---

## Git Conventions

### Branch Strategy

- `main` — stable, release-ready code. All merges via PR.
- `dev` — integration branch. Features merge here first.
- `feature/<name>` — feature branches. Short-lived, descriptive names.
- `fix/<name>` — bug fix branches.
- `release/<version>` — release preparation branches.

### Commit Messages

Follow Conventional Commits:

```
feat(sim): implement radar energy budget tradeoff
fix(tactical): correct velocity leader direction for southbound tracks
refactor(fire-control): extract illuminator scheduling into dedicated module
docs(system-patterns): add ECS component diagram
test(radar): add Pd boundary condition tests for SPY-1 model
perf(ipc): switch GameState serialization to MessagePack
chore(deps): update Three.js to r170
```

### PR Requirements

- All CI checks pass (clippy, rustfmt, eslint, tests)
- At least one approval (when working with collaborators)
- Descriptive PR body linking to relevant design doc sections
- Screenshots/recordings for UI changes
- Performance impact noted for simulation changes
