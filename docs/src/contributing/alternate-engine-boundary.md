# Alternate engine boundary

## Why `snora-core` is iced-free

`snora-core` has no dependency on iced. This is valuable for three reasons:

1. **Vocabulary stability.** iced's API changes between major releases.
   Keeping vocabulary types (`AppLayout`, `Toast`, `LayoutDirection`, …)
   iced-free means they do not need to change when iced changes.
2. **Testability.** Pure data types are easy to test without a renderer.
   `AppLayout::new(body).header(h)` can be asserted in a unit test that
   never touches iced. The `pub` fields on vocabulary types make state
   assertions natural.
3. **Architectural clarity.** The boundary makes it explicit where Snora
   ends and iced begins. `snora-widgets` and `snora` depend on iced;
   `snora-core` does not.

This boundary was established in v0.6 (the three-crate workspace split)
and is a firm design decision.

## What an alternate engine would need

An engine that consumes `snora-core` vocabulary without iced would need
to implement:

| Capability | Notes |
|---|---|
| Compose base skeleton | header + (sidebar ∣ body) + footer row; respect `LayoutDirection` |
| Stack overlays deterministically | layers 0–7 per the z-stack contract |
| Capture pointer events on backdrop layers | click-to-close for menus and modals |
| Resolve logical Start/End to physical sides | under LTR and RTL |
| Manage transient toast sweep outside rendering | `sweep_expired` is already engine-independent |
| Accept app-supplied content nodes | the `Node` generic parameter on `AppLayout` |

## What is iced-specific and not portable

- `snora-widgets` entirely (all prefab widgets return `iced::Element`).
- `snora::render` returns `iced::Element`.
- The `iced_test` headless harness used in render-semantics tests.
- The `Renderer` and `Theme` type parameters that iced injects.

An alternate engine would need its own widget layer and cannot reuse
`snora-widgets`.

## Current status and public wording

`snora-core` is iced-free to keep vocabulary stable, testable, and
insulated from iced upgrades. It *may* be useful for alternate engines
in the future, but **Snora does not currently promise alternate renderer
support** and has no plans to build one.

The `iced_test` headless CPU renderer already covers internal test-double
needs (see `crates/snora/tests/render_semantics.rs`). A separate
test-double engine is not needed.

If you are building an alternate renderer that consumes `snora-core`,
open an issue with your use case. Snora may add guidance or minor
vocabulary adjustments if a concrete need appears.

## Related

- [Internal architecture](architecture.md)
- [Design decisions](design-decisions.md) — "Why three crates instead of two"
- RFC-016-A in `rfcs/done/`
