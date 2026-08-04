//! The engine: turn an [`AppLayout`] into an [`iced::Element`].
//!
//! This is the only place in snora where composition of layers, backdrops,
//! and z-order happens. Nothing else in the framework mutates layer state,
//! and application code never touches [`iced::widget::stack`] directly when
//! using snora.
//!
//! # Layer order (bottom to top)
//!
//! ```text
//! 0. skeleton          — header + (side_bar | body) + footer
//! 1. menu backdrop     — transparent mouse_area, dispatches on_close_menus
//! 2. header_menu       — dropdown under the header bar
//! 3. context_menu      — floating menu at click point
//! 4. modal backdrop    — 40%-dim mouse_area, dispatches on_close_modals
//! 5. dialog            — centered card
//! 6. sheet             — edge-anchored panel, size per SheetSize
//! 7. toasts            — stacked at the configured ToastPosition (RTL-aware),
//!                        newest toast closest to the anchor edge
//! ```
//!
//! This order is part of the framework contract. See the
//! [overlay interaction semantics](https://docs.snora.dev/reference/overlay-interaction-semantics.html)
//! reference page for the normative behavioral rules.
//!
//! Layers 1–6 are conditional on the corresponding `AppLayout` fields
//! being populated. Layer 7 is always evaluated but emits nothing when
//! the toast queue is empty.
//!
//! # Graceful degradation
//!
//! When overlay content is present but the matching `on_close_*` handler
//! is `None`, the engine **still renders the content**. It simply omits
//! the click-outside backdrop. This lets applications opt into explicit
//! close buttons instead of click-outside-to-close without silently losing
//! their overlays.

use iced::{
    Color, Element, Length,
    widget::{column, container, mouse_area, row, space, stack},
};

use snora_core::{AppLayout, LayoutDirection};

use crate::identifiers;
use crate::overlay::dialog::DialogCardStyle;
use crate::overlay::{dialog::render_dialog, sheet::render_sheet};
use crate::toast::render_toasts;

/// Style parameters [`render_with_style`] composes into the layer stack,
/// letting [`render`] (unstyled) and [`crate::design::render::render`]
/// (`design`-gated) share one implementation (RFC-039) instead of two
/// that could drift.
pub(crate) struct ChromeStyle {
    /// Fill for the modal-backdrop layer (layer 4). Includes alpha.
    pub(crate) dim_color: Color,
    /// Card wrapper for the dialog layer (layer 5). `None` renders
    /// `dialog.content` centered with no card at all.
    pub(crate) dialog_card: Option<DialogCardStyle>,
}

impl ChromeStyle {
    /// Today's literal, unmodified: opaque black at 40% alpha, no card.
    /// This is [`render`]'s exact pre-RFC-039 behavior — the gating
    /// invariant depends on this value never changing.
    fn unstyled() -> Self {
        Self {
            dim_color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            dialog_card: None,
        }
    }
}

/// Compile an [`AppLayout`] into an iced [`Element`].
///
/// The layout is consumed by value. All references inside `layout`
/// (including inside toasts and overlay content) are preserved through
/// the output element's lifetime `'a`.
pub fn render<'a, Message>(layout: AppLayout<Element<'a, Message>, Message>) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    render_with_style(layout, &ChromeStyle::unstyled())
}

