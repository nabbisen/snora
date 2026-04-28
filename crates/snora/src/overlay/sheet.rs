//! Edge-anchored sheet renderer.
//!
//! Resolves a [`Sheet`] (content + [`SheetEdge`] + [`SheetSize`]) to an
//! `iced::Element` placed against the chosen window edge, occupying the
//! configured size along the perpendicular axis.
//!
//! # Size resolution
//!
//! * Ratio-based sizes ([`SheetSize::OneThird`], `Half`, `TwoThirds`,
//!   `Ratio`) use `Length::FillPortion` — the spacer on the opposite side
//!   gets the remaining portion, the sheet gets the declared one.
//! * [`SheetSize::Pixels`] uses `Length::Fixed` for the sheet and
//!   `Length::Fill` for the spacer.
//!
//! # Corner radius
//!
//! Each anchor edge rounds only the *inside-facing* corners — the corners
//! that sit against the application content rather than against the
//! window edge. So a bottom-anchored sheet has its top corners rounded;
//! a start-anchored sheet (LTR=left) has its right corners rounded; etc.
//! Direction-aware: `SheetEdge::Start` / `End` resolve through the
//! [`LayoutDirection`].

use iced::{
    Element, Length,
    border::Radius,
    widget::{column, container, opaque, row, space},
};
use snora_core::{LayoutDirection, Sheet, SheetEdge, SheetSize};

pub(crate) fn render_sheet<'a, Message>(
    sheet: Sheet<Element<'a, Message>, Message>,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let edge = sheet.edge;
    let size = sheet.size;

    // Sheet content must capture its own events; otherwise clicks inside
    // the sheet fall through to the modal backdrop and dismiss it.
    let body_surface = container(sheet.content)
        .style(move |theme: &iced::Theme| sheet_surface_style(theme, edge, direction))
        .width(Length::Fill)
        .height(Length::Fill);
    let body = opaque(body_surface);

    if edge.is_vertical() {
        render_vertical(body, edge, size)
    } else {
        render_horizontal(body, edge, size, direction)
    }
}

/// Layout for top- or bottom-anchored sheets.
fn render_vertical<'a, Message>(
    body: Element<'a, Message>,
    edge: SheetEdge,
    size: SheetSize,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let (top_cell, bottom_cell) = match (edge, size) {
        // Pixel-sized sheets: fixed sheet height, spacer fills the rest.
        (SheetEdge::Bottom, SheetSize::Pixels(px)) => (
            Element::from(space().height(Length::Fill)),
            Element::from(container(body).width(Length::Fill).height(Length::Fixed(px))),
        ),
        (SheetEdge::Top, SheetSize::Pixels(px)) => (
            Element::from(container(body).width(Length::Fill).height(Length::Fixed(px))),
            Element::from(space().height(Length::Fill)),
        ),
        // Ratio-sized sheets: portions split the available height.
        (SheetEdge::Bottom, _) => {
            let ratio = size.as_ratio().unwrap_or(1.0 / 3.0);
            let (spacer_pct, sheet_pct) = portions(ratio);
            (
                Element::from(space().height(Length::FillPortion(spacer_pct))),
                Element::from(
                    container(body)
                        .width(Length::Fill)
                        .height(Length::FillPortion(sheet_pct)),
                ),
            )
        }
        (SheetEdge::Top, _) => {
            let ratio = size.as_ratio().unwrap_or(1.0 / 3.0);
            let (sheet_pct, spacer_pct) = (
                (ratio * 100.0).round().clamp(1.0, 100.0) as u16,
                {
                    let s = (ratio * 100.0).round().clamp(0.0, 100.0) as u16;
                    100u16.saturating_sub(s).max(1)
                },
            );
            (
                Element::from(
                    container(body)
                        .width(Length::Fill)
                        .height(Length::FillPortion(sheet_pct)),
                ),
                Element::from(space().height(Length::FillPortion(spacer_pct))),
            )
        }
        // Unreachable: caller filtered to vertical edges.
        _ => unreachable!("render_vertical called with horizontal edge"),
    };

    column![top_cell, bottom_cell]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Layout for start- or end-anchored sheets.
