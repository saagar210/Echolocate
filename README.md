# Echolocate

A desktop network discovery and topology visualizer. See every device on your network — who's connected, what they're running, and how they're behaving.

Built with **Tauri 2** (Rust backend) + **SvelteKit** (Svelte 5 frontend) + **SQLite**.

> **Status: Alpha → Phase 1** — Core functionality implemented and compiles cleanly. Currently executing **Definitive Implementation Plan** for production 1.0 (13-week roadmap). Phase 1 (Secure & Stabilize) in progress: input validation, error handling, and CI/CD infrastructure.

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
npm test

# Run Rust tests (requires glib-2.0 dev package on Linux)
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

## Implementation Roadmap (13-Week Plan to 1.0)

A comprehensive **Definitive Implementation Plan** governs all development:

### Phase 1: Secure & Stabilize (Weeks 1-2) — **IN PROGRESS**
- ✅ Input validation layer (IP, port, hostname, device name validation)
- ✅ AppError type with structured error codes and context
- 🔄 Error event emission from backend to frontend
- 🔄 Error store and Toast notification UI
- 📋 Rust backend CI pipeline (GitHub Actions for all platforms)

### Phase 2: Cross-Platform (Weeks 3-5)
- Linux scanner implementation (`ip neigh`, `ip addr` instead of `arp`/`ifconfig`)
- Windows scanner implementation (PowerShell commands)
- Integration tests (full scan workflows)
- CI matrix for Linux, macOS, Windows
- Platform-specific README instructions

### Phase 3: Test & Validate (Weeks 6-7)
- Vitest setup for frontend component tests
- Component tests for all 13 UI components
- E2E tests with Tauri driver (user workflows)
- Error scenario tests (missing commands, malformed input, DB corruption)

### Phase 4: User Features (Weeks 8-9)
- Custom alert rules UI (users create conditions)
- IPv6 support (discovery and scanning)
- Performance optimization (pagination, graph culling)

### Phase 5: Release & Distribution (Weeks 10-11)
- GitHub Actions release pipeline (build binaries for all platforms)
- Binary signing and notarization
- GitHub Releases with downloadable installers

### Phase 6: Polish & Harden (Weeks 12-13)
- Database encryption (sqlcipher)
- Export encryption UI
- Error recovery and graceful degradation

**Full Plan Document:** See `IMPLEMENTATION_PLAN.md` (auto-generated from definitive plan).

## Known Limitations (Phase 1 Status)

- **macOS only** (Phase 2 adds Linux/Windows) — Network commands are platform-specific
- **No IPv6** (Phase 4B adds IPv6) — Discovery and scanning is IPv4 only
- **Top 100 ports only** (not yet parameterized) — Placeholder for port range selection
- **No custom alert rules** (Phase 4A) — Rules are hardcoded, UI allows enable/disable only

## License

MIT
