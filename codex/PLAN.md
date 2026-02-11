# DELTA PLAN

## A) Executive Summary

### Current state (repo-grounded)
- Tauri backend emits real-time events for scan progress/completion, alerts, monitor status, and departed devices from `scanner/orchestrator.rs` and `commands/settings.rs`.
- Frontend consumes these via centralized event bridge `src/lib/services/tauri-events.ts` and store reducers in `src/lib/stores/*.svelte.ts`.
- Latest commit already aligned `device:departed` payload to `{ deviceId }` and monitor status to camelCase fields.
- Alert UI currently uses snake_case alert event types (`new_device`, `device_departed`, etc.) in both filter page and icon rendering.
- Alert typing currently conflates alert events and alert rules in one `AlertType` union (`src/lib/types/alert.ts`).
- Backend rules table uses `untrusted_device` (`src-tauri/migrations/001_initial.sql`), while generated alert events for that case are `unknown_device` (`src-tauri/src/alerts/engine.rs`).
- Monitor loop currently hardcodes scan config (`interface_id = "auto"`, quick + top100) in `src-tauri/src/commands/settings.rs`.
- Manual scan controls now use persisted frontend settings for `portRange` and monitor interval in `src/lib/components/scanning/ScanControls.svelte`.

### Key risks
- Type conflation risk: `Alert.ruleType` and `Alert.alertType` are semantically different domains but share one type alias.
- Contract drift risk between backend rule identifiers and frontend type model may hide real data mismatches.
- Monitor loop hardcoding weakens product invariant that DB settings are source-of-truth.
- Event contract changes are high-risk because they affect runtime event wiring across Rust + Svelte layers.

### Improvement themes (prioritized)
1. Harden alert type contracts (separate event types vs rule types).
2. Make monitor scan config settings-driven from backend persisted settings.
3. Add auditable session artifacts and verification trail for interruption-safe execution.

## B) Constraints & Invariants (Repo-derived)

### Explicit invariants
- `tauri-bridge.ts` remains the only file calling `invoke()`.
- `tauri-events.ts` remains the only file calling `listen()`.
- App state in scanner pipeline remains DB-backed (`db::queries`) and event-driven for UI updates.

### Implicit invariants (inferred)
- Event payload names are camelCase at JS boundary via serde rename in Rust.
- Scan monitor should honor user settings stored in SQLite (`settings` table), not separate ephemeral values.
- Alert rule types and generated alert types are not identical and must be modeled distinctly.

### Non-goals
- No schema migration changes in this session.
- No broad TypeScript baseline cleanup (existing `npm run check` errors are pre-existing and out of scope).
- No platform-portability changes for scanner commands.

## C) Proposed Changes by Theme (Prioritized)

### Theme 1: Alert contract hardening
- Current approach:
  - Single `AlertType` union used for both alert event types and rule types.
- Proposed change:
  - Split into `AlertEventType` and `AlertRuleType`.
  - `Alert.alertType` uses event type union (`unknown_device` included).
  - `AlertRule.ruleType` uses rule type union (`untrusted_device` included).
- Why:
  - Matches backend semantics and avoids silent contract skew.
- Tradeoffs:
  - Slightly more type verbosity.
  - Alternative rejected: keep broad `string`; too weak.
- Scope boundary:
  - Types + alert page filter typing only.

### Theme 2: Monitor config from persisted settings
- Current approach:
  - monitor loop uses hardcoded `auto/quick/top100`.
- Proposed change:
  - Read settings from DB before each monitor scan; derive interface and port range from settings.
- Why:
  - Enforces “DB is source-of-truth” and aligns monitor behavior with settings UI.
- Tradeoffs:
  - One DB read per monitor interval.
  - Alternative rejected: pass full settings from UI once; stale if settings change later.
- Scope boundary:
  - `commands/settings.rs` only.

### Theme 3: Operational auditability artifacts
- Current approach:
  - No session artifacts existed.
- Proposed change:
  - Maintain `codex/*.md` logs/checkpoints/decisions/changelog draft.
