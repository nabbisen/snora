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
    Background, Border, Color, Element, Length, Padding, Shadow, Theme,
    widget::{button, column, container, row, space, stack, text},
};

use snora_core::{LayoutDirection, TabAction, TabBar};

use crate::direction::row_dir;
use crate::icon::icon_element;

/// Height of the active tab's underline (RFC-102 R-1).
const INDICATOR_HEIGHT: f32 = 2.0;

/// Height of the bar's bottom rule (RFC-102 R-2). One pixel, the width
/// the container border it replaces had.
const BAR_RULE_HEIGHT: f32 = 1.0;

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
}

// `bar_border_radius` was retired by RFC-102. The bar had
// `background: None`, so its radius was only ever visible through the
// container border — and that border is now a rule element, which has
// no corners to round. A geometry field that configures nothing is a
// dead setting whose mapping test would go on "verifying" it.

impl TabGeometry {
    /// Today's literals, unmodified.
    pub(crate) const fn unstyled() -> Self {
        Self {
            bar_gap: 2.0,
            bar_pad_x: 12.0,
            content_gap: 6.0,
            tab_pad_x: 12.0,
            tab_pad_y: 8.0,
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
    // Direction affects tab order below (declaration order vs. reversed),
    // not this row's construction — both arms built the same `row![]`
    // here (F-33, RFC-089).
    let mut tab_row = row![].spacing(geometry.bar_gap).align_y(Center);

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

    // The bar's horizontal padding belongs to the row of tabs, not to the
    // bar: the rule below is the bar's bottom *edge*, so it spans the
    // whole width, as the container border it replaces did.
    let tabs = container(body)
        .width(Length::Fill)
        .padding(Padding::from([0.0, geometry.bar_pad_x]));

    let bottom_rule = container(space())
        .width(Length::Fill)
        .height(BAR_RULE_HEIGHT)
        .style(tab_bar_rule_style);

    container(column![tabs, bottom_rule])
        .style(tab_bar_container_style)
        .width(Length::Fill)
        .into()
}

/// Render a single tab. The active tab gets an underline element along
/// its bottom edge; inactive tabs look like flat text buttons.
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

    if !is_active {
        return pressable.into();
    }

    // The underline is its own element (RFC-102 R-1), laid over the
    // bottom of the tab's own box in a `stack`.
    //
    // **Why a stack rather than a column.** A column of
    // `[button, underline]` is the obvious shape, but the underline has
    // to span the tab, and a tab's width is its content's. `Length::Fill`
    // is the only way to say "as wide as the tab" — and width is a row's
    // *main* axis, so a Fill-width tab claims an equal share of the bar
    // instead: measured at 511px per tab against a 58px button. A stack
    // lays its upper layers out within the base layer's size, so Fill
    // there means exactly the button's width. It also means the tab's
    // height does not depend on whether it is active, so labels cannot
    // shift when the active tab changes.
    let indicator = container(space())
        .width(Length::Fill)
        .height(INDICATOR_HEIGHT)
        .style(tab_indicator_style);

    stack![pressable, container(indicator).align_bottom(Length::Fill)].into()
}

/// Container style for the whole tab bar.
///
/// **Paints no border (RFC-102 R-2).** It used to set
/// `Border { width: 1.0, .. }` under a comment claiming to "drop the
/// top/left/right borders; keep only a thin bottom edge" — but iced 0.14
/// borders are all-sided, so that outlined the whole bar. The bottom
/// edge is [`tab_bar_rule_style`]'s element instead, which is also what
/// makes a hovered tab's fill unable to paint over it: the rule is
/// outside every tab's box.
pub(crate) fn tab_bar_container_style(theme: &Theme) -> container::Style {
    let ep = theme.extended_palette();
    container::Style {
        text_color: Some(ep.background.base.text),
        background: None,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

/// The bar's bottom edge: a 1 px rule spanning the bar's full width.
///
/// Keeps RFC-085 F-15's measured colour, `background.base.text` — the
/// only value derivable from `Theme::extended_palette()` alone that iced
/// itself guarantees against `background.base.color`, which is what this
/// rule is drawn over. F-15's contrast guarantee moved here with it, and
/// is asserted by `contrast_tests::tab_bar_bottom_rule_meets_non_text_floor`.
pub(crate) fn tab_bar_rule_style(theme: &Theme) -> container::Style {
    let ep = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(ep.background.base.text)),
        ..container::Style::default()
    }
}

/// The active tab's underline: a straight, square 2 px line.
///
/// **`primary.strong`, not `primary.base` (RFC-102 R-3).** The underline
/// is the active tab's state indicator, so it carries WCAG 1.4.11's
/// 3.0:1 non-text floor against the page it is drawn on. `primary.base`
/// measured **2.99:1 on stock `Theme::Dark`** — under the floor, by 0.01,
/// for as long as the widget has existed. `primary.strong` measures
/// 3.70:1 there and 3.73:1 on stock Light, its two worst cases, and
/// 10.00–17.70:1 across the four design presets. It is also the shade
/// `crate::style::sidebar_active_color` moved to in RFC-085, so snora's
/// two navigation widgets now share one indicator colour.
///
/// Asserted in all six theme contexts by
/// `contrast_tests::tab_indicator_meets_non_text_floor`.
pub(crate) fn tab_indicator_style(theme: &Theme) -> container::Style {
    let ep = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(ep.primary.strong.color)),
        ..container::Style::default()
    }
}

