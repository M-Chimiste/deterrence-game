# Project Status

## Implemented (2026-02-07)
- Replaced PixiJS UI (MainMenu/HUD/Strategic actions) with React overlay mounted on `#ui-root`
- Added Zustand `GameStore` for UI state synced from Tauri event streams
- Introduced UI design system (`theme.css`) and CSS modules for menu, HUD, and strategic panels
- Implemented UI click sound and UI screen shake mirroring impacts
- Kept UI/scene crisp by avoiding CRT on gameplay UI; added subtle CRT post-process for **MainMenu background only**
- Simplified `StrategicView` to focus on map rendering only
- Upgraded MainMenu background visuals (drifting grid, starfield, scan band, signal blips, higher activity)
- Added `src/vite-env.d.ts` so TypeScript understands CSS module imports (`*.module.css`)
- Added Windows build scripts (`scripts/dev.ps1`, `scripts/build.ps1`) plus `.cmd` wrappers and `npm` shortcuts (`win:dev`, `win:build`)

## Next Steps
- Add region hover highlighting on the map for better feedback
- Refine HUD data presentation (contacts, weather) with icons and compact layouts

## Debug Log
- 2026-02-07: Migrated UI layer to React, removed PixiJS HUD/MainMenu, added new UI store and CSS overlay utilities. Verified GameRenderer now updates UI state and audio toggles are wired through game actions.
- 2026-02-07: Implemented battery/city index mapping using campaign-owned region ordering and updated HUD/highlight wiring to match backend battery index semantics.
- 2026-02-07: Removed CRT filter/shader and UI scanline overlay, and cleaned up CRT toggles/hints.
- 2026-02-07: Switched Pixi canvas to HiDPI rendering with renderer resize + stage scaling for sharper UI and map lines.
- 2026-02-07: Increased interceptor blast radius by 25% across all types (via shared multiplier).
- 2026-02-07: Increased standard interceptor thrust for higher base speed.
- 2026-02-07: Improved title screen ambience by layering animated Pixi menu background (stars, drift, scan band, blips) and applying `CRTFilter` only while in `MainMenu`. Added `vite-env.d.ts` to fix TS errors for CSS modules.
- 2026-02-07: Added Windows-friendly dev/build scripts in `scripts/` (PowerShell + `.cmd` wrappers) and wired `npm run win:dev` / `npm run win:build`.
