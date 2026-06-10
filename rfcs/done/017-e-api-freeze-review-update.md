# RFC-017-E — API Freeze Review Update

**Status.** Implemented (v0.17.0)
**Tracks.** 1.0 governance.
**Touches.** `docs/src/contributing/api-freeze-review.md` (full rewrite
of status table), `ROADMAP.md` (gate status table updated).

## 1. Summary of changes to api-freeze-review.md

All five sections updated to reflect the state as of v0.17.0:

- Status header updated from "v0.13.0" to "v0.17.0".
- Type-names audit section replaced with completed audit results table.
- Semantic contract: RTL integration test row added (Gate 5).
- Documentation: `docs.rs` and versioning policy rows marked ✅.
- Release hygiene: first data-point rows updated.
- 1.0 gates: Gate 2 ✅, Gate 5 updated, Gate 9 first-point noted.
  Six of ten gates now satisfied.

## 2. Acceptance criteria

- `api-freeze-review.md` reflects six satisfied gates.
- Gate 2 text in ROADMAP matches.
- No gate is marked ✅ prematurely.
