# Migrating from 0.18 to 0.19

v0.19 is an **additive release** on the 1.0-gate track. There are no breaking
changes to any existing public API. All existing snora applications compile
unchanged.

## What changed

### New: Snora Design System foundation (opt-in)

v0.19 introduces the foundation of the Snora Design System as an **opt-in**
layer. Nothing in your existing code is affected unless you explicitly add
the `design` feature.

The new `snora-design` crate (iced-free design tokens) is a workspace member
but is **not published** in this release (`publish = false`). It is groundwork
for v0.20, which will activate the design feature and publish the crate.

To preview the design system now:

```toml
snora = { version = "0.19", features = ["widgets", "design"] }
```

This gives access to `snora::design::{Tokens, Palette, Color, …}`, the iced
style bridge (`snora::design::style`), and the pilot button and card helpers
(`snora::design::button`, `snora::design::card`).

**No obligation to use it.** The default feature set is unchanged:
`default = ["widgets"]`.

### New: design feature CI gates

The `ci.yaml` workflow now checks three additional design feature combinations
(`widgets,design`; `widgets,design,lucide-icons`; `widgets,design,svg-icons`)
and a new `design-isolation` job that enforces the iced-free guarantee on
`snora-design`.

### Docs improvements

- New `docs/src/design/` section: overview, feature flags, tokens, high
  contrast, buttons, cards, iced style bridge, and v0.21 primitives preview.
- New contributing pages: accessibility checklist, semantic accessibility
  policy, recipes and dogfood process, API governance.
- `mdbook test docs` now passes cleanly (fixed pre-existing fence-tag issues
  in `vocabulary.md`, `widgets.md`, and `anchored-popover-design.md`).
- `book.toml`: removed deprecated `multilingual` key (mdbook 0.5 compat).

## Upgrade steps

1. In `Cargo.toml`, change `snora = "0.18"` to `snora = "0.19"`.
2. Run `cargo check`. No other changes required.

## Versioning questions (per policy)

| Question | Answer |
|---|---|
| Does this break any public API? | No |
| Does any type rename or move? | No |
| Does a default behavior change? | No |
| Does a new public item require downstream action? | No — all new items are feature-gated behind `design` |
