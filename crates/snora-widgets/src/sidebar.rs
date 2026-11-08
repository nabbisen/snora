//! Navigation rail.
//!
//! Produces a vertical strip of items, each an icon with its name. Where
//! the name goes is the application's choice, [`SideBarLabels`]: beneath
//! the icon (the default), beside it, or — as every rail was before
//! RFC-114 — only in a tooltip under the pointer. The active item
//! (matching [`SideBar::active`]) gets a background highlight, which
//! covers the whole item.
//!
//! A labelled rail is content-sized, as wide as its longest name needs
//! and never past [`LABELLED_RAIL_MAX_WIDTH`]. The tooltip form is the
//! fixed [`RAIL_WIDTH`] strip of [`BUTTON_SIZE`] squares it always was,
//! with its tooltip on the end side of the rail so it never overlaps the
//! main content.
//!
//! **Every form of this rail is reachable by keyboard** (RFC-116): each
//! item is a focus target, Enter and Space press it, and a ring shows
//! which has focus. See [`SideBarLabels`].

use std::rc::Rc;

use iced::widget::text::Wrapping;
use iced::{
    Alignment, Background, Border, Element, Length, Padding, Shadow, Theme,
    widget::{button, column, container, text, tooltip},
};

use snora_core::{LayoutDirection, SideBar};

use crate::direction::row_dir;
use crate::icon::icon_beside_label;
use crate::style::ControlGeometry;
use crate::style::sidebar_active_color;

/// Default rail width in pixels.
const RAIL_WIDTH: f32 = 64.0;
/// Default button size — the icon's square box, in every form.
const BUTTON_SIZE: f32 = 48.0;

/// The rail's horizontal padding: whatever width is left once a button is
/// centred in the rail, split evenly (RFC-099, Q-1 (a)).
///
/// **Derived, not chosen.** The rail used to pad all four sides by the same
/// `16`, leaving a 32px content box for a 48px button, which iced resolved
/// by rendering the button 32px wide — in both variants and every preset,
/// since at least 0.10.0. Horizontal padding is not a spacing decision; it
/// is what is left over, so it is computed here from the two dimensions
/// that are decisions.
///
/// Vertical padding and the inter-button gap remain spacing decisions and
/// keep their sources (literals unstyled, `Spacing` tokens styled).
const RAIL_HORIZONTAL_PADDING: f32 = (RAIL_WIDTH - BUTTON_SIZE) / 2.0;

// A button wider than its rail would make the derived padding negative.
// Checked at compile time rather than in a test: this is the one arithmetic
// fact about these constants that can actually fail.
const _: () = assert!(
    BUTTON_SIZE <= RAIL_WIDTH,
    "sidebar BUTTON_SIZE exceeds RAIL_WIDTH"
);

/// How wide a labelled rail may grow (RFC-114 R-2).
///
/// A labelled rail is content-sized: it is as wide as its longest name
/// needs, so `RAIL_WIDTH` is a floor for it rather than its width. Without
/// a ceiling one long name would take the window, so the names wrap at
/// this one instead.
///
/// **Confirmed by measurement, not chosen.** Every rail in `examples/` was
/// measured on both paths at the host's default text size, at a host text
/// size of 24px, and with `Typography` scaled x2.0 (an adopter's setting).
/// The widest rail of the 36 measured is 196px — `Beside`, the workbench's
/// four names, at x2.0 — so 240 cuts nothing any example asks for, and
/// leaves a name about 14 characters long on one line at the largest of
/// those sizes. The figures are in RFC-114's review request, under
/// `evidence/unit2-width-measurement.md`.
const LABELLED_RAIL_MAX_WIDTH: f32 = 240.0;

// A cap below the icon rail's own width would make a labelled rail
// narrower than the icon it contains. Checked at compile time, like the
// button-fits-the-rail fact above.
const _: () = assert!(
    RAIL_WIDTH <= LABELLED_RAIL_MAX_WIDTH,
    "sidebar LABELLED_RAIL_MAX_WIDTH is below RAIL_WIDTH"
);