- Why:
  - Required for interruption-safe autonomous execution.

## D) File/Module Delta (Exact)

### ADD
- `codex/SESSION_LOG.md` — chronological execution log.
- `codex/PLAN.md` — this delta plan.
- `codex/DECISIONS.md` — judgment call register.
- `codex/CHECKPOINTS.md` — checkpoint ledger.
- `codex/VERIFICATION.md` — command evidence.
- `codex/CHANGELOG_DRAFT.md` — delivery draft.

### MODIFY
- `src/lib/types/alert.ts` — split alert event and rule type unions.
- `src/routes/alerts/+page.svelte` — filter typing to `AlertEventType`.
- `src-tauri/src/commands/settings.rs` — derive monitor scan config from persisted settings.

### REMOVE/DEPRECATE
- None.

### Boundary rules
- No changes to migration SQL or DB schema.
- No changes to tauri event names established in latest commit.

## E) Data Models & API Contracts (Delta)

### Current
- Frontend alert contracts defined in `src/lib/types/alert.ts`.
- Backend rule types seeded by migration and read via db queries.

### Proposed changes
- Introduce:
  - `AlertEventType = 'new_device' | 'device_departed' | 'port_changed' | 'unknown_device'`
  - `AlertRuleType = 'new_device' | 'device_departed' | 'port_changed' | 'untrusted_device'`
- Keep wire format unchanged.

### Compatibility
- Backward compatible runtime behavior; compile-time strictness improved.

### Migrations
- None.

### Versioning strategy
- Internal type contract refinement only (no external API version bump).

## F) Implementation Sequence (Dependency-Explicit)

1. **Step S1 — Add session artifact scaffolding + discovery checkpoint**
   - Files: `codex/*`
   - Preconditions: baseline verification captured.
   - Verification: `git status --short` (artifact-only delta).
   - Rollback: remove added files.

2. **Step S2 — Alert type model split**
   - Files: `src/lib/types/alert.ts`, `src/routes/alerts/+page.svelte`
   - Preconditions: S1 complete.
   - Dependencies: none.
   - Verification: `npm run check` (expect baseline errors only; no new alert-type errors).
   - Rollback: revert two files.

3. **Step S3 — Monitor loop settings-driven scan config (risky: contract-adjacent)**
   - Files: `src-tauri/src/commands/settings.rs`
   - Preconditions: S2 complete.
   - Dependencies: existing `db_settings::get_settings` and `ScanConfig`.
   - Verification: `rustfmt --edition 2021 src-tauri/src/commands/settings.rs` + `cd src-tauri && cargo test` (expect baseline env failure only).
   - Rollback: revert file.

4. **Step S4 — Final hardening + documentation + changelog**
   - Files: `codex/*`
   - Verification: rerun baseline command set, record deltas.

## G) Error Handling & Edge Cases

- Current error pattern: Rust commands return `Result<_, String>` and log failures.
- Proposed improvements:
  - Preserve existing pattern.
  - For monitor config parse, safely map string `port_range` to enum with fallback `Top100` and warning log.
- Edge cases:
  - Missing settings row values -> existing defaults from `get_settings` already applied.
  - Unexpected `port_range` string -> explicit fallback with warning.

## H) Integration & Testing Strategy

- Integration points:
  - Frontend alert type declarations consumed by routes/components/stores.
  - Monitor settings lookup consumed by scan orchestration.
- Tests:
  - No new automated tests added due baseline environment constraints.
  - Use targeted static checks and compile-format checks.
- Definition of done:
  - No new verification failures beyond baseline.
  - Alert contracts are semantically separated.
  - Monitor scan config is settings-driven.

## I) Assumptions & Judgment Calls

### Assumptions
- User dissatisfaction relates to quality/contract rigor of latest commit.
- No hidden external inline comments were delivered outside prompt content.

### Judgment calls
- Chose not to revert prior commit wholesale; instead harden and narrow-risk improvements.
- Chose backend settings read per monitor cycle to keep live settings consistency.
