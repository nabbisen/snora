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
    pub density:    Density,    // Comfortable (Compact deferred)
}
```

`Tokens` and `Palette` are `#[non_exhaustive]` — new fields can be added in
future minor releases without breaking downstream code. All other token
sub-types (`Spacing`, `Radius`, `Typography`, `FocusTokens`) are
constructible by struct literal and are not marked `#[non_exhaustive]`.

## Picking a preset

```rust,ignore
use snora::design::Tokens;

let tokens = Tokens::light();               // calm, readable light theme
let tokens = Tokens::dark();                // low-glare dark theme
let tokens = Tokens::high_contrast_light(); // WCAG enhanced contrast (light)
let tokens = Tokens::high_contrast_dark();  // WCAG enhanced contrast (dark)
```

## Customizing

Fields are `pub` — mutate what you need after cloning a preset:

```rust,ignore
let mut tokens = Tokens::light();
tokens.palette.accent = snora::design::Color::rgb(0.0, 0.5, 0.4);
tokens.radius.md = 8.0;
```

When you customize a color, re-verify contrast for affected pairs using the
`snora_design::contrast` module:

```rust,ignore
use snora::design::Color;
use snora_design::contrast::contrast_ratio;

let ratio = contrast_ratio(my_text, my_background);
assert!(ratio >= 4.5, "WCAG AA body text requires 4.5:1");
```

## Storing tokens in application state

Because `view()` borrows from `&self`, store `Tokens` in your state struct
rather than constructing them inline in `view()`:

```rust,ignore
struct App {
    tokens: snora::design::Tokens,
}

impl App {
    fn view(&self) -> Element<'_, Message> {
        let t = &self.tokens;   // no local token construction; no lifetime issues
        snora::design::button::primary(t, "Save", Message::Save)
    }
}
```