/// Where a labelled rail puts an item's name (RFC-114 R-1).
///
/// The application's choice, defaulting to [`SideBarLabels::Beneath`]:
/// before RFC-114 every name lived in a hover tooltip, which a touch user
/// never sees at all.
///
/// # Every form of this rail is keyboard-reachable
///
/// An item is a `control`, not an iced `button`: it registers as a focus
/// target, Enter and Space press it (publishing the item's `on_press`),
/// and a ring shows which has focus. iced binds no Tab, so the
/// application routes it with `snora::keyboard::traverse`; Tab past the
/// last item leaves the rail, and does not wrap to the first. Where the
/// names are drawn changes none of this (RFC-116).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SideBarLabels {
    /// The icon, with the name centred beneath it. The default.
    #[default]
    Beneath,
    /// The icon at the start edge, then the name — mirrored under RTL.
    Beside,
    /// Icon only, with the name in a tooltip.
    ///
    /// What every rail looked like before RFC-114. iced opens a tooltip
    /// only while the cursor rests on the item
    /// (`iced_widget-0.14.2/src/tooltip.rs:224`), so this form names its
    /// items **only under the pointer**: a touch user gets no name at
    /// all, and a pointer user gets one only for the item they already
    /// guessed to hover. Choose it when the rail's icons are
    /// self-evident to every one of your users and the space matters.
    Tooltip,
}

/// How much width a name has on a labelled rail, once the rail's own
/// padding and — in `Beside` — the icon and its gap are taken out of
/// [`LABELLED_RAIL_MAX_WIDTH`].
///
/// # Why the name is bounded here and not only by the rail
///
/// `container(col).max_width(..)` caps the rail, but it does not decide
/// how the room inside is shared, and iced's row hands it out in child
/// order: the first child is laid out against everything available, and
/// what is left goes to the next. Under RTL the name comes first, so an
/// unbounded name takes the room and the icon is laid out against the
/// remainder.
///
/// **Measured**, with a 36-character name and no bound: the icon's glyph
/// is drawn **7.63px wide under LTR and 2.26px under RTL** — squeezed to
/// under a third, in one direction only, which is exactly the kind of
/// defect ABDD exists to prevent. With the bound it is 7.63px in both.
///
/// `Beneath` does not need this: its column is a cross-axis fit, so the
/// rail's own `max_width` already bounds the name there (measured
/// 222.46px against the 224px available). The arm is spelled out anyway
/// — the two forms share one cap, and a form whose bound comes from
/// somewhere else is a thing a reader would have to go and check.
fn name_max_width(placement: LabelPlacement, label_gap: f32) -> f32 {
    let inside_rail = LABELLED_RAIL_MAX_WIDTH - 2.0 * RAIL_HORIZONTAL_PADDING;

    match placement {
        LabelPlacement::Beneath => inside_rail,
        LabelPlacement::Beside => inside_rail - BUTTON_SIZE - label_gap,
    }
}

/// Which of the two labelled forms [`labelled_rail`] is drawing.
///
/// [`SideBarLabels`] minus `Tooltip`, which `build_side_bar` has already
/// dispatched away. Having it means `labelled_rail` matches exhaustively
/// over the forms it can actually draw, rather than carrying an
/// unreachable arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LabelPlacement {
    Beneath,
    Beside,
}

/// The unstyled tooltip body's corner radius: the rail's own button
/// radius literal, so the tooltip is shaped like the buttons it belongs
/// to.
const UNSTYLED_TOOLTIP_RADIUS: f32 = 6.0;

/// The unstyled tooltip body's padding and its gap from the rail.
///
/// Derived from the rail's one spacing literal, `16` (the inter-button
/// gap and the vertical padding are both `16`): half of it horizontally,
/// a quarter of it vertically and for the gap. That lands on `[4, 8]`
/// with a gap of `4`, which is also what the styled body's `[xs, sm]`
/// and `xs` resolve to in all four shipped presets — so the two variants
/// are the same shape without either one reading the other's source
/// (RFC-102 Q-2 (a)).
const UNSTYLED_TOOLTIP_PADDING: Padding = Padding {
    top: 4.0,
    right: 8.0,
    bottom: 4.0,
    left: 8.0,
};
const UNSTYLED_TOOLTIP_GAP: f32 = 4.0;

/// How a variant draws the body behind its tooltips (RFC-102 R-4).
///
/// **Why this is a parameter and not a [`SideBarGeometry`] field.**
/// `build_side_bar` is shared, and only the styled caller has tokens, so
/// the body has to be carried in from the caller either way. It is a
/// style, not geometry: `SideBarGeometry` holds four numbers and derives
/// `Debug`/`PartialEq` for the tests that compare them field by field,
/// and a style function is neither comparable nor printable. Keeping it
/// separate leaves the geometry struct exactly as it was.
///
/// No public API: both constructors are crate-internal, and applications
/// reach them only through `app_side_bar`'s two variants.
#[derive(Clone)]
pub(crate) struct TooltipBody {
    /// Container style for the body. `Rc` because one body is shared by
    /// every item in the rail, and each item's closure needs its own
    /// handle.
    style: Rc<dyn Fn(&Theme) -> container::Style>,
    /// Padding between the body's edge and its text.
    padding: Padding,
    /// Gap between the rail button and the body.
    gap: f32,
    /// Text size, or `None` to keep iced's default.
    text_size: Option<f32>,
}

