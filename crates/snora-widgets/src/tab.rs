//! A horizontal tab strip — typically placed under the header.
//!
//! Layout (logical, ABDD):
//!
//! ```text
//!  ┌──────────────────────────────────────────────────────────────────┐
//!  │ [Tab A] [Tab B*] [Tab C] [Tab D] ...                             │
//!  └──────────────────────────────────────────────────────────────────┘
//!    └─────── start (LTR) / end (RTL) — first tab ─────────────┘
//! ```
//!
//! Under [`LayoutDirection::Rtl`] the tab order is mirrored as a whole;
//! individual tab labels keep their text direction (set by iced's
//! BiDi handling, which this widget does not override).

use std::fmt::Debug;

use iced::{
    Alignment::Center,
    Background, Border, Color, Element, Length, Padding, Theme,
    widget::{button, container, row, space, text},
};

use snora_core::{LayoutDirection, TabAction, TabBar};

use crate::direction::row_dir;
use crate::icon::icon_element;
use crate::style::chrome_container_style_with_radius;

/// Geometry parameters [`build_tab_bar`] takes, letting [`app_tab_bar`]
/// (unstyled) and the `design`-gated styled variant (RFC-040) share one
/// implementation.
#[derive(Debug, PartialEq)]
pub(crate) struct TabGeometry {
    /// Gap between tabs.
    pub(crate) bar_gap: f32,
    /// Bar's own horizontal padding. Vertical padding is a structural
    /// `0.0` in both paths — tabs supply their own vertical padding
    /// (`tab_pad_y`), not part of this geometry.
    pub(crate) bar_pad_x: f32,
    /// Gap between a tab's icon and its label.
    pub(crate) content_gap: f32,
    /// Per-tab button horizontal padding.
    pub(crate) tab_pad_x: f32,
    /// Per-tab button vertical padding.
    pub(crate) tab_pad_y: f32,
    /// Bar's own corner radius.
    pub(crate) bar_border_radius: f32,
}

impl TabGeometry {
    /// Today's literals, unmodified.
    pub(crate) const fn unstyled() -> Self {
        Self {
            bar_gap: 2.0,
            bar_pad_x: 12.0,
            content_gap: 6.0,
            tab_pad_x: 12.0,
            tab_pad_y: 8.0,
            bar_border_radius: 0.0,
        }
    }
}

/// Build a horizontal tab bar.
///
/// * `bar` — the tab list and the currently active id. Cloned by the
///   widget; the application keeps its own copy.
/// * `on_action` — maps [`TabAction`] into your message type.
/// * `direction` — application's reading direction. Determines the
///   visual order of the tabs.
///
/// The active tab is rendered with a colored underline drawn from the
/// theme's primary palette. Each tab is a regular `button`, so keyboard
/// focus and click semantics come from iced.
pub fn app_tab_bar<'a, Message, TabId, F>(
    bar: TabBar<TabId>,
    on_action: &'a F,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    TabId: Clone + Debug + PartialEq + 'a,
    F: Fn(TabAction<TabId>) -> Message + 'a,
{
    build_tab_bar(bar, on_action, direction, TabGeometry::unstyled())
}

pub(crate) fn build_tab_bar<'a, Message, TabId, F>(
    bar: TabBar<TabId>,
    on_action: &'a F,
    direction: LayoutDirection,
    geometry: TabGeometry,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    TabId: Clone + Debug + PartialEq + 'a,
    F: Fn(TabAction<TabId>) -> Message + 'a,
{
    let active = bar.active.clone();
    let mut tab_row = match direction {
        LayoutDirection::Ltr => row![],
        LayoutDirection::Rtl => row![],
    }
    .spacing(geometry.bar_gap)
    .align_y(Center);

    // We push tabs in declaration order under LTR and reverse order
    // under RTL, so that the *first declared* tab visually leads in
    // both reading directions. This matches how `row_dir` treats its
    // start/end pair, and what users expect from `vec[0]` being the
    // primary tab.
    let tabs: Vec<_> = match direction {
        LayoutDirection::Ltr => bar.tabs.into_iter().collect(),
        LayoutDirection::Rtl => bar.tabs.into_iter().rev().collect(),
    };

    for tab in tabs {
        let is_active = tab.id == active;
        tab_row = tab_row.push(render_tab(
            tab,
            is_active,
            on_action,
            geometry.content_gap,
            geometry.tab_pad_x,
            geometry.tab_pad_y,
        ));
    }

    // Leave the trailing edge fillable so the row hugs the start edge
    // without stretching tabs.
    let body = row_dir(direction, tab_row, space().width(Length::Fill));

    let bar_border_radius = geometry.bar_border_radius;
    container(body)
        .style(move |theme| tab_bar_container_style(theme, bar_border_radius))
        .width(Length::Fill)
        .padding(Padding::from([0.0, geometry.bar_pad_x]))
        .into()
}

