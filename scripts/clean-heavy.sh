#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

paths=(
  "src-tauri/target"
  ".svelte-kit"
  "build"
  "dist"
  "node_modules/.vite"
  "node_modules/.cache"
)

removed_any=0
for path in "${paths[@]}"; do
  if [ -e "$path" ]; then
    rm -rf "$path"
    echo "Removed $path"
    removed_any=1
  fi
done

if [ "$removed_any" -eq 0 ]; then
  echo "No heavy build artifacts found."
fi
