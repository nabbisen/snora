# Snora Design — Overview

Snora is not becoming a general widget component framework. Snora remains
a small layout and overlay framework for iced-based desktop applications.

By default, snora positions and stacks: applications supply content and
styling, and snora's own chrome carries only the minimum needed to be
legible.

When the optional `design` feature is active, Snora Design additionally
supplies a coherent visual default for the surfaces snora itself renders
— chrome, overlays, and notification surfaces — derived from tokens the
application owns and may replace. Applications still own their domain
behavior, complex widgets, validation, data presentation, navigation, and
final brand identity.

Surface coverage arrives incrementally. As of v0.26.0, chrome colours
follow the emitted theme automatically — the prefab widgets already read
iced's palette. As of v0.27.0, the dialog card, the modal dim
(`snora::design::render`, RFC-039), and chrome spacing/radius
(`snora::design::widget::*`, RFC-040) are also token-derived — but not
automatically: each requires calling the corresponding `design`-gated
entry point instead of its unstyled `snora::render` /
`snora::widget::*` counterpart, per the compatibility promise below.
Elevation remains out of scope: `Tokens` carries no shadow or elevation
scale, and adding one is a frozen-surface change (RFC-036) no RFC in
this milestone made.

**Compatibility promise.** With `design` inactive, snora's rendered output
is unchanged from v0.25 — this is a guarantee, not an aspiration. The
skeleton remains the default; the coherent appearance above is what
activating the feature buys.

Enabled via the `design` feature (opt-in; the default is `["widgets"]`).

## What it provides

- **`snora-design` crate** — iced-free vocabulary: `Tokens`, `Palette`,
  `Color`, `Spacing`, `Typography`, `Radius`, `FocusTokens`, and the
  `Tone` / `Emphasis` / `Size` / `Density` variant enums.
- **Four built-in token presets** — `light`, `dark`, `high_contrast_light`,
  `high_contrast_dark` — each with verified WCAG AA contrast.
- **iced style bridge** (`snora::design::style`) — maps tokens to
  `iced::widget::button::Style` and `iced::widget::container::Style`.
- **Pilot button helpers** (`snora::design::button`) — `primary`, `secondary`,
  `ghost`, `danger`, and their `*_maybe` disabled-state variants.
- **Pilot card helpers** (`snora::design::card`) — `surface`, `raised`,
  `selected`.

## What it does not do

- Replace iced's `Theme` or the snora layout engine.
- Guarantee that arbitrary app content is accessible (see
  [Accessibility checklist](../contributing/accessibility-checklist.md)).
- Provide forms, data tables, charts, or domain-specific widgets.
- Apply OS contrast or reduced-motion settings automatically.

## Three usage paths

**Minimal** — no design feature; iced's default theme only:

```toml
snora = { version = "0.38", default-features = false }
```

**Default** — snora's existing layout + prefab widgets, no design tokens:

```toml
snora = { version = "0.38" }   # default = ["widgets"]
```

**Design** — layout + widgets + Snora Design tokens and helpers:

```toml
snora = { version = "0.38", features = ["widgets", "design"] }
```

For the full feature flag reference see [Feature flags](feature-flags.md).
