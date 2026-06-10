# RFC-017-B — RTL Render-Semantics Integration Tests

**Status.** Implemented (v0.17.0)
**Tracks.** Test coverage / ABDD integration.
**Touches.** `crates/snora/tests/render_semantics.rs` (2 new tests).

## 1. Gap addressed

Prior render-semantics tests covered z-stack ordering, backdrop dismissal,
modal blocking, and toast visibility — but zero tests exercised
`LayoutDirection::Rtl`. The RTL path through `render()` was covered only
by `horizontal_align` unit tests in `toast.rs`. An integration regression
in the RTL mirroring path could go undetected.

## 2. Two new tests

`sheet_end_edge_reachable_under_rtl` — verifies `Sheet::new(...).at(SheetEdge::End)`
with `AppLayout::direction(LayoutDirection::Rtl)` renders interactive
sheet content. The content is findable and clickable.

`toast_dismiss_reachable_under_rtl` — verifies toast dismiss button is
findable and fires its message under `LayoutDirection::Rtl` with
`ToastPosition::TopEnd`.

## 3. Acceptance criteria

- Both tests pass in the headless `iced_test` simulator.
- Total render-semantics test count: 10 (was 8).
- Gate 5 updated: "10 tests including 2 RTL."
