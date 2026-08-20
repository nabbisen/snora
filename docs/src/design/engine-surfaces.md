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
it in those presets, by the token data's own design. Shadows are also
close to meaningless in the high-contrast presets (a light shadow on a
near-white background, or a barely-there shadow on near-black), so a
border is the one visual signal that works uniformly across all four —
that motivation for choosing a border over a shadow stands regardless of
what follows below.

**What that border protects, precisely, is narrower than "the card is
visible."** The table immediately below measures the border against
`background` — the **plain, undimmed** page. That is a real, useful
guarantee (`card_border_distinguishable_from_background_all_presets` in
`crates/snora/src/design/render/tests.rs` asserts it), but it is not the
condition the card actually renders under: `snora::design::render` only
draws a dialog card when `AppLayout.dialog` is populated, and that same
condition (`has_modal`) always paints the modal dim behind it too — a
real dialog's card never sits directly on the plain `background` this
table measures. **What the card's border is actually adjacent to, in
every dialog snora renders, is the dim.** See ["What actually separates
the card from the dim"](#what-actually-separates-the-card-from-the-dim)
below for that measurement — it is not this one, and the two disagree.

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
modal dim below), actually achieves against the plain page across all
four built-in presets, with the worst case (`light`) clearing it by 13%.
The border was repaired to clear this bar specifically in 0.34.0
(RFC-058); the assertion did not test the real bar until 0.38.1
(RFC-071) — for four minors it tested a discount value (`1.3:1`) chosen
when the border's worst case was believed, incorrectly, to be 1.39:1.
**This repair is still correct and still needed** — it is just answering
a narrower question than "does the border outline the card a user
actually sees," because that card always has the dim behind it, not the
plain background.