fn render_horizontal<'a, Message>(
    body: Element<'a, Message>,
    edge: SheetEdge,
    size: SheetSize,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    // Resolve logical edge to physical (left vs right) so we know which
    // side of the row the sheet sits on.
    let on_left = match (edge, direction) {
        (SheetEdge::Start, LayoutDirection::Ltr) => true,
        (SheetEdge::Start, LayoutDirection::Rtl) => false,
        (SheetEdge::End, LayoutDirection::Ltr) => false,
        (SheetEdge::End, LayoutDirection::Rtl) => true,
        _ => unreachable!("render_horizontal called with vertical edge"),
    };

    let (left_cell, right_cell) = match size {
        SheetSize::Pixels(px) => {
            let sheet_cell = container(body).width(Length::Fixed(px)).height(Length::Fill);
            let spacer_cell = space().width(Length::Fill);
            if on_left {
                (
                    Element::from(sheet_cell),
                    Element::from(spacer_cell),
                )
            } else {
                (
                    Element::from(spacer_cell),
                    Element::from(sheet_cell),
                )
            }
        }
        _ => {
            let ratio = size.as_ratio().unwrap_or(1.0 / 3.0);
            let (sheet_pct, spacer_pct) = if on_left {
                let (a, b) = portions(ratio);
                // portions returns (spacer, sheet); for left-anchored we
                // want sheet first, so swap.
                (b, a)
            } else {
                portions(ratio)
            };
            // Always put the sheet on the chosen side.
            if on_left {
                (
                    Element::from(
                        container(body)
                            .width(Length::FillPortion(sheet_pct))
                            .height(Length::Fill),
                    ),
                    Element::from(space().width(Length::FillPortion(spacer_pct))),
                )
            } else {
                (
                    Element::from(space().width(Length::FillPortion(spacer_pct))),
                    Element::from(
                        container(body)
                            .width(Length::FillPortion(sheet_pct))
                            .height(Length::Fill),
                    ),
                )
            }
        }
    };

    row![left_cell, right_cell]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Split a 0..1 ratio into (spacer_portion, sheet_portion) as `u16`s for
/// `Length::FillPortion`. Uses a 100-unit base so rounding is intuitive
/// (e.g. 1/3 → 67 spacer / 33 sheet).
///
/// Both returned portions are at least 1 to avoid the (0, 0) panic in iced.
fn portions(sheet_ratio: f32) -> (u16, u16) {
    let sheet = (sheet_ratio * 100.0).round().clamp(0.0, 100.0) as u16;
    let spacer = 100u16.saturating_sub(sheet);
    if sheet == 0 {
        (100, 1)
    } else if spacer == 0 {
        (1, 100)
    } else {
        (spacer, sheet)
    }
}

/// Theme-aware surface style for the sheet body. Rounds the corners that
/// face the application content; leaves the corners against the window
/// edge square.
fn sheet_surface_style(
    theme: &iced::Theme,
    edge: SheetEdge,
    direction: LayoutDirection,
) -> iced::widget::container::Style {
    use iced::{Background, Border, widget::container::Style};

    let palette = theme.extended_palette();
    let r = 12.0;
    let radius = match edge {
        SheetEdge::Bottom => Radius {
            top_left: r,
            top_right: r,
            bottom_left: 0.0,
            bottom_right: 0.0,
        },
        SheetEdge::Top => Radius {
            top_left: 0.0,
            top_right: 0.0,
            bottom_left: r,
            bottom_right: r,
        },
        // Start (LTR=left): inside-facing corners are top-right + bottom-right.
        // Start (RTL=right): inside-facing corners are top-left + bottom-left.
        SheetEdge::Start => match direction {
            LayoutDirection::Ltr => Radius {
                top_left: 0.0,
                top_right: r,
                bottom_left: 0.0,
                bottom_right: r,
            },
            LayoutDirection::Rtl => Radius {
                top_left: r,
                top_right: 0.0,
                bottom_left: r,
                bottom_right: 0.0,
            },
        },
        SheetEdge::End => match direction {
            LayoutDirection::Ltr => Radius {
                top_left: r,
                top_right: 0.0,
                bottom_left: r,
                bottom_right: 0.0,
            },
            LayoutDirection::Rtl => Radius {
                top_left: 0.0,
                top_right: r,
                bottom_left: 0.0,
                bottom_right: r,
            },
        },
    };

    Style {
        background: Some(Background::Color(palette.background.base.color)),
        text_color: Some(palette.background.base.text),
        border: Border {
            radius,
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
        let (a, b) = portions(0.0);
        assert!(a > 0 && b > 0);
        let (a, b) = portions(1.0);
        assert!(a > 0 && b > 0);
    }
}
