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
    pub density:    Density,    // *spacing* density — Comfortable; Compact deferred
}
```

`Tokens` and `Palette` are `#[non_exhaustive]` — new fields can be added in
future minor releases without breaking downstream code. All other token
sub-types (`Spacing`, `Radius`, `Typography`, `FocusTokens`) are
constructible by struct literal and are not marked `#[non_exhaustive]`.

This entire surface is under a contractual additive-only covenant — see
[Stability](stability.md) for what is and is not frozen.

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
