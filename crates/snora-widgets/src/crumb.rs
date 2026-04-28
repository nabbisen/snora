//! Breadcrumb trail — a horizontal sequence of ancestor links plus
//! the current page as a non-clickable leaf.
//!
//! Layout (logical, ABDD):
//!
//! ```text
//!  Home › Library › Books › The Hobbit
//!  └────┘ └───────┘ └────┘  └────────┘
//!  ancestor ancestor ancestor   leaf
//!  (clickable)                   (plain text)
//! ```
//!
//! Under [`LayoutDirection::Rtl`] the order is mirrored as a whole and
//! the separator glyph flips (`›` → `‹`).

use std::fmt::Debug;

use iced::{
    Alignment::Center,
    Color, Element, Length, Padding, Theme,
    widget::{button, container, row, text},
};

use snora_core::{BreadcrumbAction, Crumb, LayoutDirection};

/// Build a breadcrumb trail.
///
/// * `crumbs` — the ordered sequence from root to leaf. The application
///   is responsible for marking exactly one entry as the leaf
///   ([`Crumb::leaf`]); ancestor entries ([`Crumb::ancestor`]) emit a
///   [`BreadcrumbAction::Pressed`] when clicked.
/// * `on_action` — maps [`BreadcrumbAction`] into your message type.
/// * `direction` — application's reading direction. Determines the
///   visual order *and* the separator glyph.
///
/// Empty crumb lists are valid and render as an empty row — no special
/// case for "no breadcrumb".
pub fn app_breadcrumb<'a, Message, CrumbId, F>(
    crumbs: Vec<Crumb<CrumbId>>,
    on_action: &'a F,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    CrumbId: Clone + Debug + 'a,
    F: Fn(BreadcrumbAction<CrumbId>) -> Message + 'a,
{
    let separator = match direction {
        LayoutDirection::Ltr => "›",
        LayoutDirection::Rtl => "‹",
    };

    let crumbs: Vec<_> = match direction {
        LayoutDirection::Ltr => crumbs,
        LayoutDirection::Rtl => crumbs.into_iter().rev().collect(),
    };

    let mut trail = row![].spacing(6).align_y(Center);

    let last = crumbs.len().saturating_sub(1);
    for (i, crumb) in crumbs.into_iter().enumerate() {
        trail = trail.push(render_crumb(crumb, on_action));
        if i < last {
            trail = trail.push(
                text(separator)
                    .size(13)
                    .style(|theme: &Theme| iced::widget::text::Style {
                        color: Some(separator_color(theme)),
                    }),
            );
        }
    }

    container(trail)
        .width(Length::Fill)
        .padding(Padding::from([4.0, 12.0]))
        .into()
}

fn render_crumb<'a, Message, CrumbId, F>(
    crumb: Crumb<CrumbId>,
    on_action: &'a F,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    CrumbId: Clone + Debug + 'a,
    F: Fn(BreadcrumbAction<CrumbId>) -> Message + 'a,
{
    if crumb.is_leaf {
        // Plain text — the user is here.
        text(crumb.label).size(13).into()
    } else {
        let id_for_msg = crumb.id.clone();
        button(text(crumb.label).size(13))
            .on_press_with(move || on_action(BreadcrumbAction::Pressed(id_for_msg.clone())))
            .padding(Padding::from([2.0, 4.0]))
            .style(crumb_button_style)
            .into()
    }
}

/// Plain text-only style for ancestor crumbs. Hover gets a subtle
/// background to signal interactivity.
fn crumb_button_style(theme: &Theme, status: button::Status) -> button::Style {
    use iced::{Background, Border};
    let palette = theme.extended_palette();
    let (background, text_color) = match status {
        button::Status::Hovered | button::Status::Pressed => (
            Some(Background::Color(palette.background.weak.color)),
            palette.primary.base.color,
        ),
        _ => (None, palette.primary.base.color),
    };
    button::Style {
        background,
        text_color,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 3.0.into(),
        },
        ..button::Style::default()
    }
}

/// Subtle gray for the separator glyph — not the same color as
/// clickable ancestor labels, so the eye reads "punctuation".
fn separator_color(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    let fg = p.background.base.text;
    let bg = p.background.base.color;
    Color {
        r: fg.r * 0.5 + bg.r * 0.5,
        g: fg.g * 0.5 + bg.g * 0.5,
        b: fg.b * 0.5 + bg.b * 0.5,
        a: 1.0,
    }
}