/// Per-tab button style. The active tab is distinguished by the
/// underline element [`tab_indicator_style`] draws, not by anything
/// here; this function styles the label and the hover fill.
///
/// **No shadow, and no corner radius (RFC-102 R-1, R-5).** The underline
/// used to be faked with a solid `Shadow` offset 1.5 px down, under a
/// button with `radius: 4.0`. A shadow takes the corner radius it is cast
/// from, so the "underline" curled upward at both ends; the code's own
/// comment claimed it was "visually indistinguishable from a
/// border-bottom in normal use", and orbok's 4.5x crop showed that it is
/// not. The radius is gone with it, so the hover fill is a square block
/// within the tab (Q-1 (a)) rather than a rounded pill sitting on a
/// straight rule.
///
/// **Label colours (RFC-085).** The active label had been
/// `primary.base.color`, measured 2.99:1 on stock Dark as text, under AA.
/// Trying `primary.strong.color` was better on the `design` path but
/// still short on both stock themes (3.73:1 light, 3.70:1 dark): no shade
/// in the `primary` family reaches AA against an arbitrary page
/// background, because none of them is calibrated against it. The active
/// label is therefore `background.base.text`, iced's own guaranteed
/// pairing for the page background.
///
/// Inactive labels are `mix(background.base.text, background.base.color,
/// 0.3)`, slightly muted, as they have been since at least 0.10.0.
/// **They are not the same value as the active label** — RFC-085's own
/// docstring here said they were, and the 0.40 -> 0.41 migration guide
/// repeated it; both were wrong when written (RFC-102 names this, and
/// the guide's correction is the architect's R-6). The error is in the
/// harmless direction: a small colour distinction was described as
/// removed, and was not. The reliable distinction is the underline,
/// which RFC-102 put above the non-text floor in every theme.
pub(crate) fn tab_button_style(
    theme: &Theme,
    status: button::Status,
    is_active: bool,
) -> button::Style {
    let palette = theme.extended_palette();

    let (background, text_color) = match (is_active, status) {
        (true, _) => (None, palette.background.base.text),
        (false, button::Status::Hovered) => (
            Some(Background::Color(palette.background.weak.color)),
            palette.background.base.text,
        ),
        (false, _) => (
            None,
            // Slightly muted so the active tab reads as foreground.
            mix(
                palette.background.base.text,
                palette.background.base.color,
                0.3,
            ),
        ),
    };

    button::Style {
        background,
        text_color,
        border: Border::default(),
        shadow: Shadow::default(),
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

#[cfg(test)]
mod tests;
