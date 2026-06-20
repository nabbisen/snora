# RFC 022 — Snora Design Token Data Model

**Status.** Implemented (v0.19.0)
**Tracks.** Snora Design System migration; v0.20 foundation.
**Touches.** `snora-design` token modules.

## Summary

This RFC defines the abstract token data model for `snora-design`.

Tokens are plain data. They are not renderer behavior.

## Goals

- Define compact design vocabulary.
- Keep tokens iced-free.
- Support light/dark/high-contrast presets.
- Support typography with line-height.
- Support future style helpers and shallow primitives.

## Non-goals

- No iced style types.
- No font loading.
- No OS theme detection.
- No global theme registry.
- No CSS-like cascade.

## Data model

### Tokens

`Tokens` and `Palette` are `#[non_exhaustive]` so future roles/fields (the
optional palette roles below, or a new token group) can be added without a
breaking change — consistent with `AppLayout` being `#[non_exhaustive]`.
Apps obtain `Tokens` via constructors and may still mutate public fields
(`tokens.palette.accent = ...`); `#[non_exhaustive]` only blocks struct
literals and exhaustive matches. The small value types (`Color`, `Spacing`,
`Radius`, `TextRole`, `FocusTokens`) are deliberately left open so apps can
build them by literal.

```rust
#[non_exhaustive]
pub struct Tokens {
    pub palette: Palette,
    pub spacing: Spacing,
    pub typography: Typography,
    pub radius: Radius,
    pub focus: FocusTokens,
    pub density: Density,
}
```

Required constructors:

```rust
impl Tokens {
    pub fn light() -> Self;
    pub fn dark() -> Self;
    pub fn high_contrast_light() -> Self;
    pub fn high_contrast_dark() -> Self;
}
```

Recommended traits:

```rust
Clone
Debug
PartialEq
```

### Color

```rust
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
```

Constructors:

```rust
impl Color {
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self;
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self;
}
```

### Palette

```rust
#[non_exhaustive]
pub struct Palette {
    pub background: Color,
    pub surface: Color,
    pub surface_raised: Color,

    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,

    pub border: Color,
    pub accent: Color,
    pub accent_text: Color,

    // Status backgrounds and their paired foreground (on-status) text.
    // The paired text roles exist so status surfaces — starting with the
    // v0.20 danger button — have a contrast-tested foreground rather than
    // borrowing accent_text. (See RFC-023 contrast pairs.)
    pub success: Color,
    pub success_text: Color,
    pub warning: Color,
    pub warning_text: Color,
    pub danger: Color,
    pub danger_text: Color,
    pub info: Color,
    pub info_text: Color,

    pub focus: Color,
}
```

`text_muted` is intentionally lower-contrast and is for non-essential text;
it is exempt from the mandatory body-text contrast pairs (documented in
RFC-023).

### Typography

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

`line_height` is a renderer-independent multiplier.

### Spacing

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

### Radius

```rust
pub struct Radius {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub pill: f32,
}
```

### Focus

```rust
pub struct FocusTokens {
    pub ring_width: f32,
    pub ring_offset: f32,
    pub ring_color: Color,
}
```

### Variants

```rust
pub enum Tone {
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
    Info,
}

pub enum Emphasis {
    Solid,
    Soft,
    Outline,
    Ghost,
}

pub enum Size {
    Small,
    Medium,
    Large,
}

pub enum Density {
    Comfortable,
    Compact,
}
```

Density policy for v0.20: the `Density` enum and the `Tokens::density` field
exist, and all built-in presets set `Density::Comfortable`. The compact
constructors, `Tokens::with_density`, and any `PalettePreset` selector are
deferred (see RFC-024); no widget helper performs density scaling in v0.20.

## Data lifecycle

1. Application chooses a token preset.
2. Application may customize tokens.
3. Application stores tokens in app state.
4. View passes tokens to style helpers/primitives.
5. If user changes light/dark/high-contrast mode, app replaces tokens.

Snora does not own token state.

## Internal module layout

```text
snora-design/src/
  lib.rs
  color.rs
  palette.rs
  tokens.rs
  spacing.rs
  typography.rs
  radius.rs
  focus.rs
  variants.rs
  presets/
    light.rs
    dark.rs
    high_contrast_light.rs
    high_contrast_dark.rs
```

## Testing

- all constructors produce valid tokens;
- no color channel is NaN/infinite/out of range;
- spacing/radius/focus values are sane;
- line-height values are positive;
- presets satisfy RFC-023 contrast tests.

## Acceptance criteria

- `snora-design` compiles without iced.
- Token constructors exist.
- Token tests pass.
- Token API is compact and documented.
