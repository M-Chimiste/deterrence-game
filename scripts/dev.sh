#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "=== Deterrence — Dev Build ==="

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

# 3. TypeScript check
echo ">>> tsc --noEmit"
npx tsc --noEmit

# 4. Launch dev app (Tauri handles frontend dev server + Rust build)
echo ">>> cargo tauri dev"
cargo tauri dev
