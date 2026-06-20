# RFC 023 — Palettes, High Contrast, and Automated Contrast Tests

**Status.** Implemented (v0.19.0)
**Tracks.** Snora Design System migration; v0.20 foundation.
**Touches.** `snora-design` presets and contrast tests.

## Summary

This RFC defines palette presets and mandatory automated contrast tests.

## Motivation

Snora Design must provide accessible-oriented visual defaults. Since palette colors are data, contrast can be tested without rendering.

## Goals

- Provide light and dark presets.
- Provide high-contrast light and high-contrast dark presets.
- Require automated contrast tests.
- Use linearized sRGB relative luminance.
- Document thresholds and exceptions.

## Non-goals

- No OS high-contrast detection in v0.20.
- No screenshot testing.
- No automatic correction of custom app tokens.

## Data model

Preset constructors:

```rust
impl Tokens {
    pub fn light() -> Self;
    pub fn dark() -> Self;
    pub fn high_contrast_light() -> Self;
    pub fn high_contrast_dark() -> Self;
}
```

## Palette semantics

Light should be calm and readable.

Dark should reduce glare without becoming low-contrast.

High-contrast presets prioritize legibility, border clarity, and focus visibility over subtle aesthetics.

## Contrast calculation

Tests must use linearized sRGB.

```rust
fn linearize_srgb_channel(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
```

Relative luminance:

```rust
fn relative_luminance(color: Color) -> f32 {
    let r = linearize_srgb_channel(color.r);
    let g = linearize_srgb_channel(color.g);
    let b = linearize_srgb_channel(color.b);

    0.2126 * r + 0.7152 * g + 0.0722 * b
}
```

Contrast ratio:

```rust
fn contrast_ratio(a: Color, b: Color) -> f32 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);

    let bright = la.max(lb);
    let dark = la.min(lb);

    (bright + 0.05) / (dark + 0.05)
}
```

## Color opacity rule

Relative luminance and the contrast ratio are computed from RGB only. That
is correct only when both colors are fully opaque. Therefore:

> For every mandatory contrast pair, both the foreground and background
> colors in a built-in preset must be fully opaque (`a == 1.0`). If a role
> legitimately uses alpha (e.g. a translucent `border` or `focus`), the test
> must first composite it over the tested background and compute luminance on
> the composited result.

v0.20 takes the simpler path: built-in preset roles used in mandatory pairs
are opaque. A `composite_over(fg, bg)` helper is the future escape hatch if
alpha-based roles are introduced.

## Required pairs

At minimum:

- `text_primary` on `background`
- `text_primary` on `surface`
- `text_primary` on `surface_raised`
- `text_secondary` on `background`
- `text_secondary` on `surface`
- `accent_text` on `accent`
- `focus` on `background`
- `focus` on `surface`

Status foreground/background pairs (the status text roles ship in the v0.20
palette per RFC-022). `danger_text on danger` is **mandatory** in v0.20
because the danger button is a v0.20 pilot primitive; the other three are
required once their buttons/primitives ship and are recommended now:

- `danger_text` on `danger`   (mandatory)
- `success_text` on `success`
- `warning_text` on `warning`
- `info_text` on `info`

`text_muted` is intentionally low-contrast for non-essential text and is
exempt from the mandatory body-text pairs; document this exemption.

## Process handling

Any PR changing built-in palette values must pass contrast tests.

Any exception must be named and justified.

## Data lifecycle

```text
preset constructor
  -> token sanity test
  -> contrast test
  -> style helper usage
  -> example visual review
```

## Testing

- all four presets tested;
- all required pairs tested;
- tests run without iced;
- tests document thresholds.

## Documentation

Docs must state:

- high-contrast presets are explicit presets;
- automatic OS high-contrast detection is deferred;
- built-in presets are tested;
- custom app tokens are app responsibility.

## Acceptance criteria

- Four presets exist.
- Pure Rust contrast tests exist.
- Tests use linearized sRGB.
- Mandatory-pair colors are opaque (or composited before luminance).
- `danger_text on danger` is tested for all presets.
- Required pairs pass or documented exceptions exist.
