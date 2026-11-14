# High contrast

Snora Design ships two high-contrast presets that exceed WCAG AA for all
mandatory text pairs — `high_contrast_light` and `high_contrast_dark`.

## When to offer high contrast

Offer a high-contrast toggle when your application targets users who rely on
strong visual separation: users with low vision, users who work in bright
environments, or users who explicitly request higher contrast from OS settings.

Snora Design does not detect OS contrast preferences automatically in v0.20.
Preference detection is application responsibility.

## What the presets guarantee

Every built-in high-contrast palette passes all of the following pairs at
`>= 7:1` for primary text and `>= 4.5:1` for all mandatory status pairs:

| Pair | Light HC | Dark HC |
|---|---|---|
| `text_primary` on `background` | 21:1 | 21:1 |
| `text_secondary` on `background` | 17:1 | 17:1 |
| `accent_text` on `accent` | 10:1 | 12:1 |
| `danger_text` on `danger` | 9:1 | 10:1 |
| `focus` on `background` | 10:1 | 21:1 |

These ratios are verified by the automated contrast test suite
(`cargo test -p snora-design`) and must not regress.

## Characteristic differences from light/dark

- **Borders are solid black / solid white.** The `border` role is `#000000`
  (light HC) or `#FFFFFF` (dark HC): 21:1 against the page and `surface`, and
  19.80:1 against `high_contrast_dark`'s `surface_raised`.
- **Surfaces nearly collapse to a single background.** `background` and
  `surface` are the same colour; `surface_raised` is too in
  `high_contrast_light`, and is `#0a0a0a` in `high_contrast_dark`.
  Separation comes from borders, not shading. **An element with neither has
  none:** a progress bar's track is invisible against the page in both
  presets (1.00:1). *(Corrected 2026-11-15, from the 0.63 audit: this said the
  three surfaces were the same colour, and that every card, panel and input was clearly
  delineated.)*
- **Focus ring is thicker.** `FocusTokens.ring_width` is `3.0` in the
  high-contrast presets versus `2.0` in light/dark.

## Customizing within high contrast

If you replace palette roles in a high-contrast preset, run the contrast
verification in your own tests:

```rust,ignore
{{#include ../../../examples/book_snippets/src/high_contrast.rs:high_contrast_custom_verification}}
```

## Visual fit checklist (high contrast)

When testing under a high-contrast preset in the workbench:

- All card borders must be clearly visible.
- All swatch backgrounds must be visually distinct.
- Button text must meet 7:1 at rest, hovered and pressed (snora tests this on
  the drawn pixels). **Disabled is exempt** (WCAG 1.4.3): disabled labels read
  2.5–3.0:1 here, and a disabled control is told apart by its dimming
  (RFC-106), not by readable text.
- The focus ring at `3.0` width must be visible when tabbing through
  controls. From 0.55 snora's controls draw it (RFC-116), so this item can
  pass: route Tab with `snora::keyboard::traverse` and look. The wider ring
  reaches 1 px inside the control (3 px wide, 2 px out); a `button::link`
  keeps its underline clear of it.