**Separately, and this is the pairing that matters for the card's own
edge**: the border against the card's **own fill**, at the inner
boundary between the two — 3.38:1 `light`, **3.17:1** `dark`, 21.0:1
`high_contrast_light`, 19.80:1 `high_contrast_dark` (re-derived, RFC-077;
`light` and `high_contrast_light` coincide with the table above because
`surface_raised == background` there by token design; `dark` and
`high_contrast_dark` do not, and were not previously stated on this
page). This is real and required — a border that cannot be told apart
from what it borders is not a border — and it clears its floor in every
preset too.

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
let dim_color = Color { a: DIM_ALPHA, ..base }; // DIM_ALPHA = 0.44
```

using a **private, local** `is_dark` — `snora-design` has no iced
dependency at all, enforced by a CI gate (RFC-021/022 Q3), so this
reimplements `iced::theme::palette::is_dark`'s own algorithm (same
sRGB→linear step, same OKLab matrices) rather than calling it. Unlike
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

### What actually separates the card from the dim

The two tables above — border vs. plain `background`, and dim vs. plain
`background` — each measure something real, but neither is "is the
dialog card's border visible against the modal dim behind it," which is
the condition every dialog snora actually renders under. **It is not,
reliably, on the border's own signal.**

**arama measured this directly, over real photographic content, and
found the border invisible against the dim: 1.02:1 in `light`, 1.23:1 in
`dark`** — against the `3.0:1` floor. Re-derived independently, swept
over the full greyscale content range (the same method
`worst_case_over_content_sweep` in `crates/snora-design/src/tests.rs`
already uses for the assertion below), across all four presets:

| Preset | `border ǀ dim` min | at content | `border ǀ dim` max | `dim ǀ fill` range |
|---|---|---|---|---|
| `light` | 1.00:1 | 0.98 | 6.21:1 | 3.24 – 21.00:1 |
| `dark` | 1.00:1 | 0.00 | 4.93:1 | 3.16 – 15.61:1 |
| `high_contrast_light` | 1.00:1 | 0.00 | 6.48:1 | 3.24 – 21.00:1 |
| `high_contrast_dark` | 1.00:1 | 1.00 | 4.94:1 | 4.01 – 19.80:1 |

**Every preset has a content luminance at which `border ǀ dim` reaches
1.00:1** — near-white content in `light`, black content in `dark` and
`high_contrast_light`, white content in `high_contrast_dark`. There is
no preset in which the border reliably outlines the card against the
dim. The high-contrast presets were checked, not assumed: an earlier
version of this finding guessed they would behave the *opposite* way
(pure black/white border, dim from the opposite pole, so highly
visible) — wrong. The same content luminance that erases the border in
`light`/`dark` erases it in the high-contrast presets too, just via the
opposite pole.

**What does carry the separation is the fill-to-dim step.** `dim ǀ
fill` never drops below **3.16:1**, in any preset, across the whole
content range — the one mechanism that always works.

**This is why `dialog_card_distinguishable_from_modal_dim_all_presets`
(RFC-066, `crates/snora-design/src/tests.rs`) asserts
`max(border ǀ dim, fill ǀ dim) ≥ NON_TEXT_MIN`, not two separate
assertions — and it is correct, exactly as written.** WCAG SC 1.4.11 is
an either-signal rule, and `max(...)` is the honest expression of that.
But across every preset and the whole content range it is the *same*
signal carrying it every time: **the fill branch, never the border
branch.** The assertion is not belt-and-braces between two working
mechanisms; it is one mechanism that works and one that does not apply
here, written as a single either-signal assertion because that is what
the rule it implements actually says.

**None of this weakens the border requirement.** The border-vs-plain-
`background` figures above and the border's contrast against the card's
own fill (3.38:1 `light`, 3.17:1 `dark`, 21.0:1 `high_contrast_light`,
19.80:1 `high_contrast_dark`) are both real, both required, and both
still clear their floor in every preset. The border does necessary
work. Separating the card from the *dimmed* page is simply not that
work — the fill does it instead.

### The sheet panel

The sheet also sits over the modal dim — `layout.sheet.is_some()`
triggers the same `has_modal` dim a dialog does
(`crates/snora/src/render.rs`) — and has its own border, unmeasured
until now (RFC-077 Q-2).

**It is not token-styled.** RFC-039 deliberately did not restyle the
sheet (see "What this RFC does not cover" below); its border and fill
come from `iced::Theme::extended_palette()`'s `background.weak` and
`background.base`, not from `snora-design`'s `border`/`surface_raised`
tokens. The figures below hold only when the application's
`iced::Theme` is `snora::design::theme::theme(&tokens)` (RFC-038); an
app using a different `iced::Theme` gets different, unmeasured numbers
from the ones stated here.

Swept the same way as the dialog card, using `extended_palette()`'s
`background.weak` as the sheet's border and `background.base` as its
fill:

| Preset | `border ǀ dim` min | `dim ǀ fill` min | own border ǀ own fill |
|---|---|---|---|
| `light` | 2.41:1 | 3.24:1 | 1.35:1 |
| `dark` | 2.95:1 | 3.80:1 | 1.29:1 |
| `high_contrast_light` | 2.41:1 | 3.24:1 | 1.35:1 |
| `high_contrast_dark` | 4.17:1 | 4.25:1 | 1.02:1 |

**Better against the dim than the dialog card's border, in every
preset** (minimum 2.41:1 against the dialog's 1.00:1) — but **worse
against its own fill**: the sheet's border-to-fill contrast is only
1.02–1.35:1, well under `NON_TEXT_MIN`, because `background.weak` is a
subtle iced-native separator, not a dedicated contrast-tested border
role the way `snora-design`'s `border` token is. This is a pre-existing
property of a surface RFC-039 chose not to restyle, not a regression
introduced here and not something this RFC changes. Stated because
"unmeasured" is not the same as "fine," and the answer is worth having
either way.

## What this RFC does not cover

Chrome geometry (header/sidebar/footer spacing and radius) is RFC-040.
The sheet is not restyled — it already has an `opaque()` wrapper and
edge-aware rounding and is not visually broken the way the dialog was.
`WARNING_COLOR` (toasts) is unchanged: toasts render on the
design-*inactive* path too, so changing them there would break the
gating invariant.
