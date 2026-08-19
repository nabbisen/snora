//! Anchored source for `docs/src/design/typography.md`.
//!
//! The "six roles" fence (`pub struct Typography { ... }` /
//! `pub struct TextRole { ... }`) is deliberately **not** anchored here
//! (review R-3, RFC-069 round 2): it is a diagram of two types' shapes,
//! and a destructuring stand-in removed the field *types* from the one
//! block whose job is showing them. A field-list guard belongs in
//! `snora-design`'s own test suite, as its own future RFC — not here.

use snora::design::Tokens;

// ANCHOR: typography_applying_a_role
fn applying_a_role(tokens: Tokens) -> iced::widget::Text<'static> {
    iced::widget::text("wrapping prose")
        .size(snora::design::style::text::body_size(&tokens))
        .line_height(snora::design::style::text::body_line_height(&tokens))
}
// ANCHOR_END: typography_applying_a_role
