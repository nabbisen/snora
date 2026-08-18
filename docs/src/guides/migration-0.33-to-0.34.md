# Migration 0.33 → 0.34

> `light` and `dark`'s `border` colour changed, and `light`'s `text_muted`
> colour shifted slightly — both WCAG accessibility repairs, not a restyle.
> Borders become more visible; nothing becomes harder to see.

## Who is affected

Every application using the `design` feature with the built-in `light` or
`dark` presets. If you construct your own `Tokens` with custom `border` /
`text_muted` values, or use `high_contrast_light` / `high_contrast_dark`
(unchanged), you are not affected.

Affected rendering: card borders (`card::surface`, `card::raised`,
`card::selected`), the dialog card (`design::render`'s modal, which RFC-039
chose to identify by border rather than shadow), any application text using
`snora::design::style::container::*` directly, and any application text
styled with `tokens.palette.text_muted` under `light`.

## What changed

`Palette::border` and (in `light` only) `Palette::text_muted`:

| Preset | Field | Before | After |
|---|---|---|---|
| `light` | `border` | `Color::rgb(0.843137, 0.858824, 0.878431)` | `Color::rgb(0.537255, 0.549020, 0.560784)` |
| `dark` | `border` | `Color::rgb(0.168627, 0.192157, 0.227451)` | `Color::rgb(0.411765, 0.443137, 0.490196)` |
| `light` | `text_muted` | `Color::rgb(0.419608, 0.447059, 0.501961)` | `Color::rgb(0.414573, 0.441694, 0.495937)` |

`dark`'s `text_muted` is **unchanged** — it already passes every asserted
pair. `high_contrast_light` and `high_contrast_dark` are unchanged for both
fields — they already pass at 19.8–21:1 (border) and 10.9–11.5:1
(`text_muted`).

## Why it changed

`crates/snora-design/src/tests.rs` asserted twelve contrast pairs; `border`
and `text_muted` were in none of them. `border` was failing: measured
**1.19–1.43:1** against surfaces in `light` and `dark`, against WCAG 2.1
SC 1.4.11's 3:1 minimum for non-text boundaries that identify a component.
For at least one surface — the RFC-039 dialog card, deliberately chosen as
border-defined rather than shadow-defined — the border **is** that
identifying boundary.

`text_muted` was also failing, narrowly, on one pair: `light/surface`
measured **4.4626:1** against WCAG 2.1 SC 1.4.3's 4.5:1 for body text.
`Palette`'s doc comment previously claimed `text_muted` was exempt from
this requirement; that exemption is not one WCAG 2.1 SC 1.4.3 grants (its
exemptions are incidental/decorative/invisible text, logotypes, and large
text at 3:1) and has been withdrawn.

This is the first exercise of RFC-036's accessibility carve-out: preset
values are additive-only *except* where a contrast test proves a defect,
recorded as **Fixed**. For both fields, the assertion was added first, the
failure captured, then the value repaired to clear the **binding** pair —
for `border`, the surface closest in luminance to the border (the
*darkest* of the three surfaces in `light`, the *lightest* in `dark`), not
merely the easiest pair to pass; for `text_muted`, the one failing pair in
`light` only — `dark` was left untouched because it already passes every
pair, and the carve-out permits a value change only where a test proves a
defect. See `CHANGELOG.md` for the full measured before/after across all
three surfaces per preset, for both fields.

## Mechanical migration

None. No API changed; `Tokens::light()` / `Tokens::dark()` return the same
type with different field values. If you read `tokens.palette.border`
directly (for a custom widget style, say), no code change is needed — the
value it returns changed automatically with the preset.

## Behavioral migration

**This is an appearance change**, not a behavior change: borders in `light`
and `dark` render visibly darker/lighter respectively — closer to a mid grey
than the previous near-invisible tint. Re-check any screenshot tests or
visual regression baselines that include card or dialog borders under the
`design` feature's `light`/`dark` presets.

`light`'s `text_muted` shift is a **1-2 of 255 per-channel** change and is
not expected to be visually perceptible; it is listed for completeness and
because a pixel-exact screenshot diff would still catch it.

The direction is one-way for both fields: contrast increases. Nothing that
was visible becomes harder to see.

## Deprecated aliases and removal schedule

None — these are value changes, not API changes.

## Examples before/after

No repository example hardcodes a border or `text_muted` colour; all
examples read `tokens.palette.{border,text_muted}` through the built-in
presets and pick up the new values automatically. No example changes were
required.
