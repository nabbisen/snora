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
    widget::{button, column, container, row, space, text},
};

use snora_core::{BreadcrumbAction, Crumb, LayoutDirection};

/// Geometry parameters [`build_breadcrumb`] takes, letting
/// [`app_breadcrumb`] (unstyled) and the `design`-gated styled variant
/// (RFC-040) share one implementation.
#[derive(Debug, PartialEq)]
pub(crate) struct CrumbGeometry {
    /// Gap between a crumb and its trailing separator.
    pub(crate) gap: f32,
    /// Trail container's horizontal padding.
    pub(crate) row_pad_x: f32,
    /// Trail container's vertical padding.
    pub(crate) row_pad_y: f32,
    /// Per-crumb button horizontal padding.
    pub(crate) btn_pad_x: f32,
    /// Per-crumb button vertical padding.
    pub(crate) btn_pad_y: f32,
    /// Per-crumb button corner radius.
    pub(crate) btn_radius: f32,
    /// Crumb and separator label size, or `None` to inherit the host's
    /// `default_text_size` (RFC-104). The unstyled path inherits; the
    /// styled path maps it to `Typography::label`.
    pub(crate) label_size: Option<f32>,
}

impl CrumbGeometry {
    /// Today's literals, unmodified.
    pub(crate) const fn unstyled() -> Self {
        Self {
            gap: 6.0,
            row_pad_x: 12.0,
            row_pad_y: 4.0,
            btn_pad_x: 4.0,
            btn_pad_y: 2.0,
            btn_radius: 3.0,
            label_size: None,
        }
    }
}

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
    build_breadcrumb(crumbs, on_action, direction, CrumbGeometry::unstyled())
}

pub(crate) fn build_breadcrumb<'a, Message, CrumbId, F>(
    crumbs: Vec<Crumb<CrumbId>>,
    on_action: &'a F,
    direction: LayoutDirection,
    geometry: CrumbGeometry,
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

    let mut trail = row![].spacing(geometry.gap).align_y(Center);

    let last = crumbs.len().saturating_sub(1);
    for (i, crumb) in crumbs.into_iter().enumerate() {
        trail = trail.push(render_crumb(
            crumb,
            on_action,
            geometry.btn_pad_x,
            geometry.btn_pad_y,
            geometry.btn_radius,
            geometry.label_size,
        ));
        if i < last {
            // The separator follows the crumbs' own size (RFC-104).
            let mut glyph = text(separator).style(|theme: &Theme| iced::widget::text::Style {
                color: Some(separator_color(theme)),
            });
            if let Some(size) = geometry.label_size {
                glyph = glyph.size(size);
            }
            trail = trail.push(glyph);
        }
    }

    container(trail)
        .width(Length::Fill)
        .padding(Padding::from([geometry.row_pad_y, geometry.row_pad_x]))
        .into()
}

/// Height of an ancestor's underline, in logical pixels (RFC-109).
///
/// One pixel is the minimum that can be drawn at all; the rule is a
/// mark, not a weight, so it takes the minimum.
pub(crate) const RULE_HEIGHT: f32 = 1.0;

