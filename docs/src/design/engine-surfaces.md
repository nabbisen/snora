# Token-derived engine surfaces

RFC-038 made chrome *colours* follow the emitted theme. Two surfaces the
**engine** renders itself — not an application-built primitive — did not
participate at all: the dialog had no card (bare content on a fixed grey
wash), and the modal dim was a hardcoded `rgba(0, 0, 0, 0.4)`, independent
of the active preset. `snora::design::render(layout, &tokens)` (RFC-039)
gives both surfaces token-derived styling, as a sibling to
`snora::render`.

```rust,ignore
use snora::design::{Tokens, render};

let tokens = Tokens::light();
let element = render(layout, &tokens);
```

Snora never calls this function on the application's behalf. Nothing
changes for applications that keep calling `snora::render` — that
function's output is byte-for-byte unchanged with `design` inactive
(RFC-037's gating invariant), proven by `crates/snora/tests/
render_semantics.rs` passing without modification.

## Why a sibling function, not a field on `AppLayout`

`AppLayout` lives in `snora-core`, which has **no dependencies at all**.
Adding a `Tokens` field would pull `snora-design` into every engine-only
build, defeating the opt-in size discipline and inverting the documented
crate-dependency direction. `snora` already depends on both
`snora-core` and (behind `design`) `snora-design`, so the sibling
function lives there instead, sharing one z-stack implementation with
`snora::render` — the layer composition (order, conditions, backdrop
wiring) is written exactly once; only the dim color and the dialog's card
style differ between the two entry points.

## Working within the frozen token surface

RFC-036's additive-only covenant freezes `snora-design`'s `Palette` (18
roles) and `Tokens` (no shadow or elevation scale). Neither surface below
has a purpose-built token role. Both derive from existing roles instead
of extending the frozen surface — the owner-confirmed approach, since
extending it would require resetting RFC-036's D-3/D-4 decisions to open
in the same change.

### The dialog card

| Property | Token role |
|---|---|
| Fill | `surface_raised` |
| Border color | `border` |
| Border width | `1.0` |
| Radius | `radius.lg` |
| Padding | `spacing.lg` |

Reuses `snora::design::style::container::card_raised` (RFC-029) directly
— the exact same fill/border/radius mapping the card primitive already
uses — rather than recomputing it, with its drop shadow zeroed out.

**Border-defined, not shadow-defined**, deliberately. Two of the four
built-in presets (`light`, `high_contrast_light`) have `surface_raised ==
background` — the card's fill is *bitwise identical* to the page behind
it in those presets, by the token data's own design. The border is what
makes the card visible there, not the fill: `card_border_distinguishable_
from_background_all_presets` in `crates/snora/src/design/render/
tests.rs` tests the border's contrast against `background`, not the
fill's, precisely because a fill-vs-background test would assert
something false by construction in half the presets. This also motivated
the "border, not shadow" choice independently: shadows are close to
meaningless in the high-contrast presets (a light shadow on a
near-white background, or a barely-there shadow on near-black), so a
border is the one visual signal that works uniformly across all four.

Measured border-vs-background contrast, all four presets (re-derived at
0.38.1, RFC-071 — the figures below stood stale since 0.34.0):

| Preset | Contrast |
|---|---|
| `light` | 3.38:1 |
| `dark` | 3.81:1 |
| `high_contrast_light` | 21.0:1 |
| `high_contrast_dark` | 21.0:1 |

The floor tested is `NON_TEXT_MIN` (`3.0:1`, WCAG SC 1.4.11's own
non-text-contrast threshold) — what the `border` role, used directly
and unmodified (per the mapping above — no derived shift, unlike the
modal dim below), actually achieves across all four built-in presets,
with the worst case (`light`) clearing it by 13%. The border was
repaired to clear this bar specifically in 0.34.0 (RFC-058); the
assertion did not test the real bar until 0.38.1 (RFC-071) — for four
minors it tested a discount value (`1.3:1`) chosen when the border's
worst case was believed, incorrectly, to be 1.39:1.

Card text (`text_primary` on the `surface_raised` fill) meets WCAG AA in
all four presets — verified independently in `design/render/tests.rs`,
in addition to `card_raised`'s own coverage.

### The modal dim

Unstyled default: `iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4)` — opaque
black at 40% alpha, unchanged by this RFC.

Composited over a **dark** page background, black-on-black is close to a
no-op — the same class of defect RFC-038's `shift_away_from` was built
to prevent for derived theme tiers, here for a fixed constant instead of
a derived one. The styled dim instead picks its base color from
**`background`'s own darkness**, not a fixed pole:

```rust,ignore
let base = if is_dark(background) { Color::WHITE } else { Color::BLACK };
let dim_color = Color { a: 0.4, ..base };
```

using iced's own public `iced::theme::palette::is_dark`. Unlike
RFC-038's `shift_away_from`, this has **no clamping edge case**:
alpha-compositing a color chosen to be the *opposite* pole from the
background's own category can never degenerate to a no-op, because the
two poles cannot both describe the same background. The two cases that
broke RFC-038's first attempt — `light`'s pure-white background and
`high_contrast_dark`'s pure-black one — are not clamping cases here at
all, precisely because the derivation never tries to move a color away
from its own tone; it only ever chooses between two fixed, maximally
distinct poles.

Measured contrast of the composited dim against the plain background,
all four presets (re-derived at 0.38.1, RFC-071 — stale since 0.37.0,
the release that changed `DIM_ALPHA` without updating this table):

| Preset | Pole chosen | Contrast |
|---|---|---|
| `light` | black | 3.2424:1 |
| `dark` | white | 4.3798:1 |
| `high_contrast_light` | black | 3.2424:1 |
| `high_contrast_dark` | white | 4.2529:1 |

All four clear `NON_TEXT_MIN` (`3.0:1`) — `light` and
`high_contrast_light` by the thinnest margin, 8%, which is exactly why
`DIM_ALPHA` was set to `0.44` rather than the `0.42` that would have
cleared it by 1.3% (RFC-065). Shared with the card border floor above,
deliberately (RFC-071): both measure "is this element visually distinct
from the page behind it," not because the two elements are otherwise
related.

## What this RFC does not cover

Chrome geometry (header/sidebar/footer spacing and radius) is RFC-040.
The sheet is not restyled — it already has an `opaque()` wrapper and
edge-aware rounding and is not visually broken the way the dialog was.
`WARNING_COLOR` (toasts) is unchanged: toasts render on the
design-*inactive* path too, so changing them there would break the
gating invariant.