/// Render a single tab. Active tabs get an underline; inactive tabs
/// look like flat text buttons.
fn render_tab<'a, Message, TabId, F>(
    tab: snora_core::Tab<TabId>,
    is_active: bool,
    on_action: &'a F,
    content_gap: f32,
    tab_pad_x: f32,
    tab_pad_y: f32,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    TabId: Clone + Debug + PartialEq + 'a,
    F: Fn(TabAction<TabId>) -> Message + 'a,
{
    let mut content = row![].spacing(content_gap).align_y(Center);
    if let Some(icon) = &tab.icon {
        content = content.push(icon_element::<Message>(icon));
    }
    content = content.push(text(tab.label).size(13));

    let id_for_msg = tab.id.clone();
    let pressable = button(content)
        .on_press_with(move || on_action(TabAction::Pressed(id_for_msg.clone())))
        .padding(Padding::from([tab_pad_y, tab_pad_x]))
        .style(move |theme: &Theme, status| tab_button_style(theme, status, is_active));

    pressable.into()
}

/// Container style for the whole tab bar — provides the bottom border
/// that sits under the inactive tabs and against which the active
/// tab's underline reads.
pub(crate) fn tab_bar_container_style(theme: &Theme, border_radius: f32) -> container::Style {
    let chrome = chrome_container_style_with_radius(theme, border_radius);
    let palette = theme.extended_palette();
    container::Style {
        // Drop the top/left/right borders; keep only a thin bottom
        // edge that the active-tab underline visually breaks.
        //
        // Corrected (RFC-085 F-15), same fix as
        // `chrome_container_style_with_radius` — see its own comment for
        // why `background.base.text` and not `background.weak`/`strong`.
        border: Border {
            color: palette.background.base.text,
            width: 1.0,
            radius: border_radius.into(),
        },
        ..chrome
    }
}

/// Per-tab button style. Active tabs get a 2 px underline in the
/// theme's primary color; inactive tabs sit on the chrome surface.
///
/// **Corrected (RFC-085, found by the widget-layer suite's own derived
/// coverage — not one of F-13/F-14/F-15).** The active tab's label used
/// `primary.base.color` against the page background — 2.99:1 on stock
/// Dark, under AA. Tried `primary.strong.color` next: better on the
/// `design` path (already clears AA there) but still short on **both**
/// stock themes (3.73:1 light, 3.70:1 dark) — no shade in the `primary`
/// family reaches AA against an arbitrary page background, because none
/// of them were calibrated against it (`primary`'s own `.text` fields
/// are calibrated against `primary`'s own colors, not against
/// `background`). Settled on `background.base.text`, the same value
/// inactive tabs already use — the label no longer visually
/// distinguishes active from inactive by color, but the underline
/// (drawn via `shadow`, not `border` — `border_color` here has
/// `width: 0.0` and paints nothing) still does, and a decorative
/// accent line carries no text-contrast requirement the way the label
/// itself does.
pub(crate) fn tab_button_style(
    theme: &Theme,
    status: button::Status,
    is_active: bool,
) -> button::Style {
    let palette = theme.extended_palette();

    let (background, text_color, border_color) = match (is_active, status) {
        (true, _) => (
            None,
            palette.background.base.text,
            palette.primary.base.color,
        ),
        (false, button::Status::Hovered) => (
            Some(Background::Color(palette.background.weak.color)),
            palette.background.base.text,
            Color::TRANSPARENT,
        ),
        (false, _) => (
            None,
            // Slightly muted so the active tab reads as foreground.
            mix(
                palette.background.base.text,
                palette.background.base.color,
                0.3,
            ),
            Color::TRANSPARENT,
        ),
    };

    button::Style {
        background,
        text_color,
        border: Border {
            color: border_color,
            width: 0.0,
            radius: 4.0.into(),
        },
        // The "underline" is a 2 px bottom border drawn via the
        // shadow's offset — iced 0.14 doesn't expose per-side border
        // widths on `button::Style`, so for the active state we fake
        // the bar with a solid colored shadow flush against the
        // bottom edge. This is visually indistinguishable from a
        // border-bottom in normal use.
        shadow: if is_active {
            iced::Shadow {
                color: palette.primary.base.color,
                offset: iced::Vector::new(0.0, 1.5),
                blur_radius: 0.0,
            }
        } else {
            iced::Shadow::default()
        },
        ..button::Style::default()
    }
}

/// Linearly mix two colors. Used to derive a "muted" foreground for
/// inactive tab labels.
fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r * (1.0 - t) + b.r * t,
        g: a.g * (1.0 - t) + b.g * t,
        b: a.b * (1.0 - t) + b.b * t,
        a: a.a * (1.0 - t) + b.a * t,
    }
}
