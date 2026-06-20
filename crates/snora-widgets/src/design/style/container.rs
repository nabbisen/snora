//! Card and container style functions for Snora Design tokens.
//!
//! Three semantic variants:
//!
//! * [`card_surface`] — standard card on a `surface` background; the default
//!   card style.
//! * [`card_raised`] — elevated card on a `surface_raised` background with a
//!   drop shadow, for popovers or prominent panels.
//! * [`card_selected`] — card with an `accent`-colored border indicating
//!   selection.
//!
//! # iced 0.14 limitation
//!
//! `iced::widget::container` has **no interaction `Status`** in iced 0.14;
//! the style closure receives only `&Theme`. These functions therefore take
//! only `&Tokens`, not a status. Focus rings are not expressible here.

use iced::{Border, Color, Shadow, widget::container};
use snora_design::Tokens;

use super::color::to_iced_color;

/// Standard card on a `surface` background.
///
/// Use for most cards, panels, and content containers that sit on the
/// application background.
#[must_use]
pub fn card_surface(tokens: &Tokens) -> container::Style {
    container::Style {
        text_color: Some(to_iced_color(tokens.palette.text_primary)),
        background: Some(to_iced_color(tokens.palette.surface).into()),
        border: Border::default()
            .rounded(tokens.radius.lg)
            .color(to_iced_color(tokens.palette.border))
            .width(1.0),
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Elevated card on a `surface_raised` background with a soft drop shadow.
///
/// Use for popovers, floating panels, or cards that visually float above the
/// surrounding content.
#[must_use]
pub fn card_raised(tokens: &Tokens) -> container::Style {
    container::Style {
        text_color: Some(to_iced_color(tokens.palette.text_primary)),
        background: Some(to_iced_color(tokens.palette.surface_raised).into()),
        border: Border::default()
            .rounded(tokens.radius.lg)
            .color(to_iced_color(tokens.palette.border))
            .width(1.0),
        shadow: Shadow {
            color: Color { a: 0.12, ..Color::BLACK },
            offset: iced::Vector { x: 0.0, y: 2.0 },
            blur_radius: 8.0,
        },
        snap: true,
    }
}

/// Card with an `accent`-colored border indicating the selected/active state.
///
/// Use when multiple cards can be selected and one needs to be visually
/// distinguished as the active choice.
#[must_use]
pub fn card_selected(tokens: &Tokens) -> container::Style {
    container::Style {
        text_color: Some(to_iced_color(tokens.palette.text_primary)),
        background: Some(to_iced_color(tokens.palette.surface).into()),
        border: Border::default()
            .rounded(tokens.radius.lg)
            .color(to_iced_color(tokens.palette.accent))
            .width(2.0),
        shadow: Shadow::default(),
        snap: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snora_design::Tokens;

    fn all_presets() -> [Tokens; 4] {
        [
            Tokens::light(),
            Tokens::dark(),
            Tokens::high_contrast_light(),
            Tokens::high_contrast_dark(),
        ]
    }

    #[test]
    fn all_card_styles_produce_background_for_all_presets() {
        for t in all_presets() {
            assert!(card_surface(&t).background.is_some(), "card_surface no background");
            assert!(card_raised(&t).background.is_some(), "card_raised no background");
            assert!(card_selected(&t).background.is_some(), "card_selected no background");
        }
    }

    #[test]
    fn card_selected_has_wider_border_than_surface() {
        let t = Tokens::light();
        assert!(
            card_selected(&t).border.width > card_surface(&t).border.width,
            "card_selected border should be wider than card_surface"
        );
    }

    #[test]
    fn card_raised_has_shadow_offset() {
        let t = Tokens::dark();
        let s = card_raised(&t);
        assert!(s.shadow.blur_radius > 0.0, "card_raised should have a shadow");
    }
}