/// The ancestor's underline (RFC-109 Q-1/Q-2).
///
/// **`snap: true` is honoured by wgpu and ignored by tiny-skia.** The
/// flag reaches the quad (`iced_widget::container` forwards it to
/// `renderer::Quad::snap`), but in iced 0.14 only `iced_wgpu` reads it
/// — `iced_tiny_skia` has no snapping at all. snora's harness is
/// pinned to tiny-skia, so **no test here can observe it**: measured
/// with and without the flag under tiny-skia, the drawn rows are
/// byte-identical. Measured under wgpu, on the tab bar's equivalent
/// rule, it removes the smear completely (RFC-108 Unit 0).
///
/// The offset is fractional either way and cannot be made whole in
/// layout: it includes the label's height, which is the host's
/// `default_text_size` times the line height — 26.8 at the default
/// size, 26.2 on the design path. Snapping at draw time is the only
/// place it can be fixed, which is why the flag is set even though
/// nothing here can check it.
///
/// **The colour is the resting label colour, `background.base.text`,
/// in every status.** A container's style closure receives the theme
/// and nothing else — it cannot see whether the button above it is
/// hovered — so the rule cannot follow `crumb_button_style`'s hovered
/// arm (`background.weak.text`). On the four design presets and on 14
/// of iced's 22 stock themes those two colours are the same value, so
/// the rule matches the label exactly; on the other 8 the hovered
/// label shifts by one tier while the rule holds.
///
/// **On hover the rule is drawn over `background.weak.color`, and
/// `base.text` is not iced's pairing for that tier** — it is the
/// pairing for `base.color`. So nothing guarantees it, and it is
/// asserted instead:
/// `contrast_tests::crumb_rule_stays_visible_on_the_hover_fill`
/// measures it over all 22 stock themes and all four presets. Lowest
/// is Kanagawa Lotus at 4.51:1, against a 3.0 floor.
pub(crate) fn crumb_rule_style(theme: &Theme) -> container::Style {
    use iced::Background;
    container::Style {
        background: Some(Background::Color(
            theme.extended_palette().background.base.text,
        )),
        snap: true,
        ..container::Style::default()
    }
}

fn render_crumb<'a, Message, CrumbId, F>(
    crumb: Crumb<CrumbId>,
    on_action: &'a F,
    btn_pad_x: f32,
    btn_pad_y: f32,
    btn_radius: f32,
    label_size: Option<f32>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    CrumbId: Clone + Debug + 'a,
    F: Fn(BreadcrumbAction<CrumbId>) -> Message + 'a,
{
    let mut label = text(crumb.label);
    if let Some(size) = label_size {
        label = label.size(size);
    }

    if crumb.is_leaf {
        // Plain text — the user is here, and the absence of the rule
        // below is what says so (RFC-109).
        label.into()
    } else {
        let id_for_msg = crumb.id.clone();
        // The rule goes inside the button, under the label and spanning
        // it: inside, so the whole mark is part of the same pointer
        // target and moves with it; `Length::Fill` inside a column that
        // shrinks to its content, so it takes the label's width rather
        // than a width this function would have to guess.
        let underlined = column![
            label,
            container(space())
                .width(Length::Fill)
                .height(RULE_HEIGHT)
                .style(crumb_rule_style),
        ]
        .width(Length::Shrink);
        button(underlined)
            .on_press_with(move || on_action(BreadcrumbAction::Pressed(id_for_msg.clone())))
            .padding(Padding::from([btn_pad_y, btn_pad_x]))
            .style(move |theme, status| crumb_button_style(theme, status, btn_radius))
            .into()
    }
}

/// Plain text-only style for ancestor crumbs. Hover gets a subtle
/// background to signal interactivity.
///
/// **Corrected (RFC-085, found by the widget-layer suite's own derived
/// coverage — not one of F-13/F-14/F-15).** `text_color` was always
/// `primary.base.color`, a **background-tier** family used as a
/// foreground against `background.weak.color` (hovered/pressed) or the
/// page background (default) — neither of which it was calibrated
/// against. Measured: 3.42:1 (stock light), 2.99–2.03:1 (stock dark).
/// `background.base.text` / `background.weak.text` are iced's own
/// guaranteed pairings for the two backgrounds this actually paints.
pub(crate) fn crumb_button_style(
    theme: &Theme,
    status: button::Status,
    radius: f32,
) -> button::Style {
    use iced::{Background, Border};
    let palette = theme.extended_palette();
    let (background, text_color) = match status {
        button::Status::Hovered | button::Status::Pressed => (
            Some(Background::Color(palette.background.weak.color)),
            palette.background.weak.text,
        ),
        _ => (None, palette.background.base.text),
    };
    button::Style {
        background,
        text_color,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius.into(),
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
