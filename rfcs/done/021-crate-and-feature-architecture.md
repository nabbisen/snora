# RFC 021 — Crate and Feature Architecture

**Status.** Implemented (v0.19.0)
**Tracks.** Snora Design System migration; v0.20 foundation.
**Touches.** `Cargo.toml`, workspace crates, feature flags, facade re-exports.

## Summary

This RFC defines the crate boundaries and feature-flag architecture for Snora Design.

## Crate responsibilities

```text
snora-core
  AppLayout and layout/overlay vocabulary.
  No iced.
  No design tokens unless strictly layout-related.

snora-design
  Design tokens and abstract design vocabulary.
  Preferably iced-free.
  No widget rendering.
  No iced style types.

snora-widgets
  iced prefab widgets.
  iced style helpers.
  iced implementation of Snora Design.
  May consume snora-design under feature gates.

snora
  Render engine.
  Overlay stack.
  Toast lifecycle.
  Facade re-exports.
  Feature coordination.
```

## Dependency direction

```text
snora-core
    ↑
snora-design
    ↑
snora-widgets
    ↑
snora
```

`snora-design` must not depend on iced.

### crates.io publish order

Bottom-up along the dependency graph, with `snora-design` inserted:

```text
snora-core → snora-design → snora-widgets → snora
```

The release process doc must be updated from the current three-crate order
to this four-crate order.

## Feature model

Recommended shape:

```toml
[features]
# `design` is NOT default-on in v0.20. It stays opt-in until binary-size
# and build-cost measurement justify enabling it by default (the project
# tracks size/compile drift per release; a new crate + default design code
# would shift those baselines). Re-evaluate default-on after measurement.
default = ["widgets"]

widgets = ["dep:snora-widgets"]
design = ["dep:snora-design", "snora-widgets?/design"]

# Icon features keep the existing multi-crate propagation and remain
# independent of `design`. Do NOT collapse these to `["widgets"]`, which
# would silently disable icon rendering on snora-core / snora-widgets.
lucide-icons = [
    "widgets",
    "snora-core/lucide-icons",
    "snora-widgets?/lucide-icons",
    "dep:lucide-icons",
]
svg-icons = [
    "widgets",
    "snora-core/svg-icons",
    "snora-widgets?/svg-icons",
    "iced/svg",
]
```

Exact Cargo wiring may differ, but two things are fixed contract:
`default = ["widgets"]` until measurement, and the icon features preserve
their current snora-core + snora-widgets + dependency propagation.

## Required usage modes

### Minimal root Snora

```toml
snora = { version = "...", default-features = false }
```

Provides layout/render/overlay/toast behavior without design.

### Widgets-only

```toml
snora = { version = "...", default-features = false, features = ["widgets"] }
```

Existing widgets compile without `design`.

### Default ergonomic (widgets)

```toml
snora = "..."
```

Default features are `["widgets"]` — prefab widgets, no design. This keeps
the default binary size and build cost on the existing trend.

### Default + design (opt-in)

```toml
snora = { version = "...", features = ["design"] }
```

Adds Snora Design tokens and `snora::widget::design::*` helpers. `design`
stays opt-in in v0.20 pending size/build-cost measurement; default-on is a
later, measured decision.

### Token-only

Preferred:

```toml
snora-design = "..."
```

Root facade token-only support is optional if feature wiring remains simple:

```toml
snora = { version = "...", default-features = false, features = ["design"] }
```

Direct `snora-design` dependency is the stable escape hatch.

## Public re-export policy

With design enabled, root `snora` may re-export (enumerated, not glob):

```rust
snora::design::Tokens
snora::design::Palette
snora::design::Color
snora::design::Typography
snora::design::TextRole
snora::design::Spacing
snora::design::Radius
snora::design::FocusTokens
snora::design::Tone
snora::design::Emphasis
snora::design::Size
snora::design::Density
```

With widgets and design:

```rust
snora::widget::design::button
snora::widget::design::card
snora::widget::design::style
```

Base widget APIs must not expose `snora-design` types when design is disabled.

## Internal design

In `snora-widgets`:

```rust
#[cfg(feature = "design")]
pub mod design;
```

Base widget modules must compile without that module.

Icon features must remain orthogonal:

```text
widgets
  ├── lucide-icons
  ├── svg-icons
  └── design
```

## Data lifecycle

Tokens are app-owned:

```text
app chooses preset
  -> app optionally customizes tokens
  -> app stores tokens in state
  -> view passes tokens to style helpers
  -> user changes theme/high contrast
  -> app replaces tokens
```

No global token store is introduced in v0.20.

## Process handling

CI must test:

- minimal path;
- widgets without design;
- default;
- all features;
- release/scheduled icon/design combinations.

## Risks

### Cargo feature complexity

Mitigation:

- direct `snora-design` dependency for token-only users;
- two-tier CI matrix;
- root token-only facade optional.

### Accidental design dependency in widgets-only path

Mitigation:

- feature isolation gate;
- compile test `widgets` without `design`.

## Acceptance criteria

- Minimal path builds.
- Widgets-only path builds.
- Default features are `["widgets"]` (design is opt-in until measured).
- `snora-design` has no iced dependency.
- Icon features keep multi-crate propagation and do not require design.
- Facade re-exports are enumerated, not glob (`pub use ...::{...}`).
- Publish order updated to `snora-core → snora-design → snora-widgets → snora`.
- Docs explain all usage modes.
