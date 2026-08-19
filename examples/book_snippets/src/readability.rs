//! Anchored source for `docs/src/guides/readability.md`.

use snora::design::Tokens;

// ANCHOR: readability_applying_a_role
fn applying_a_role(tokens: Tokens) -> iced::widget::Text<'static> {
    iced::widget::text("wrapping prose")
        .size(snora::design::style::text::body_size(&tokens))
        .line_height(snora::design::style::text::body_line_height(&tokens))
}
// ANCHOR_END: readability_applying_a_role