impl TooltipBody {
    pub(crate) fn new(
        style: Rc<dyn Fn(&Theme) -> container::Style>,
        padding: Padding,
        gap: f32,
        text_size: Option<f32>,
    ) -> Self {
        Self {
            style,
            padding,
            gap,
            text_size,
        }
    }

    /// The unstyled variant's body: the theme's own page background, the
    /// chrome border, and the text colour iced guarantees against that
    /// background (RFC-102 Q-2 (a)).
    pub(crate) fn unstyled() -> Self {
        Self::new(
            Rc::new(unstyled_tooltip_body_style),
            UNSTYLED_TOOLTIP_PADDING,
            UNSTYLED_TOOLTIP_GAP,
            None,
        )
    }
}

/// Style of the unstyled variant's tooltip body.
///
/// Before RFC-102 the sidebar drew `tooltip(btn, text(..), ..)` with no
/// body at all, so the glyphs landed on whatever the application had
/// rendered beside the rail and their contrast was the page's business,
/// not snora's. `background.base.color` under
/// `background.base.text` is iced's own guaranteed-readable pairing, and
/// the 1 px `background.base.text` border is the same value the chrome
/// containers use (RFC-085 F-15), so the body is visible against the page
/// as well as legible on itself. Both are asserted in all six theme
/// contexts by `contrast_tests`.
pub(crate) fn unstyled_tooltip_body_style(theme: &Theme) -> container::Style {
    let ep = theme.extended_palette();
    container::Style {
        text_color: Some(ep.background.base.text),
        background: Some(Background::Color(ep.background.base.color)),
        border: Border {
            color: ep.background.base.text,
            width: 1.0,
            radius: UNSTYLED_TOOLTIP_RADIUS.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Geometry parameters [`build_side_bar`] takes, letting [`app_side_bar`]
/// (unstyled) and the `design`-gated styled variant (RFC-040) share one
/// implementation.
///
/// Horizontal padding is deliberately absent: it is derived from
/// [`RAIL_WIDTH`] and [`BUTTON_SIZE`] ([`RAIL_HORIZONTAL_PADDING`]) and does
/// not vary by variant or token.
#[derive(Debug, PartialEq)]
pub(crate) struct SideBarGeometry {
    /// Gap between icon buttons.
    pub(crate) gap: f32,
    /// Rail's padding above the first button and below the last.
    ///
    /// Renamed from `padding` (RFC-099): that field was applied to all four
    /// sides, and horizontally it did not fit. It now means only the half it
    /// can safely mean.
    pub(crate) vertical_padding: f32,
    /// Icon button corner radius.
    pub(crate) button_radius: f32,
    /// Size of an item's name on a labelled rail, or `None` to inherit
    /// the host's `default_text_size`.
    ///
    /// RFC-104: the unstyled path inherits, so an application that sets a
    /// larger text size gets larger rail names without snora knowing
    /// about it; the design path uses `Typography::label`, the role the
    /// other chrome labels use. Unused by
    /// [`SideBarLabels::Tooltip`], which sizes its tooltip text through
    /// [`TooltipBody`] instead.
    pub(crate) label_size: Option<f32>,
    /// The minimum-target token and the focus ring (RFC-116).
    pub(crate) control: ControlGeometry,
}

impl SideBarGeometry {
    /// The unstyled variant's literals.
    pub(crate) const fn unstyled() -> Self {
        Self {
            gap: 16.0,
            vertical_padding: 16.0,
            button_radius: 6.0,
            label_size: None,
            control: ControlGeometry::unstyled(),
        }
    }
}

/// Render a [`SideBar`] as a rail that names its items beneath their
/// icons.
///
/// Equivalent to [`app_side_bar_with`] with [`SideBarLabels::Beneath`].
/// To choose another form — names beside the icons, or icons only with
/// the names in tooltips — call that instead.
pub fn app_side_bar<'a, Message, ViewId>(
    side_bar: SideBar<Message, ViewId>,
    direction: LayoutDirection,
) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
    ViewId: Clone + PartialEq + 'a,
{
    app_side_bar_with(side_bar, direction, SideBarLabels::Beneath)
}

/// Render a [`SideBar`], choosing where its items are named (RFC-114).
///
/// The rail is content-sized in the two labelled forms — as wide as its
/// longest name needs, and never wider than 240px, which the names wrap
/// at. In [`SideBarLabels::Tooltip`] it is the fixed 64px icon rail.
///
/// Every form of the rail is reachable by keyboard: see [`SideBarLabels`].
pub fn app_side_bar_with<'a, Message, ViewId>(
    side_bar: SideBar<Message, ViewId>,
    direction: LayoutDirection,
    labels: SideBarLabels,
) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
    ViewId: Clone + PartialEq + 'a,
{
    build_side_bar(
        side_bar,
        direction,
        SideBarGeometry::unstyled(),
        TooltipBody::unstyled(),
        labels,
    )
}

pub(crate) fn build_side_bar<'a, Message, ViewId>(
    side_bar: SideBar<Message, ViewId>,
    direction: LayoutDirection,
    geometry: SideBarGeometry,
    tooltip_body: TooltipBody,
    labels: SideBarLabels,
) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
    ViewId: Clone + PartialEq + 'a,
{
    match labels {
        SideBarLabels::Beneath => {
            labelled_rail(side_bar, direction, geometry, LabelPlacement::Beneath)
        }
        SideBarLabels::Beside => {
            labelled_rail(side_bar, direction, geometry, LabelPlacement::Beside)
        }
        SideBarLabels::Tooltip => icon_rail(side_bar, direction, geometry, tooltip_body),
    }
}

