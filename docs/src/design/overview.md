# Snora Design — Overview

Snora Design is an **optional layer** on top of the snora layout engine. It
provides contrast-tested design tokens, a thin iced style bridge, and shallow
helpers for standard button and card surfaces — without prescribing a complete
design system or replacing iced's own theming.

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
snora = { version = "0.19", default-features = false }
```

**Default** — snora's existing layout + prefab widgets, no design tokens:

```toml
snora = { version = "0.19" }   # default = ["widgets"]
```

**Design** — layout + widgets + Snora Design tokens and helpers:

```toml
snora = { version = "0.19", features = ["widgets", "design"] }
```

For the full feature flag reference see [Feature flags](feature-flags.md).
