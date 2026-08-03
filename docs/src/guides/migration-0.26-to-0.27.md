# Migrating from 0.26 to 0.27

v0.27 completes the appearance milestone. v0.26 made chrome **colours**
follow your design tokens; v0.27 adds the **surfaces and geometry** —
the dialog card, the modal dim, and chrome spacing and radius.

**No breaking changes to any public API.** Everything new is additive and
behind the opt-in `design` feature. If your application does not enable
`design`, **snora's rendered output is unchanged from v0.26.**

## Read this first if you enable `design` and see no difference

Enabling the feature is **not** sufficient. Unlike theme emission — where
setting the theme makes chrome colours follow automatically — the new
surfaces and geometry require you to **call different functions**:

| Instead of | Call |
|---|---|
| `snora::render(layout)` | `snora::design::render(layout, &tokens)` |
| `snora::widget::app_header(…)` | `snora::design::widget::app_header(&tokens, …)` |
| `snora::widget::app_side_bar(…)` | `snora::design::widget::app_side_bar(&tokens, …)` |
| `snora::widget::app_footer(…)` | `snora::design::widget::app_footer(&tokens, …)` |
| `snora::widget::app_tab_bar(…)` | `snora::design::widget::app_tab_bar(&tokens, …)` |
| `snora::widget::app_breadcrumb(…)` | `snora::design::widget::app_breadcrumb(&tokens, …)` |

This is deliberate. Existing call sites keep their exact appearance, so
upgrading cannot change how your application looks — the styled variants
are opt-in per call site, not per feature flag.

## What changed

### The dialog has a card (RFC-039)

Previously `snora::render` centred your dialog content with **no
background, padding, radius or border** — bare content floating on the dim.

Through `snora::design::render` it now gets a card: `surface_raised` fill,
`border` edge, `radius.lg`, `spacing.lg` padding.

The card is **border-defined rather than shadow-defined**. Two reasons:
shadows carry almost no information in the high-contrast presets, and in
two of the four built-in presets `surface_raised` is *bitwise identical* to
`background` — so the card is visible there **only** because of its border.

### The modal dim is derived, and was invisible in one preset (RFC-039)

The dim was a hardcoded `rgba(0, 0, 0, 0.4)`. Its colour is now chosen from
your background's own darkness.

**If you use `high_contrast_dark`, this is a real fix.** That preset's
background is pure black, and 40% black over pure black composites to
*pure black* — the dim has been completely invisible, meaning modals have
had no visual modality signal at all in the preset whose purpose is maximum
legibility.

The dim's strength is unchanged (still 40%); only its colour is derived.

Note the caveat: this fix applies through `snora::design::render`. The
unstyled `snora::render` keeps the old dim, by the compatibility promise
above.

### Chrome geometry follows the token scales (RFC-040)

Chrome previously hardcoded seven different padding shapes across six
widgets, with corner radii of `0.0` in one place and `6.0` in another. The
styled variants derive spacing from `Spacing` and radius from `Radius`, so
chrome shares one rhythm.

Some values changed deliberately rather than being mapped to match — the
sidebar's item gap moves from 16 to 12, for example, because the scale says
so. The full mapping, including which values happen to coincide with the
old ones, is in
[`design/chrome-geometry.md`](../design/chrome-geometry.md).

### Measurement integrity (RFC-044)

Not user-facing. The `runner_os` column in the budget CSVs now emits
`ubuntu-latest` rather than `Linux` — GitHub reserves the `RUNNER_*`
variable namespace, so v0.26's override was silently ignored. The release
checklist now verifies row *contents*, not just that a row exists.

## What is still not token-derived

Being explicit, since v0.26's guide made the same commitment:

- **Elevation and shadows.** `Tokens` carries no shadow or elevation
  scale. Adding one is a frozen-surface change under the design-surface
  covenant, and no RFC in this milestone made it.
- **Toast surfaces.** Toasts render on the design-inactive path too, so
  restyling them would change appearance for applications that never opted
  in. Deferred.
- **The sheet.** It already has edge-aware rounding and an opaque wrapper;
  it is not visually broken in the way the dialog was.

## Upgrading

1. Change `snora = "0.26"` to `snora = "0.27"` in `Cargo.toml`.
2. That is the whole migration if you do not use `design`.
3. If you use `design` and want the new surfaces, switch the call sites in
   the table above. Each is independent — you can adopt the dialog card
   without touching chrome, or vice versa.

## Minimum supported Rust version

Unchanged: **1.88**, inherited from `iced` and `wgpu`.
