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
use crate::style::chrome_container_style;

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
    let active = bar.active.clone();
    let mut tab_row = match direction {
        LayoutDirection::Ltr => row![],
        LayoutDirection::Rtl => row![],
    }
    .spacing(2)
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
        tab_row = tab_row.push(render_tab(tab, is_active, on_action));
    }

    // Leave the trailing edge fillable so the row hugs the start edge
    // without stretching tabs.
    let body = row_dir(direction, tab_row, space().width(Length::Fill));

    container(body)
        .style(tab_bar_container_style)
        .width(Length::Fill)
        .padding(Padding::from([0.0, 12.0]))
        .into()
}

/// Render a single tab. Active tabs get an underline; inactive tabs
/// look like flat text buttons.
fn render_tab<'a, Message, TabId, F>(
    tab: snora_core::Tab<TabId>,
    is_active: bool,
    on_action: &'a F,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    TabId: Clone + Debug + PartialEq + 'a,
    F: Fn(TabAction<TabId>) -> Message + 'a,
{
    let mut content = row![].spacing(6).align_y(Center);
    if let Some(icon) = &tab.icon {
        content = content.push(icon_element::<Message>(icon));
    }
    content = content.push(text(tab.label).size(13));

    let id_for_msg = tab.id.clone();
    let pressable = button(content)
        .on_press_with(move || on_action(TabAction::Pressed(id_for_msg.clone())))
        .padding(Padding::from([8.0, 12.0]))
        .style(move |theme: &Theme, status| tab_button_style(theme, status, is_active));

    pressable.into()
}

/// Container style for the whole tab bar — provides the bottom border
/// that sits under the inactive tabs and against which the active
/// tab's underline reads.
fn tab_bar_container_style(theme: &Theme) -> container::Style {
    let chrome = chrome_container_style(theme);
    let palette = theme.extended_palette();
    container::Style {
        // Drop the top/left/right borders; keep only a thin bottom
        // edge that the active-tab underline visually breaks.
        border: Border {
            color: palette.background.weak.color,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..chrome
    }
}

/// Per-tab button style. Active tabs get a 2 px underline in the
/// theme's primary color; inactive tabs sit on the chrome surface.
fn tab_button_style(theme: &Theme, status: button::Status, is_active: bool) -> button::Style {
    let palette = theme.extended_palette();

    let (background, text_color, border_color) = match (is_active, status) {
        (true, _) => (
            None,
            palette.primary.base.color,
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