/// The icon-only rail: a 64px strip of 48x48 icon buttons, each naming
/// its item in a tooltip. What every rail was before RFC-114.
fn icon_rail<'a, Message, ViewId>(
    side_bar: SideBar<Message, ViewId>,
    direction: LayoutDirection,
    geometry: SideBarGeometry,
    tooltip_body: TooltipBody,
) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
    ViewId: Clone + PartialEq + 'a,
{
    let tooltip_position = match direction {
        LayoutDirection::Ltr => tooltip::Position::Right,
        LayoutDirection::Rtl => tooltip::Position::Left,
    };

    let mut col = column![].spacing(geometry.gap).align_x(Alignment::Center);

    for (index, item) in side_bar.items.into_iter().enumerate() {
        let is_active = item.view_id == side_bar.active;
        // 1 em of the label size its name would have (RFC-115's
        // amendment): the Tooltip form shows no name on the rail, but the
        // icon is sized as if it did, so it does not change size between
        // forms.
        let icon = icon_beside_label(&item.icon, tooltip_body.text_size);
        let button_radius = geometry.button_radius;

        // Centred on both axes (RFC-099). Without this, a fixed-size button
        // hands its content fixed limits, the icon's text widget fills the
        // whole content area, and the glyph is drawn at that box's top-left:
        // measured ~10px left of and ~10px above the button's centre. The
        // container passes *loose* limits to the icon, so the glyph box
        // shrinks to its natural size and is then placed at the centre.
        //
        // The button's padding is zeroed. iced's default is 10 horizontal and
        // 5 vertical, which is symmetric, so centring within the padded area
        // would coincidentally land on the same point — but it would leave
        // an icon only 28 x 38 to live in, clipping any icon larger than that,
        // and it would make "centred in the button" depend on the default
        // staying symmetric. With zero padding the container is the full
        // 48 x 48 and centring means exactly what it says.
        let centred_icon = container(icon).center(Length::Fill);

        // RFC-116 Unit 0: `control` here too, so all three forms of the
        // item are reachable rather than two of them.
        let btn = rail_item(
            centred_icon,
            is_active,
            button_radius,
            geometry.control,
            tooltip_body.text_size,
        )
        .on_press(item.on_press.clone())
        // A shortcut to this item finds it by this (RFC-116 R-2).
        .tag(snora_interaction::Role::SideBarItem, index)
        .padding(0)
        .width(BUTTON_SIZE)
        .height(BUTTON_SIZE);

        // The tooltip gets a body of its own (RFC-102 R-4): bare text
        // over the page is legible only by luck.
        //
        // Bounded and wrapped (RFC-105 R-1). A tooltip is content-sized,
        // so a long label had nothing to wrap against but the viewport
        // and was cut off inside the overlay — measured by frame
        // comparison, since tooltip text is invisible to
        // `Simulator::find`. `max_width` gives it a line to wrap at, and
        // `WordOrGlyph` breaks text that has no spaces to break at.
        // Both variants share this: the styled one differs only in the
        // body's style, padding, gap and text size.
        let mut label = text(item.tooltip).wrapping(Wrapping::WordOrGlyph);
        if let Some(size) = tooltip_body.text_size {
            label = label.size(size);
        }
        let body_style = Rc::clone(&tooltip_body.style);
        let body = container(label)
            .max_width(crate::style::TOOLTIP_MAX_WIDTH)
            .padding(tooltip_body.padding)
            .style(move |theme: &Theme| body_style(theme));

        let with_tip = tooltip(btn, body, tooltip_position).gap(tooltip_body.gap);
        col = col.push(with_tip);
    }

    // `Padding::from([a, b])` is `[vertical, horizontal]` in iced_core 0.14
    // (top/bottom = a, left/right = b).
    container(col)
        .width(RAIL_WIDTH)
        .height(Length::Fill)
        .padding(Padding::from([
            geometry.vertical_padding,
            RAIL_HORIZONTAL_PADDING,
        ]))
        .into()
}

