# Echolocate

A desktop network discovery and topology visualizer. See every device on your network — who's connected, what they're running, and how they're behaving.

Built with **Tauri 2** (Rust backend) + **SvelteKit** (Svelte 5 frontend) + **SQLite**.

> **Status: Alpha** — Core functionality is implemented and compiles cleanly. The app discovers devices, scans ports, fingerprints operating systems, and renders an interactive topology graph. Not yet tested in production environments.

## What It Does

- **Passive network discovery** — Reads the ARP table to find every device on your LAN without sending a single packet
- **Active scanning** — Ping sweep for latency, TCP connect port scan (top 100 ports), banner grabbing
- **OS fingerprinting** — Identifies iOS, macOS, Windows, Linux, and Android from port signatures and vendor OUI patterns
- **Device classification** — Auto-categorizes devices as router, computer, phone, printer, IoT, or media device
- **Interactive topology graph** — Force-directed SVG graph (d3-force) with zoom, pan, drag, color-coded nodes by OS/type
- **Alert engine** — Detects new devices, untrusted devices, and device departures with desktop notifications
- **Continuous monitoring** — Background scan loop at configurable intervals with real-time UI updates
- **Vendor lookup** — 38K+ IEEE OUI entries for MAC-to-manufacturer resolution
- **Hostname resolution** — Reverse DNS lookups for discovered IPs
- **Export/Import** — JSON export of all device and alert data

## Screenshots

*Coming soon — the app compiles and builds but needs real network testing.*

## Architecture

```
src-tauri/          Rust backend (Tauri 2)
  src/
    scanner/        ARP parsing, ping sweep, port scan, OS fingerprint
    alerts/         Alert evaluation engine + desktop notifications
    db/             SQLite with r2d2 pool, WAL mode, migration system
    network/        Interface discovery, OUI database, hostname resolver
    commands/       Tauri IPC command handlers
    state.rs        Shared app state (DB pool, OUI db, cancellation tokens)

src/                SvelteKit frontend (Svelte 5 runes)
  lib/
    components/     Topology graph, device list, alerts, scan controls
    stores/         Svelte writable stores (devices, scan, alerts, settings)
    services/       tauri-bridge.ts (invoke), tauri-events.ts (listen)
    types/          TypeScript interfaces matching Rust serde structs
  routes/           SPA pages: topology, devices, alerts, settings
```

**Key design decisions:**
- Single `tauri-bridge.ts` for all `invoke()` calls, single `tauri-events.ts` for all `listen()` calls
- d3 owns SVG rendering, Svelte manages state — no fighting between frameworks
- Cooperative scan cancellation via `tokio_util::CancellationToken`
- All device state derived from SQLite — the DB is the source of truth

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri 2 |
| Frontend | SvelteKit + Svelte 5 (runes) |
| Styling | Tailwind CSS v4 |
| Graph | d3-force (SVG) |
| Backend | Rust (tokio async runtime) |
| Database | SQLite (rusqlite + r2d2 pool, WAL mode) |
| Notifications | tauri-plugin-notification |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+
- macOS (network scanning currently uses macOS-specific commands: `arp`, `ifconfig`, `netstat`, `ping`, `host`)

### Development

```bash
# Install frontend dependencies
npm install

# Run in development mode (starts both Vite dev server and Tauri)
npm run tauri dev

# Run Rust tests (48 tests)
cd src-tauri && cargo test

# Build for production
npm run tauri build
```

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+R` | Quick scan |
| `Escape` | Deselect device |

## Test Coverage

48 Rust tests across all modules:

- **Database**: Migrations, CRUD operations, FK constraints, settings roundtrip
- **Scanner**: ARP output parsing, ping response parsing, port service mapping
- **Fingerprint**: OS detection (iOS/macOS/Windows/Linux/Android), device classification
- **Alerts**: New device, untrusted device, departed device, trusted device exclusion

## Known Limitations

- **macOS only** — Network commands (`arp -a`, `ifconfig`, `netstat -rn`) are macOS-specific. Linux/Windows support would require platform-specific implementations.
- **No privileged scanning** — Uses passive ARP table reading, not raw socket ARP requests. Requires devices to have communicated recently.
- **Top 1000 ports** — Currently maps to Top 100 (placeholder).
- **No IPv6** — Discovery and scanning is IPv4 only.

## License

MIT
