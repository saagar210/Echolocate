# CHANGELOG DRAFT

## Unreleased

### Theme: Alert contract correctness
- Split frontend alert typing into two explicit domains:
  - `AlertEventType` for emitted alert records.
  - `AlertRuleType` for configurable rule rows.
- Updated alerts page filter typing to use `AlertEventType`.

### Theme: Monitor settings consistency
- Monitor scan loop now derives runtime scan configuration from persisted settings on each cycle.
- Added fallback behavior:
  - Missing settings load => safe default config (`auto`, quick, top100).
  - Unknown `port_range` value => warning log and fallback `top100`.

### Theme: Session auditability
- Added codex session artifacts:
  - `SESSION_LOG.md`, `PLAN.md`, `DECISIONS.md`, `CHECKPOINTS.md`, `VERIFICATION.md`, `CHANGELOG_DRAFT.md`.