/// A rail item's control: the sidebar's fill, the ring, and the sizing
/// rules (RFC-116).
///
/// # Why the ring's colour is chosen here
///
/// The ring has to clear 3:1 against **whatever is behind it**, and that
/// is not one thing: an inactive item sits on the rail, an active one on
/// its own `primary.strong` highlight. `control` cannot sample what is
/// underneath, so the decision belongs where `is_active` is known. The
/// rule itself is [`crate::style::focus_ring`]'s.
fn rail_item<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    is_active: bool,
    button_radius: f32,
    geometry: ControlGeometry,
    label_size: Option<f32>,
) -> snora_interaction::Control<'a, Message> {
    crate::style::control(
        content,
        move |theme: &Theme, status| sidebar_button_style(theme, status, is_active, button_radius),
        geometry,
        label_size,
        move |theme: &Theme| {
            if is_active {
                sidebar_active_color(theme)
            } else {
                crate::style::page_background(theme)
            }
        },
    )
}

/// A rail that names its items on the rail itself (RFC-114).
///
/// # Why this is content-sized and the icon rail is not
///
/// The icon rail's width is a constant: a 48px button plus the padding
/// left over once it is centred. A name has no constant width, so a
/// labelled rail is as wide as its longest name needs —
/// `Length::Shrink`, bounded by [`LABELLED_RAIL_MAX_WIDTH`], which is
/// also the width the names wrap at.
///
/// # What the highlight covers
///
/// The whole item is one button, so the active highlight spans the icon
/// and the name together. In the icon rail the button was the icon alone,
/// because the name was not on the rail to cover.
///
/// # What the items align to
///
/// `Beneath` centres each item, so a name sits under its own icon.
/// `Beside` aligns them to the **logical start** edge, so every icon
/// lands at the same place however long its neighbour's name is — the
/// start edge being the left under LTR and the right under RTL, since
/// iced's `Column::align_x` is physical.
fn labelled_rail<'a, Message, ViewId>(
    side_bar: SideBar<Message, ViewId>,
    direction: LayoutDirection,
    geometry: SideBarGeometry,
    placement: LabelPlacement,
) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
    ViewId: Clone + PartialEq + 'a,
{
    // Half the rail's own inter-item gap: an icon and the name belonging
    // to it are closer together than two different items are, and this
    // keeps the two distances related rather than independently chosen.
    // On the design path `geometry.gap` is a `Spacing` token, so this
    // follows it.
    let label_gap = geometry.gap / 2.0;

    let align = match (placement, direction) {
        (LabelPlacement::Beneath, LayoutDirection::Ltr)
        | (LabelPlacement::Beneath, LayoutDirection::Rtl) => Alignment::Center,
        (LabelPlacement::Beside, LayoutDirection::Ltr) => Alignment::Start,
        (LabelPlacement::Beside, LayoutDirection::Rtl) => Alignment::End,
    };

    let mut col = column![].spacing(geometry.gap).align_x(align);

    for (index, item) in side_bar.items.into_iter().enumerate() {
        let is_active = item.view_id == side_bar.active;
        let button_radius = geometry.button_radius;

        // The icon keeps the 48x48 box it has on the icon rail, centred
        // on both axes for the reason RFC-099 gives there. It is what
        // makes every item at least 48px tall, whichever form this is,
        // and it keeps the icons on a labelled rail the same size as the
        // icons on an icon rail (icon sizing itself is RFC-115's).
        let icon_box = container(icon_beside_label(&item.icon, geometry.label_size))
            .center(Length::Fixed(BUTTON_SIZE));

        // Wrapped, never clipped (RFC-105 R-1). `WordOrGlyph` also
        // breaks a name with no space in it, which a word-only wrap
        // would push past the cap instead.
        let mut name = text(item.tooltip).wrapping(Wrapping::WordOrGlyph);
        if let Some(size) = geometry.label_size {
            name = name.size(size);
        }

        let name = container(name).max_width(name_max_width(placement, label_gap));

        let content: Element<'a, Message> = match placement {
            LabelPlacement::Beneath => column![icon_box, name]
                .spacing(label_gap)
                .align_x(Alignment::Center)
                .into(),
            LabelPlacement::Beside => row_dir(direction, icon_box, name)
                .spacing(label_gap)
                .align_y(Alignment::Center)
                .into(),
        };

        // Zero padding, as on the icon rail: the rail's own horizontal
        // padding already insets the item, and a second inset here would
        // put the icons further from the rail edge than they are today
        // for no reason a reader could see.
        //
        // **RFC-116 Unit 0:** `control` in place of `button`, so the item
        // is a focus target and Enter and Space reach it. The style
        // closure is unchanged, because `control` takes
        // `button::Status` — it is drop-in on the styling side.
        let btn = rail_item(
            content,
            is_active,
            button_radius,
            geometry.control,
            geometry.label_size,
        )
        .on_press(item.on_press.clone())
        .tag(snora_interaction::Role::SideBarItem, index)
        .padding(0);

        col = col.push(btn);
    }

    // No `.width(..)`: `Shrink` is the container's default, and the rail
    // taking its width from its names is the point. `max_width` both caps
    // the rail and gives the names a line to wrap at.
    //
    // `Padding::from([a, b])` is `[vertical, horizontal]` in iced_core
    // 0.14 (top/bottom = a, left/right = b).
    container(col)
        .max_width(LABELLED_RAIL_MAX_WIDTH)
        .height(Length::Fill)
        .padding(Padding::from([
            geometry.vertical_padding,
            RAIL_HORIZONTAL_PADDING,
        ]))
        .into()
}

