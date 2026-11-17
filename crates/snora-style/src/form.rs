//! The look of a form control's row: the checkbox, the toggle, the radio group, the slider and
//! the select's trigger (RFC-134).
//!
//! Each of the five is one `snora_interaction::Control` whose content is its mark and its
//! label. This is the style of that control: **nothing at rest** (the page shows through, so the
//! control sits on `background`, `surface` or `surface_raised` alike), and **a tint of the label's
//! own colour** when hovered or pressed. The marks (the box, the switch, the dots, the bar and the
//! thumb) are drawn by the widgets from the palette's `border`, `accent` and their pairs.
//!
//! # Hovered and pressed (RFC-129)
//!
//! The label is held at the palette's [`text_contrast`](snora_design::Palette::text_contrast) on
//! every neutral surface, and the state is visibly different from rest (ΔE76 of at least 3), by the
//! rule the buttons use: the tint where it holds, another shift of the label where it does not.
//! Disabled is exempt.

use iced::{Border, Color, Shadow, widget::button};
use snora_design::Tokens;

use super::color::to_iced_color;
use super::state::{Knob, Pair, shifted};

/// The row's hovered or pressed look: `text_primary` at `alpha` behind the label, and the label
/// shifted the way the buttons' is.
fn tint_in_state(tokens: &Tokens, ink: Color, alpha: f32, today: f32) -> Pair {
    let rest = Pair {
        label: ink,
        fill: Color::TRANSPARENT,
    };
    let state = Pair {
        label: ink,
        fill: Color { a: alpha, ..ink },
    };
    shifted(&tokens.palette, Knob::Label, today, rest, state)
}

/// The style of a form control's row, for one status.
///
/// ```rust,no_run
/// use snora_design::Tokens;
/// use snora_style::form;
///
/// let tokens = Tokens::light();
/// let _style = form::row(&tokens, iced::widget::button::Status::Hovered);
/// ```
#[must_use]
pub fn row(tokens: &Tokens, status: button::Status) -> button::Style {
    let ink = to_iced_color(tokens.palette.text_primary);
    let radius = iced::border::Radius::from(tokens.radius.md);
    let (background, text_color) = match status {
        button::Status::Active => (Color::TRANSPARENT, ink),
        button::Status::Hovered => {
            let p = tint_in_state(tokens, ink, 0.08, 0.0);
            (p.fill, p.label)
        }
        button::Status::Pressed => {
            let p = tint_in_state(tokens, ink, 0.14, -0.06);
            (p.fill, p.label)
        }
        button::Status::Disabled => (Color::TRANSPARENT, super::button::disabled_alpha(ink)),
    };
    button::Style {
        background: Some(background.into()),
        text_color,
        border: Border::default().rounded(radius),
        shadow: Shadow::default(),
        snap: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snora_design::{Color as DC, contrast::contrast_ratio};

    fn over(fill: Color, surface: DC) -> DC {
        let a = fill.a;
        DC::rgb(
            fill.r * a + surface.r * (1.0 - a),
            fill.g * a + surface.g * (1.0 - a),
            fill.b * a + surface.b * (1.0 - a),
        )
    }

    /// The label never sits under the palette's level, in any state but disabled, on any surface
    /// (RFC-129), for every preset.
    #[test]
    fn the_label_holds_its_level_hovered_and_pressed_on_every_surface() {
        let mut checked = 0;
        for (name, tokens) in Tokens::presets() {
            let level = tokens.palette.text_contrast.ratio();
            for status in [
                button::Status::Active,
                button::Status::Hovered,
                button::Status::Pressed,
            ] {
                let style = row(&tokens, status);
                let Some(iced::Background::Color(fill)) = style.background else {
                    panic!("{name}: no fill");
                };
                let label = DC::rgb(style.text_color.r, style.text_color.g, style.text_color.b);
                for surface in [
                    tokens.palette.background,
                    tokens.palette.surface,
                    tokens.palette.surface_raised,
                ] {
                    let ratio = contrast_ratio(label, over(fill, surface));
                    assert!(
                        ratio >= level - 0.02,
                        "{name} {status:?}: label {ratio:.2}:1 under {level}:1"
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked >= 54, "only {checked} readings");
    }

    /// Disabled is not a stop, and draws no fill.
    #[test]
    fn a_disabled_row_has_no_fill_and_a_dimmed_label() {
        let tokens = Tokens::light();
        let style = row(&tokens, button::Status::Disabled);
        assert_eq!(style.background, Some(Color::TRANSPARENT.into()));
        assert!(style.text_color.a < 1.0);
    }
}
