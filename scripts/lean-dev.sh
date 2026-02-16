#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [ ! -d "node_modules" ]; then
  echo "Missing node_modules. Run npm install first."
  exit 1
fi

TMP_BASE_DIR="${LEAN_DEV_BASEDIR:-/tmp}"
LEAN_TMP_ROOT="$(mktemp -d "${TMP_BASE_DIR}/echolocate-lean-dev.XXXXXX")"
VITE_CACHE_DIR="${LEAN_TMP_ROOT}/vite-cache"
CARGO_TARGET_DIR="${LEAN_TMP_ROOT}/cargo-target"
mkdir -p "$VITE_CACHE_DIR" "$CARGO_TARGET_DIR"

cleanup() {
  local exit_code="$1"
  if [ -d "$LEAN_TMP_ROOT" ]; then
    rm -rf "$LEAN_TMP_ROOT"
    echo "Lean dev cleanup complete: removed ephemeral caches at $LEAN_TMP_ROOT"
  fi
  exit "$exit_code"
}

on_exit() {
  local exit_code=$?
  cleanup "$exit_code"
}

trap on_exit EXIT INT TERM

echo "Starting lean dev mode"
echo "Temporary Vite cache: $VITE_CACHE_DIR"
echo "Temporary Cargo target: $CARGO_TARGET_DIR"

config_override=$(cat <<EOF
{"build":{"beforeDevCommand":"LEAN_VITE_CACHE_DIR=${VITE_CACHE_DIR} npm run dev -- --host 127.0.0.1 --port 1420 --strictPort"}}
EOF
)

CARGO_TARGET_DIR="$CARGO_TARGET_DIR" npm run tauri -- dev -c "$config_override" "$@"