pub(crate) fn sidebar_button_style(
    theme: &iced::Theme,
    status: button::Status,
    is_active: bool,
    radius: f32,
) -> button::Style {
    use iced::{Background, Border, Color, Shadow};

    let ep = theme.extended_palette();
    let base_bg = if is_active {
        Some(Background::Color(sidebar_active_color(theme)))
    } else {
        match status {
            button::Status::Hovered => Some(Background::Color(ep.background.weak.color)),
            _ => None,
        }
    };

    // Corrected (RFC-085 F-14): the active state used
    // `background.base.text` — calibrated against `background.base`,
    // not against the highlight it was actually painted on
    // (`sidebar_active_color`, a `primary` tier). Measured: 2.01:1
    // (design light), 2.13:1 (design dark), 1.59:1 (high_contrast_light),
    // **1.51:1 (high_contrast_dark)** — the low-vision preset scoring
    // worst, on its own a release blocker (RFC-085 Q-4). `primary.strong`
    // is the tier `sidebar_active_color` now uses; `.text` is iced's own
    // guaranteed-readable pairing for it. The inactive states are
    // unaffected — `background.base.text` already clears the floor
    // against both the page background and `background.weak.color`.
    //
    // **Corrected again (RFC-114 Unit 3b).** "iced's own
    // guaranteed-readable pairing" is not guaranteed at AA: measured
    // from drawn pixels, `primary.strong.text` on `primary.strong.color`
    // comes to 3.93:1 on Solarized Light and Solarized Dark, 4.23 on
    // Tokyo Night Light and 4.27 on Catppuccin Latte. That cleared the
    // 3.0 non-text floor while the active item held only an icon. RFC-114
    // puts the item's **name** there, which is text at 4.5 — so the same
    // rule the toast's text already uses applies here:
    // [`crate::style::readable_text`] keeps the theme's pair wherever it
    // holds and falls back to the stronger pole where it does not.
    //
    // This is one rule for the active item in **every** form, the icon
    // rail included: the icon is the item's only content there, and a
    // glyph a reader cannot make out is no better than a name they
    // cannot read.
    let text_color = if is_active {
        crate::style::readable_text(sidebar_active_color(theme), ep.primary.strong.text)
    } else {
        ep.background.base.text
    };

    button::Style {
        background: base_bg,
        text_color,
        border: Border {
            radius: radius.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: true,
    }
}
