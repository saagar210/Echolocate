# Echolocate

A desktop network discovery and topology visualizer.

Stack: Tauri 2 (Rust backend), SvelteKit (Svelte 5 frontend), SQLite.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+
- Linux (for local Rust/Tauri test builds): `glib-2.0` development package installed (provides `glib-2.0.pc`)
- macOS (network scanning currently uses macOS-specific commands: `arp`, `ifconfig`, `netstat`, `ping`, `host`)

### Development

```bash
# Install frontend dependencies
npm install

# Run in development mode (starts both Vite dev server and Tauri)
npm run tauri dev

# Run frontend checks
npm run check

# Build for production
npm run tauri build
```

### Lean Dev Mode (Low Disk)

Use lean mode for day-to-day development when you want to keep disk usage under control.

```bash
# Show current cache/build/output sizes
npm run size:report

# Start dev with ephemeral build caches
npm run lean:dev

# Remove heavy build artifacts only (keeps dependencies for speed)
npm run clean:heavy

# Remove all reproducible local caches/outputs (includes node_modules)
npm run clean:local
```

Normal dev (`npm run tauri dev`) keeps Rust build artifacts in `src-tauri/target`, which grows quickly but gives faster restarts after the first compile.

Lean dev (`npm run lean:dev`) moves Rust build output and Vite cache to a temporary directory and deletes it automatically when the app exits. This saves disk space but increases startup time because Rust recompiles from scratch each run.

## Known Limitations

- **macOS only** — Network commands are currently platform-specific
- **No IPv6** — Discovery and scanning are currently IPv4 only
- **Top 100 ports only** (not yet parameterized) — Placeholder for port range selection
- **No custom alert rules** — Rules are hardcoded, UI allows enable/disable only

## Updating OUI Vendor Data

The bundled OUI database is stored in a compact format at:
`src-tauri/resources/oui.csv` (columns: `assignment,org`).

To regenerate it from the latest IEEE source:

```bash
python3 scripts/compact_oui_csv.py
```

To regenerate from a local IEEE CSV export instead:

```bash
python3 scripts/compact_oui_csv.py --input /path/to/oui.csv
```

## License

MIT
