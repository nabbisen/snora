//! Anchored source for `docs/src/design/tokens.md`.
//!
//! The "Structure" fence (`pub struct Tokens { ... }`) is deliberately
//! **not** anchored here (review R-3, RFC-069 round 2): it is a diagram
//! of a type's shape, not code demonstrating usage, and `Tokens` is
//! `#[non_exhaustive]` so no real field-access guard can live in a
//! reader-facing snippet without either padding or losing the field
//! comments that are the whole point of the fence. A field-list guard
//! belongs in `snora-design`'s own test suite (the `Palette::usages()`
//! shape, RFC-063), as its own future RFC — not here.

use snora::design::{Color, Tokens};

// ANCHOR: tokens_picking_a_preset
fn picking_a_preset() {
    use snora::design::Tokens;

    let tokens = Tokens::light(); // calm, readable light theme
    let tokens = Tokens::dark(); // low-glare dark theme
    let tokens = Tokens::high_contrast_light(); // WCAG enhanced contrast (light)
    let tokens = Tokens::high_contrast_dark(); // WCAG enhanced contrast (dark)
}
// ANCHOR_END: tokens_picking_a_preset

// ANCHOR: tokens_customizing
fn customizing() {
    let mut tokens = Tokens::light();
    tokens.palette.accent = snora::design::Color::rgb(0.0, 0.5, 0.4);
    tokens.radius.md = 8.0;
}
// ANCHOR_END: tokens_customizing

// ANCHOR: tokens_reverify_contrast
fn reverify_contrast(my_text: Color, my_background: Color) {
    use snora_design::contrast::contrast_ratio;

    let ratio = contrast_ratio(my_text, my_background);
    assert!(ratio >= 4.5, "WCAG AA body text requires 4.5:1");
}
// ANCHOR_END: tokens_reverify_contrast

#[derive(Debug, Clone)]
enum Message {
    Save,
}

// ANCHOR: tokens_storing_in_state
struct App {
    tokens: snora::design::Tokens,
}

impl App {
    fn view(&self) -> iced::Element<'_, Message> {
        let t = &self.tokens; // no local token construction; no lifetime issues
        snora::design::button::primary(t, "Save", Message::Save)
    }
}
// ANCHOR_END: tokens_storing_in_state
