# Migration 0.41 → 0.42

> **Rendered appearance changes on both the stock and `design` theme
> paths.** Menu text, the sidebar's active highlight, chrome borders, tab
> labels, and breadcrumb text all change color. Every change is a
> contrast repair — text or a border that previously failed WCAG AA (or,
> for borders, the WCAG non-text minimum) now clears it. **If you hold
> reference images or visual-regression baselines that include any of
> these surfaces, they are invalidated**, the same precedent 0.34.0's
> border repair set.

## Who is affected

Anyone whose application uses, in either the default (stock `iced::Theme`)
or `design`-enabled path:

- The prefab header/context menu (`snora::widget::app_header`'s dropdown,
  or `render_menu` directly).
- The prefab sidebar (`app_side_bar`), specifically its active-item
  highlight.
- The prefab header, footer, or tab bar chrome border.
- The prefab tab bar's active-tab label color.
- The prefab breadcrumb's link color.

If your application supplies its own elements for all of these slots
(bypassing snora's prefab widgets entirely), nothing changes for you.

## What changed, and why

**The widget layer paired colours from different token families, and no
contrast suite in this project could see it.** Every contrast assertion
before this release lived in `snora-design` and tested tokens against
roles (`Palette::usages`) — a different, and already-correct, layer.
`snora-widgets` invents its own pairings at render time (a background-tier
colour used as a button's text, for instance), which a role-based suite
has no way to know exist. A new suite in `snora-widgets` itself now
measures every style this crate produces, against the background it is
actually painted over, on both theme paths.

| Widget | Was | Measured (worst case) | Now |
|---|---|---|---|
| Menu text (all states) | `primary.weak`/`.strong`/`.base` — a background-tier family used as text | 1.89:1 (stock light, at rest) | `background.base.text` — 4.5:1+ everywhere |
| Sidebar active highlight vs. rail | `primary.weak.color` | 1.89:1 (stock light); **1.51:1 under `high_contrast_dark`** | `primary.strong.color` — clears the 3.0:1 floor in every preset |
| Sidebar active item's icon/text | `background.base.text` — calibrated for the page background, not the highlight | 1.51–2.13:1 across presets | `primary.strong.text` — iced's own calibrated pairing for the new highlight |
| Chrome border (header/footer/tab bar) | `background.weak.color` | 1.02–1.48:1, every preset and both stock themes | `background.base.text` — clears the 3.0:1 floor everywhere |
| Active tab label | `primary.base.color` | 2.99:1 (stock dark) | `background.base.text` — the active/inactive underline (unchanged) now carries the state distinction instead of label colour |
| Breadcrumb text | `primary.base.color` | 2.03–3.42:1, stock themes | `background.base.text` / `background.weak.text`, matched to the actual background |

**The `high_contrast_dark` preset — the one that exists specifically for
low-vision users — was measured failing worse than every other preset
(1.51:1) before this release.** It is now the *best* of the four design
presets on every corrected pairing (worst case 9.96:1, versus 6.58:1 for
`light` and 8.59:1 for `dark`).

## What did not change

- No `snora-design` token value changed — `git diff -- crates/snora-design`
  for this release is empty. Every fix is a different choice of *which*
  existing, already-correct token the widget layer reads, not a new
  colour.
- No new `Palette` role — `RFC-036`'s frozen token surface is untouched.
- `Palette::usages`'s own contract is untouched; it declares role usage
  and continues to do that well. This release adds a *different*
  suite, in `snora-widgets`, for render-time pairings it was never meant
  to see.
- No public API removed or renamed.

## A visual trade-off, stated plainly

Two of these fixes drop a color-based state distinction that existed
before, because no color from the family being used could clear AA
against the actual background on **both** theme paths at once:

- **Menu items** no longer change color on hover/press — the text color
  is now the same in every state, since the background never changes
  and only one calibrated color is guaranteed safe against it.
- **The active tab's label** is now the same color as an inactive one;
  the underline (unchanged, still the theme's primary color) is the
  state indicator.

Both are candidates for a follow-up that reintroduces the distinction
via a **background** change on hover/press (the pattern the sidebar and
breadcrumb already use) rather than a foreground color — not done here,
to keep this release a contrast repair rather than a visual redesign.

## What is still a known, un-addressed concern

The sidebar's active-item highlight remains the **only** cue that an
item is active on the stock theme path (a WCAG 1.4.1 use-of-colour
consideration, separate from the contrast fix here) — it is far more
visible now than before, but a non-color cue (a border, an icon change)
would still be a more complete fix. Not added in this release, to keep
the change a re-pairing rather than a new visual element; tracked as a
follow-up, not silently dropped.

## If you are jumping more than one minor

Read the guides for the jumps in between — several carry real changes,
and the [migration index](migrations.md) lists them.
