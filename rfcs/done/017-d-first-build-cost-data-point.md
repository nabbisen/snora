# RFC-017-D — First Build-Cost Data Point

**Status.** Implemented (v0.17.0)
**Tracks.** Performance observability / Gate 9.
**Touches.** `docs/src/reference/binary-size-budget/binary-size.csv`,
`docs/src/reference/build-cost-budget/compile-time.csv`,
`docs/src/reference/performance-envelope/render-cost.csv`.

## 1. Purpose

Gate 9 requires binary-size and compile-time trends monitored with ≥2
data points. The infra has been ready since v0.10 (binary-size) and v0.12
(compile-time + render-cost). This RFC populates the first rows of all
three CSVs so the tracking system is proven end-to-end.

## 2. Data recorded (v0.17.0, sandbox environment)

Sandbox note: measured in the project build environment (headless Linux,
no GPU). Values reflect cold builds after `cargo clean`. The
`build_widgets_ms` column is `N/A` due to a `winit` platform error in the
sandbox; CI on real hardware will populate it. The binary-size diff is 0
because the sandbox stripped the same object for both feature variants
(same iced object reuse); CI will record the real diff.

These are "infra-proven" values, not representative performance numbers.
Gate 9 will be fully satisfied when a second data point exists (next
release on real CI hardware).

## 3. Acceptance criteria

- All three CSVs have a header row plus at least one data row.
- Gate 9 marked ⬜ with note "first point recorded v0.17; need second."
