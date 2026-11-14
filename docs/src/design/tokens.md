# Design tokens

A `Tokens` struct is the top-level bundle of design decisions. It groups
spacing, typography, radius, focus, and a semantic color palette into one
value an application stores and passes to helpers.

## Structure

```rust,ignore
pub struct Tokens {             // #[non_exhaustive]
    pub palette:    Palette,    // semantic color roles
    pub spacing:    Spacing,    // xs / sm / md / lg / xl / xxl
    pub typography: Typography, // body / title / heading / …
    pub radius:     Radius,     // sm / md / lg / pill
    pub focus:      FocusTokens,// ring_width / ring_offset / ring_color
    pub targets:    TargetTokens,// the minimum pointer target (0.55, RFC-115)
    pub density:    Density,    // *spacing* density — Comfortable; Compact deferred
}
```

`Tokens` and `Palette` are `#[non_exhaustive]` — new fields can be added in
future minor releases without breaking downstream code. All other token
sub-types (`Spacing`, `Radius`, `Typography`, `FocusTokens`) are
constructible by struct literal and are not marked `#[non_exhaustive]`.

This entire surface is under a contractual additive-only covenant — see
[Stability](stability.md) for what is and is not frozen.

## The palette's text level (0.62)

`Palette::text_contrast` is the contrast level the palette's text is held to:
`TextContrast::Aa` (4.5:1) or `TextContrast::Aaa` (7:1), WCAG 1.4.3 and 1.4.6
(RFC-129).

| Preset | `text_contrast` |
|---|---|
| `light`, `dark` | `Aa` |
| `light_aaa`, `dark_aaa`, `high_contrast_light`, `high_contrast_dark` | `Aaa` |

- **It is the level snora holds every hovered and pressed label to.** A state's shift is
  chosen so that the label stays at that level on every neutral surface.
- **It is not a certificate of the resting colours.** Change `accent` on `light_aaa` and the
  field still says `Aaa`: snora's states aim at 7:1, and the resting accent is what you
  chose. Lowering it is your explicit act: `palette.text_contrast = TextContrast::Aa`.
- **It lives on the palette,** so `tokens.palette = Tokens::light_aaa().palette` brings the
  level with the colours.
- **There is no `Default`.** A palette starts from a preset, which sets the level of its own
  colours.

## Picking a preset

```rust,ignore
{{#include ../../../examples/book_snippets/src/tokens.rs:tokens_picking_a_preset}}
```

## Customizing

Fields are `pub` — mutate what you need after cloning a preset:

```rust,ignore
{{#include ../../../examples/book_snippets/src/tokens.rs:tokens_customizing}}
```

When you customize a color, re-verify contrast for affected pairs using the
`snora_design::contrast` module:

```rust,ignore
{{#include ../../../examples/book_snippets/src/tokens.rs:tokens_reverify_contrast}}
```

## Storing tokens in application state

Because `view()` borrows from `&self`, store `Tokens` in your state struct
rather than constructing them inline in `view()`:

```rust,ignore
{{#include ../../../examples/book_snippets/src/tokens.rs:tokens_storing_in_state}}
```
