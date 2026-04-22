//! Bottom sheet — a drawer that slides up from the bottom of the window.
//!
//! The sheet obeys its declared [`SheetHeight`]. The dim backdrop and its
//! click-to-close behavior are installed by [`crate::render::render`], so
//! this renderer only draws the sheet surface itself.
//!
//! # Height resolution
//!
//! * Ratio-based heights ([`SheetHeight::OneThird`], `Half`, `TwoThirds`,
//!   `Ratio`) are implemented with `Length::FillPortion` — the top spacer
//!   gets the remaining portion, the sheet gets the declared one.
//! * [`SheetHeight::Pixels`] uses `Length::Fixed` for the sheet and
//!   `Length::Fill` for the top spacer, so the sheet is pinned to the
//!   bottom at its requested pixel height.

use iced::{
    Element, Length,
    widget::{column, container, opaque, space},
};
use snora_core::{BottomSheet, SheetHeight};

pub(crate) fn render_bottom_sheet<'a, Message>(
    sheet: BottomSheet<Element<'a, Message>, Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    // Sheet content must capture its own events — otherwise clicks inside
    // the sheet fall through to the modal backdrop and dismiss the sheet.
    let body_surface = container(sheet.content)
        .style(sheet_surface_style)
        .width(Length::Fill);
    let body = opaque(body_surface);

    let (top_spacer, sheet_cell) = match sheet.height {
        SheetHeight::Pixels(px) => {
            let top = space().height(Length::Fill);
            let cell = container(body).width(Length::Fill).height(Length::Fixed(px));
            (
                Element::<'a, Message>::from(top),
                Element::<'a, Message>::from(cell),
            )
        }
        other => {
            // Ratio path. `as_ratio` always returns Some here.
            let ratio = other.as_ratio().unwrap_or(1.0 / 3.0).clamp(0.0, 1.0);
            let (top_pct, sheet_pct) = portions(ratio);
            let top = space().height(Length::FillPortion(top_pct));
            let cell = container(body)
                .width(Length::Fill)
                .height(Length::FillPortion(sheet_pct));
            (
                Element::<'a, Message>::from(top),
                Element::<'a, Message>::from(cell),
            )
        }
    };

    column![top_spacer, sheet_cell]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Split a 0..1 ratio into (top_portion, sheet_portion) as `u16`s for
/// `Length::FillPortion`. Uses a 100-unit base so rounding is intuitive
/// (e.g. 1/3 → 67 top / 33 sheet).
fn portions(sheet_ratio: f32) -> (u16, u16) {
    let sheet = (sheet_ratio * 100.0).round().clamp(0.0, 100.0) as u16;
    let top = 100u16.saturating_sub(sheet);
    // Guard against (0, 0) which would make iced panic.
    if sheet == 0 { (100, 1) } else if top == 0 { (1, 100) } else { (top, sheet) }
}

/// The sheet's own surface styling — background pulled from the theme so
/// light / dark themes both look right.
fn sheet_surface_style(theme: &iced::Theme) -> iced::widget::container::Style {
    use iced::{Background, Border, border::Radius, widget::container::Style};

    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.background.base.color)),
        text_color: Some(palette.background.base.text),
        border: Border {
            radius: Radius {
                top_left: 12.0,
                top_right: 12.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            width: 1.0,
            color: palette.background.weak.color,
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portions_handle_edges() {
        assert_eq!(portions(0.5), (50, 50));
        assert_eq!(portions(1.0 / 3.0), (67, 33));
        assert_eq!(portions(2.0 / 3.0), (33, 67));
        // Clamp at endpoints.
        let (t, s) = portions(0.0);
        assert!(t > 0 && s > 0);
        let (t, s) = portions(1.0);
        assert!(t > 0 && s > 0);
    }
}
