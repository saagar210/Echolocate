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

## Known Limitations

- **macOS only** — Network commands are currently platform-specific
- **No IPv6** — Discovery and scanning are currently IPv4 only
- **Top 100 ports only** (not yet parameterized) — Placeholder for port range selection
- **No custom alert rules** — Rules are hardcoded, UI allows enable/disable only

## License

MIT
