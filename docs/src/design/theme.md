# Token-derived `iced::Theme`

`snora::design::theme(&Tokens) -> iced::Theme` derives a complete iced
theme from a Snora Design token bundle, so stock iced widgets
(`text_input`, `pick_list`, `scrollable`, …) and the window background
follow the same palette as snora's design primitives — instead of the
preset reaching only the primitives and the application maintaining a
second, separately configured `iced::Theme` by hand.

```rust,ignore
{{#include ../../../examples/book_snippets/src/theme.rs:theme_basic_usage}}
```

Snora never calls this function on the application's behalf — it returns
a value, and the application owns it. Nothing changes for applications
that don't call it.

## Why not `Theme::custom`

iced's default theme constructor, `Theme::custom`, routes every color pair
through `Pair::new(color, text)`, which *corrects* the text color —
lightening or darkening it in steps until it clears a heuristic
`relative_contrast >= 6.0` bar. Passing `snora-design`'s contrast-tested
roles through that would let iced silently replace them with its own
approximation, and the WCAG AA guarantee `snora-design`'s own contrast
tests establish would not carry over to the emitted theme.

`theme()` instead uses `Theme::custom_with_fn` with a generator that
builds every `Pair` as a struct literal — never `Pair::new` — directly
from a verified token role, or from a deterministic transform of one.

## Base tiers vs. derived tiers

Every iced `Extended` palette set (`Background`, and the `{ base, weak,
strong }` shape shared by `primary`/`secondary`/`success`/`warning`/
`danger`) has more tiers than `snora-design` has token variants —
`snora-design` carries exactly one color+text pair per semantic role, with
no separate weak/strong variants. An earlier revision of this function
resolved that by assigning the same pair to every tier. That satisfied
"every emitted color matches its source token role exactly" but broke
iced's own stock widgets: `button::primary` reads `primary.base` at rest
and `primary.strong` on hover
(`iced_widget-0.14.2/src/button.rs:600,605`), so a collapsed theme
silently removed all hover/pressed feedback from every stock button —
worse than iced's own default theme.

The corrected rule, and what `theme()` actually implements:

- **Base** tiers (`Background::base`, and `base` of every semantic set)
  equal their source token role **exactly** — struct literals built
  directly from a token color, never touched by `Pair::new`'s heuristic.
- **Derived** tiers (`weak`, `strong`, and `Background`'s seven non-base
  tiers) are computed from the base color by a deterministic transform,
  `shift_away_from`, and independently verified to meet the same contrast
  thresholds as `base`.

`shift_away_from(color, reference, amount)` lightens or darkens `color` by
`amount`, choosing the direction from a `reference` color's darkness. One
further edge case applies regardless of what `reference` is: a color
already at a luminance extreme (pure black or white) can't move further
in the contrast-increasing direction — `darken`/`lighten` clamp there —
so `shift_away_from` falls back to the opposite direction in that case,
which is safe precisely because a color already at an extreme has the
most contrast headroom to spare.

`reference` is not always `color` itself, because the two derivations
below need different notions of "away from" — and, after a round-2
correction, they deliberately use different ones:

- **Semantic sets** (`weak`/`strong` of `primary`, `secondary`, `success`,
  `warning`, `danger`) pass the tier's *fixed paired text* as `reference`
  — chosen because their contrast against that text is the property that
  matters. This makes a derived tier's contrast against its fixed text
  provably no worse than `base`'s: moving `color` further from the text's
  own tone can only widen the luminance gap, never narrow it (unlike
  iced's own `iced::theme::palette::deviate`, which picks a direction from
  `color`'s own darkness, independent of what it's paired with). Two
  simpler alternatives were tried first and rejected on measured contrast,
  not by inspection: mixing `weak` toward `background` (iced's own
  `Primary::generate` shape) degenerates to `base` unmodified whenever
  `background` happens to equal the seed color — both high-contrast
  presets deliberately have `surface == background` — and dropped one
  preset's contrast as low as 2.04:1 even where it didn't collapse;
  `deviate` alone held for most cases but still dropped one preset's
  `success.weak` to 4.40:1, just under the 4.5 AA floor. Text-relative
  direction removed both failures without per-preset tuning.
