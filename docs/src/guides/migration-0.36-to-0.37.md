# Migration 0.36 → 0.37

> The `design`-path modal dim is stronger — a WCAG 2.1 SC 1.4.11 repair,
> not a restyle.

## Who is affected

Every `design`-path application: anything rendering through
`snora::design::render` or `snora::design::responsive_render` and
opening a `Dialog` or `Sheet`. The unstyled/engine path
(`snora::render`, `snora::responsive_render`) is unaffected — it dims
with a fixed literal and draws no card, so this repair does not apply
there.

## What changed

The modal dim's alpha, `snora_design::surfaces::DIM_ALPHA`: `0.40` →
`0.44`. The dim's base color (black in light presets, white in dark —
whichever pole is opposite `background`'s own darkness) is unchanged.

The derivation itself also moved. It previously lived as a private
`dim_color` helper in `crates/snora/src/design/render.rs`, calling
`iced::theme::palette::is_dark` directly. It now lives as
`snora_design::surfaces::modal_dim`, a pure function over `Tokens` —
`snora-design` reimplements the same OKLCH-lightness classification
without depending on iced to compute it. `dim_color` is now a thin
adapter that calls it. Neither function is public API; this is an
internal relocation, not a signature change.

## Why it changed

RFC-063 closed the *role* axis: no `Palette` role can be added without
declaring where it renders, compiler-enforced. It did not close the
*surface* axis. The modal dim is not a `Palette` role — it is composited
at render time — so `Palette::usages` could not see it, and nothing in
`snora-design`'s contrast suite ever measured it.

Measured (RFC-065), the dialog card in the `light` preset was
distinguishable from its own dimmed backdrop at **2.85:1** by either
signal (border or fill), below SC 1.4.11's 3:1. The other three built-in
presets already passed:

| preset | before (0.40) | after (0.44) |
|---|---|---|
| `light` | **2.85 — FAIL** | **3.24** |
| `dark` | 3.18 | 3.64 |
| `high_contrast_light` | 7.37 | 6.48 |
| `high_contrast_dark` | 5.25 | 4.56 |

(Worst case across the three neutral surfaces the dim can sit over —
`background`, `surface`, `surface_raised` — and the better of the
card's border or fill signal against that backdrop, per SC 1.4.11's
either-signal rule. `0.42` would have cleared `light` too, at 3.04:1, a
1.3% margin; `0.44` was chosen for an 8% margin instead, following the
precedent RFC-058 set when repairing `border`.)

## Mechanical migration

None. No public API changed.

## Behavioral migration

**This is an appearance change**, not a behavior change: the modal dim
behind an open `Dialog` or `Sheet` on the `design` path is visibly
darker — a 10% relative increase in opacity (0.40 → 0.44). Re-check any
screenshot tests or visual regression baselines that include an open
dialog or sheet on the `design` path.

The direction is one-way: the dialog card becomes easier to distinguish
from the page behind it in every preset it was already passing in, and
newly distinguishable in `light`, where it previously was not. A
stronger dim also obscures slightly more of the content behind the
modal — a legibility trade the owner weighed against the contrast
repair when accepting RFC-065, not a side effect to discover later.

## Deprecated aliases and removal schedule

None — this is a rendering change, not an API change.

## Examples before/after

No repository example constructs the modal dim directly; all examples
call `render`/`responsive_render` through their normal signatures and
pick up the repaired dim automatically. No example changes were
required.