/// The shared implementation behind [`render`] and
/// [`crate::design::render::render`]. `style` carries every value that
/// differs between the unstyled and `design`-gated paths; the z-stack
/// composition itself — layer order, conditions, and backdrop wiring —
/// is written exactly once.
pub(crate) fn render_with_style<'a, Message>(
    layout: AppLayout<Element<'a, Message>, Message>,
    style: &ChromeStyle,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    // -----------------------------------------------------------------
    // Layer 0 — skeleton.
    // -----------------------------------------------------------------
    let skeleton = build_skeleton(
        layout.direction,
        layout.header,
        layout.side_bar,
        layout.body,
        layout.footer,
    );

    let mut layers = stack![skeleton];

    // -----------------------------------------------------------------
    // Layers 1-3 — light overlays (menus).
    // -----------------------------------------------------------------
    let has_menu = layout.header_menu.is_some() || layout.context_menu.is_some();

    if has_menu {
        if let Some(on_close) = layout.on_close_menus {
            layers = layers.push(transparent_backdrop(on_close));
        }
        if let Some(header_menu) = layout.header_menu {
            layers = layers.push(header_menu);
        }
        if let Some(context_menu) = layout.context_menu {
            layers = layers.push(context_menu);
        }
    }

    // -----------------------------------------------------------------
    // Layers 4-6 — modals.
    // -----------------------------------------------------------------
    let has_modal = layout.dialog.is_some() || layout.sheet.is_some();

    if has_modal {
        if let Some(on_close) = layout.on_close_modals {
            layers = layers.push(dim_backdrop(on_close, style.dim_color));
        } else {
            // No click-to-close requested — still paint the dim to signal
            // "this is modal", but don't capture clicks.
            layers = layers.push(dim_without_capture(style.dim_color));
        }

        if let Some(dialog) = layout.dialog {
            layers = layers.push(render_dialog(dialog, style.dialog_card.as_ref()));
        }
        if let Some(sheet) = layout.sheet {
            layers = layers.push(render_sheet(sheet, layout.direction));
        }
    }

    // -----------------------------------------------------------------
    // Layer 7 — toasts.
    // -----------------------------------------------------------------
    if let Some(toast_layer) = render_toasts(layout.toasts, layout.toast_position, layout.direction)
    {
        layers = layers.push(toast_layer);
    }

    layers.into()
}

// -----------------------------------------------------------------------
// Skeleton composition.
// -----------------------------------------------------------------------

fn build_skeleton<'a, Message>(
    direction: LayoutDirection,
    header: Option<Element<'a, Message>>,
    side_bar: Option<Element<'a, Message>>,
    body: Element<'a, Message>,
    footer: Option<Element<'a, Message>>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let mut main_col = column![];

    if let Some(header) = header {
        main_col = main_col.push(
            container(header)
                .width(Length::Fill)
                .id(identifiers::HEADER_REGION),
        );
    }

    // Body row: sidebar on the logical start side.
    let body_container = container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .id(identifiers::BODY_REGION);

    let sidebar_region = |sb: Element<'a, Message>| {
        container(sb)
            .height(Length::Fill)
            .id(identifiers::SIDEBAR_REGION)
    };

    let body_row = match (direction, side_bar) {
        (LayoutDirection::Ltr, Some(sb)) => row![sidebar_region(sb), body_container],
        (LayoutDirection::Rtl, Some(sb)) => row![body_container, sidebar_region(sb)],
        (_, None) => row![body_container],
    }
    .width(Length::Fill)
    .height(Length::Fill);

    main_col = main_col.push(body_row);

    if let Some(footer) = footer {
        main_col = main_col.push(
            container(footer)
                .width(Length::Fill)
                .id(identifiers::FOOTER_REGION),
        );
    }

    container(main_col)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// -----------------------------------------------------------------------
// Backdrops.
// -----------------------------------------------------------------------

/// A full-window, fully transparent click target. Used above the skeleton
/// and below menus so that any click outside an open menu dismisses it.
fn transparent_backdrop<'a, Message>(on_press: Message) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let surface = container(space())
        .width(Length::Fill)
        .height(Length::Fill)
        .id(identifiers::MENU_BACKDROP);
    mouse_area(surface).on_press(on_press).into()
}

/// A full-window dim click target, filled with `dim_color`. Used above
/// menus and below modals so that clicking outside a dialog / sheet
/// dismisses it and signals "this is modal" by dimming the background
/// content.
fn dim_backdrop<'a, Message>(on_press: Message, dim_color: Color) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    use iced::{Background, widget::container::Style};

    let dim = container(space())
        .width(Length::Fill)
        .height(Length::Fill)
        .id(identifiers::MODAL_DIM)
        .style(move |_theme| Style {
            background: Some(Background::Color(dim_color)),
            ..Default::default()
        });
    mouse_area(dim).on_press(on_press).into()
}

/// Same visual as [`dim_backdrop`] but without the click sink — used when
/// the application chose not to provide `on_close_modals`.
fn dim_without_capture<'a, Message>(dim_color: Color) -> Element<'a, Message>
where
    Message: 'a,
{
    use iced::{Background, widget::container::Style};

    container(space())
        .width(Length::Fill)
        .height(Length::Fill)
        .id(identifiers::MODAL_DIM)
        .style(move |_theme| Style {
            background: Some(Background::Color(dim_color)),
            ..Default::default()
        })
        .into()
}
