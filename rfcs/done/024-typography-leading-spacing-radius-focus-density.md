# RFC 024 — Typography, Leading, Spacing, Radius, Focus, and Density

**Status.** Implemented (v0.19.0)
**Tracks.** Snora Design System migration; v0.20 foundation.
**Touches.** `snora-design` non-color tokens.

## Summary

This RFC defines non-color design tokens.

## Goals

- Provide readable typography roles.
- Include line-height from the start.
- Define compact spacing/radius/focus tokens.
- Define density policy without overcommitting.
- Keep tokens renderer-independent.

## Non-goals

- No font loading.
- No default font family.
- No OS text scale sync.
- No per-widget density recalculation.

## Typography

```rust
pub struct Typography {
    pub body: TextRole,
    pub body_small: TextRole,
    pub label: TextRole,
    pub title: TextRole,
    pub heading: TextRole,
    pub display: TextRole,
}

pub struct TextRole {
    pub size: f32,
    pub line_height: f32,
}
```

`line_height` is a multiplier. iced-specific conversion happens in `snora-widgets`.

## Role semantics

| Role | Purpose |
|---|---|
| body | ordinary readable text |
| body_small | compact help/metadata |
| label | button/chip/control labels |
| title | card/dialog/notice title |
| heading | page or section heading |
| display | rare major title |

## Spacing

```rust
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}
```

## Radius

```rust
pub struct Radius {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub pill: f32,
}
```

## Focus

```rust
pub struct FocusTokens {
    pub ring_width: f32,
    pub ring_offset: f32,
    pub ring_color: Color,
}
```

Focus must be visible across light, dark, and high-contrast presets.

Caveat: `FocusTokens` are design vocabulary. Whether a given iced 0.14 widget
can actually render a focus ring depends on the widget's styling surface; in
particular standard `button`/`container` styling does not expose focus state
(see RFC-025). The tokens are still defined now for the widgets/integration
paths that do, and for future iced versions.

## Density

```rust
pub enum Density {
    Comfortable,
    Compact,
}
```

v0.20 policy: the `Density` enum and the `Tokens::density` field exist, and
all built-in presets set `Density::Comfortable`. The following are
**deferred** (future-only, not part of the v0.20 API):

```rust
Tokens::with_density(...)   // future
Tokens::light_compact()     // future
Tokens::dark_compact()      // future
PalettePreset               // future selector enum (undefined in v0.20)
```

When density resolution is implemented later, resolve it during token
construction so widget helpers consume already-resolved values. Do not
calculate density inside widget helpers, in v0.20 or later.

## Data lifecycle

```text
app chooses preset + density
  -> token constructor resolves values
  -> app stores tokens
  -> widgets consume resolved tokens
```

## Visual fit process

Line-height and focus tokens require live review for:

- vertical clipping;
- off-center text;
- focus ring clipping;
- too-tight compact controls.

## Testing

- sizes > 0;
- line-height > 0;
- spacing finite and non-negative;
- radius finite and non-negative;
- focus width/offset finite and non-negative.

## Acceptance criteria

- Typography uses `TextRole`.
- Line-height is documented as multiplier.
- Focus tokens exist.
- Density policy is decided or deferred explicitly.