- **`Background`'s tiers** instead pass `color` as its own `reference` —
  matching `deviate`'s direction exactly. An earlier revision (round 2)
  used the text-relative rule here too, which was wrong: `Background`
  tiers are compared against *each other* (a border against its adjacent
  surface), not against text, so the text-relative guarantee that helps
  semantic sets doesn't apply — and in the `dark` preset it picked the
  wrong direction outright, darkening every tier from an already
  near-black background instead of lightening. Full story in the next
  section.

These two derivations are a **deliberately flagged inconsistency**, not
an oversight: a semantic tier's job is legibility of the text painted on
it, while a background tier's job is visibility against its own
neighbors, and the correct reference color differs accordingly.

The full derivation lives in `crates/snora-widgets/src/design/theme.rs`;
`crates/snora-widgets/src/design/theme/tests.rs` independently
re-computes each transform and asserts every derived tier matches it,
plus a dedicated fidelity/contrast/distinctness suite described below.

## The 18 → 6 mapping

iced's `Theme::palette()` reports a six-slot base `Palette`
(`background`, `text`, `primary`, `success`, `warning`, `danger`). This is
a **lossy view** — widgets don't read it directly. What widgets actually
read is `Theme::extended_palette()`, which `theme()` constructs in full
from the 18 `snora-design` roles.

### Base `Palette` (six slots)

| iced slot | Token role |
|---|---|
| `background` | `background` |
| `text` | `text_primary` |
| `primary` | `accent` |
| `success` | `success` |
| `warning` | `warning` |
| `danger` | `danger` |

### `Extended.background` (eight tiers)

