#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

paths=(
  "node_modules"
  "src-tauri/target"
  ".svelte-kit"
  "build"
  "dist"
  "node_modules/.vite"
  "node_modules/.cache"
  ".pnpm-store"
  ".npm"
  "src-tauri/gen"
)

echo "Disk usage snapshot for reproducible dependencies/caches/build outputs:"
for path in "${paths[@]}"; do
  if [ -e "$path" ]; then
    du -sh "$path"
  else
    echo "0B	$path (missing)"
  fi
done
