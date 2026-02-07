#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "=== Deterrence — Production Build ==="

# 1. Install frontend deps if needed
if [ ! -d node_modules ]; then
  echo ">>> npm install"
  npm install
fi

# 2. Rust checks
echo ">>> cargo clippy"
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

echo ">>> cargo test"
cargo test --manifest-path src-tauri/Cargo.toml

# 3. Frontend build (tsc + vite)
echo ">>> npm run build"
npm run build

# 4. Tauri production build (compiles Rust release + bundles .app + .dmg)
echo ">>> cargo tauri build"
cargo tauri build

echo ""
echo "=== Build complete ==="
echo "Artifacts:"
ls -lh src-tauri/target/release/bundle/macos/*.app 2>/dev/null || true
ls -lh src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null || true