`background.base` is `background` exactly. The other seven tiers are all
paired with `text_primary` — matching iced's own `Background::new(base,
text)` convention of one text color per gradient — but are derived two
different ways:

- **Six tiers** (`weakest`, `weaker`, `weak`, `neutral`, `stronger`,
  `strongest`) are `shift_away_from(background, background, amount)` at
  fixed, increasing amounts — the same magnitudes iced's own
  `Background::new` uses for its equivalent slots.
- **`strong`** — the one tier iced's stock widgets actually read as a
  border/separator color, not a text background
  (`iced_widget-0.14.2/src/button.rs:700,721`, `checkbox.rs:574,581`,
  `rule.rs:308`, `slider.rs:687`, and others) — is instead computed by
  growing the shift amount, starting from the same fixed amount the other
  tiers would use, until the result clears a `1.5:1` WCAG contrast floor
  against `background` itself, or the amount reaches `1.0`. For three of
  the four built-in presets the starting amount already clears the floor
  and this returns immediately — identical output to the fixed-amount
  approach. Only a preset whose background sits at a luminance extreme
  (`high_contrast_dark`'s pure black) needs the amount grown further: the
  same fixed delta that comfortably clears the floor for a near-black
  background (`dark`) produces an almost invisible border against a
  *literal* black one, because WCAG's contrast formula is highly
  nonlinear near the extremes — the same OKLCH-lightness step yields a
  far smaller contrast gain there than anywhere else on the scale. `1.5`
  is a deliberately modest floor, well below WCAG SC 1.4.11's full `3.0`
  non-text-contrast threshold for arbitrary UI component boundaries;
  reaching `3.0` uniformly would require growing the amount for every
  preset, not just the one that actually fails.

`surface` and `surface_raised` are **not** used for `Background` — an
earlier, round-1 mapping doubled/tripled those two roles across the eight
tiers, which meant no tier was actually derived, just relabeled. Deriving
every non-base tier from `background` itself keeps the gradient
consistent and lets it clear contrast independently of what `surface`
happens to be in a given preset.

### Why `strong`'s direction mattered (round-2 correction)

A round-2 revision derived all seven non-base tiers — including
`strong` — the same way the semantic sets are derived: direction from
`text_primary`'s darkness, not `background`'s own. That produced a real
defect in the `dark` preset: its background is dark-but-not-extreme with
light text, so the text-relative rule picked "darken" as the
contrast-increasing direction — every background tier darkened from an
already near-black background, and every stock button/checkbox/rule
border rendered as darker-than-its-surface, effectively invisible. The
existing contrast tests didn't catch it because they only check a tier's
color against its own paired *text*, not against the adjacent surface a
border is actually seen next to — a real coverage gap, closed by the
adjacent-surface test described in "Contrast and distinctness guarantees"
below.

Two independent signals confirmed the seed-relative direction (what
`theme()` now implements) is the correct one: `dark`'s own `border` token
(`rgb(0.169, 0.192, 0.227)`) is *lighter* than its `background` — the
token set's own design intent is "borders lighten in dark mode," the
opposite of what the text-relative rule produced — and `high_contrast_dark`
(pure-black background) only ever looked correct by accident, because its
extremity forced the shift's own clamp-fallback to fire regardless of
which reference color was driving the direction.

`border` was tried as an eighth, more-emphasized fill tier and reverted:
in the `high_contrast_light` preset it is bitwise identical to
`text_primary` (both pure black — `border` is a *stroke* color, not a
fill), so pairing it as a background emitted invisible black-on-black
text. The fidelity/contrast test suite caught this during development,
exactly as it is meant to; `border` is not represented in the emitted
theme at all.

### `Extended` semantic sets (`primary`, `success`, `warning`, `danger` — three tiers each)

`base` is the token pair exactly. `weak` and `strong` are
`shift_away_from(base_color, paired_text, amount)` at two increasing
amounts, with the text held fixed at the token's own paired foreground —
never re-derived, never passed through `Pair::new`.

| iced set | Token pair (`base`) |
|---|---|
| `primary` | `accent` / `accent_text` |
| `success` | `success` / `success_text` |
| `warning` | `warning` / `warning_text` |
| `danger` | `danger` / `danger_text` |

### `secondary` — the neutral family, not an accent

**`secondary` has no corresponding token role at all**, and unlike the
sets above it is not treated as an accent. iced's own theme derives it
that way: `Secondary::generate(palette.background, palette.text)`
(`iced_core-0.14.0/src/theme/palette.rs:403`) builds `Secondary` from
background and text as a **neutral, muted** set, deliberately not a
semantic color.

An earlier revision mapped `secondary` to `info`/`info_text` as "the
closest existing non-primary accent" — semantically reasonable by the
iced *name*, but wrong against iced's *derivation*: it would have
rendered every stock `button::secondary` in the info hue (typically blue)
instead of as neutral chrome. `theme()` instead derives `secondary` from
`surface`/`text_primary` — the same three-tier `shift_away_from`
treatment as the sets above, seeded from the neutral surface role rather
than an accent.

### What is not represented

`text_secondary`, `text_muted`, `border`, `focus`, and `surface_raised`
have no slot in the emitted theme. Applications needing these read
`tokens.palette` directly — trivial, since they already hold the
`Tokens` value used to build the theme.

### `is_dark`

`Tokens` carries no preset-identity field, and `theme()`'s signature is
fixed to `&Tokens` only, so `is_dark` is derived from the token data
itself via iced's own public `iced::theme::palette::is_dark` function on
`palette.background` — the same function `Extended::generate` uses
internally. This is correct for all four built-in presets but means an
application supplying a custom, mutated `Tokens` gets `is_dark` computed
from its background luminance rather than a hand-declared intent.

## Contrast and distinctness guarantees

Every `Pair` in the emitted `Extended` — base and derived — meets WCAG AA
(≥ 4.5:1) against its own paired text, for all four presets, and the two
high-contrast presets meet AAA (≥ 7.0:1) wherever the underlying tokens
already do. Within every semantic set, `base`, `weak`, and `strong` are
pairwise distinct for all four presets — the property whose absence broke
stock-widget hover/pressed feedback in the collapsed-tier design.

`background.strong` additionally meets a `1.5:1` contrast floor against
`background.base` itself, for all four presets — the adjacent-surface
comparison that matters for its role as a border color, which the
text-contrast checks above can't catch (see "Why `strong`'s direction
mattered" above).

All of this is verified by `crates/snora-widgets/src/design/theme/tests.rs`;
if a future token change or transform edit ever breaks one of these, the
tests fail loudly rather than shipping a regressed theme.
