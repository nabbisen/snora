//! The engine: turn an [`AppLayout`] into an [`iced::Element`].
//!
//! This is the only place in snora where composition of layers, backdrops,
//! and z-order happens. Nothing else in the framework mutates layer state,
//! and application code never touches [`iced::widget::stack()`] directly when
//! using snora.
//!
//! # Layer order (bottom to top)
//!
//! ```text
//! 0. skeleton          — header + (side_bar | body) + footer
//! 1. menu backdrop     — transparent mouse_area, dispatches on_close_menus
//! 2. header_menu       — the application's own menu content under the header
//!                        (`app_header`'s dropdown is not a layer: it floats over
//!                        the page from its trigger, and covers the page while open)
//! 3. context_menu      — floating menu at click point
//! 4. modal backdrop    — 40%-dim mouse_area, dispatches on_close_modals
//! 5. dialog            — centered; token-styled card via `design::render`
//! 6. sheet             — edge-anchored panel, size per SheetSize
//! 7. toasts            — stacked at the configured ToastPosition (RTL-aware),
//!                        newest toast closest to the anchor edge
//! ```
//!
//! This order is part of the framework contract. See the
//! [overlay interaction semantics](https://nabbisen.github.io/snora/reference/overlay-interaction-semantics.html)
//! reference page for the normative behavioral rules.
//!
//! Layers 1–6 are conditional on the corresponding `AppLayout` fields
//! being populated. Layer 7 is always evaluated but emits nothing when
//! the toast queue is empty.
//!
//! # Graceful degradation
//!
//! When overlay content is present but the matching `on_close_*` handler
//! is `None`, the engine **still renders the content**. What happens to the
//! backdrop differs by kind:
//!
//! * **Menu backdrop** — omitted entirely; no click sink means no
//!   transparent outside-click layer is installed at all.
//! * **Modal dim** — never omitted. It still paints and still blocks
//!   pointer input from reaching content beneath it (RFC-084); with no
//!   sink it simply produces no dismiss message when clicked or
//!   scrolled, rather than reaching through to whatever is underneath.
//!
//! Either way, this lets applications opt into explicit close buttons
//! instead of click-outside-to-close without silently losing their
//! overlays — or, for modals, without silently losing pointer
//! containment either.

use iced::{
    Color, Element, Length, Theme,
    widget::{column, container, mouse_area, row, space, stack},
};

use iced::keyboard::Modifiers;
use snora_core::{AppLayout, LayoutDirection, Platform, ShortcutAction, Shortcuts, TargetRole};
use snora_interaction::{Inert, KeyMatch, OnEscape, Role, Route, Router, Tag};

use crate::identifiers;
use crate::overlay::dialog::DialogCardStyle;
use crate::overlay::{dialog::render_dialog, sheet::render_sheet};
use crate::toast::{ToastStyle, render_toasts};

/// Style parameters [`render_with_style`] composes into the layer stack,
/// letting [`render`] (unstyled) and [`crate::design::render::render`]
/// (`design`-gated) share one implementation (RFC-039) instead of two
/// that could drift.
pub(crate) struct ChromeStyle {
    /// Fill for the modal-backdrop layer (layer 4). Includes alpha.
    pub(crate) dim: DimFill,
    /// Card wrapper for the dialog layer (layer 5). `None` renders
    /// `dialog.content` centered with no card at all.
    pub(crate) dialog_card: Option<DialogCardStyle>,
    /// How the toast layer (layer 7) sizes and colours itself
    /// (RFC-104). The engine renders toasts itself, so without this the
    /// `design` path could not reach them at all: `ChromeStyle` carried
    /// only the dim and the dialog card.
    pub(crate) toast: ToastStyle,
}

/// Alpha for the unstyled modal dim.
///
/// The same number as [`snora_design::surfaces::DIM_ALPHA`], which the
/// `design` path uses, and deliberately **not** shared with it:
/// `snora-design` is an optional dependency of this crate, so the
/// unstyled path cannot reach it, and that crate's surface is frozen
/// (RFC-036) so widening it for one constant is the worse trade. This
/// is RFC-106's `CHIP_DISABLED_ALPHA` pattern — reproduce the value
/// with a comment naming its twin, rather than couple two crates for a
/// float.
pub(crate) const UNSTYLED_DIM_ALPHA: f32 = 0.44;

/// How the modal backdrop decides its colour.
///
/// The `design` path knows its colour before anything is drawn: it
/// derives it from the token palette. The unstyled path cannot, because
/// it has no tokens — only the `iced::Theme` the application supplies,
/// which it does not see until paint time. The backdrop's style closure
/// already receives that theme, so this enum defers the decision to it
/// rather than adding a parameter to carry a theme that is already
/// there.
#[derive(Debug, Clone, Copy)]
pub(crate) enum DimFill {
    /// A colour fixed in advance, as `design` supplies from its tokens.
    ///
    /// **`design`-gated, because only that path can produce one.** The
    /// sole constructor is `design::render::chrome_style`, which is
    /// itself behind the feature; without it the variant has no way to
    /// be built, and saying so in the type is what lets the compiler
    /// agree rather than warning that it is never constructed
    /// (RFC-112 R-5). An `allow(dead_code)` would have silenced a true
    /// statement instead of making it false.
    #[cfg(feature = "design")]
    Fixed(Color),
    /// Derived from the theme at paint time: the pole opposite the
    /// page, at [`UNSTYLED_DIM_ALPHA`].
    FromTheme,
}

impl DimFill {
    /// The colour to paint on `theme`.
    pub(crate) fn resolve(&self, theme: &Theme) -> Color {
        match self {
            #[cfg(feature = "design")]
            DimFill::Fixed(color) => *color,
            // The pole opposite the page, which is the only choice
            // that dims both a white page and a near-black one. iced's
            // `is_dark` is the same OKLCH-lightness classification
            // `snora_design::surfaces::is_dark` reimplements, so the
            // two paths agree on every background.
            DimFill::FromTheme => {
                let base = if theme.extended_palette().is_dark {
                    Color::WHITE
                } else {
                    Color::BLACK
                };
                Color {
                    a: UNSTYLED_DIM_ALPHA,
                    ..base
                }
            }
        }
    }
}

impl ChromeStyle {
    /// No card, and a dim that follows the theme (RFC-110).
    ///
    /// **The dim is no longer a literal.** It was `rgba(0, 0, 0, 0.4)`
    /// under a comment saying "the gating invariant depends on this
    /// value never changing", and that comment was false: no test
    /// pinned the value, and on a dark theme black over a nearly-black
    /// page measured 1.02–1.33:1 against the undimmed page — a dim that
    /// did not dim. The unstyled path now picks the pole opposite the
    /// page, the same rule `snora_design::surfaces::modal_dim` has used
    /// since RFC-065.
    ///
    /// The invariant that does still bind is **composition** — which
    /// layer the backdrop occupies, what it blocks, and what dismisses
    /// it — and that is asserted by `tests/render_semantics.rs`, not by
    /// any particular colour.
    fn unstyled() -> Self {
        Self {
            dim: DimFill::FromTheme,
            dialog_card: None,
            // Inherit the host's text size; derive colours from the theme.
            toast: ToastStyle::default(),
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
    let has_menu = layout.header_menu.is_some() || layout.context_menu.is_some();
    let has_modal = layout.dialog.is_some() || layout.sheet.is_some();

    // The keyboard does what the pointer can (RFC-119): a layer the pointer
    // cannot reach is out of the keyboard's reach too.
    //
    // - The skeleton is covered by a menu's outside-click backdrop (a click
    //   there closes the menu and presses nothing) and by a modal. A menu
    //   **without** a backdrop leaves the page pressable, and so reachable.
    // - A menu, and its backdrop, are covered by a modal.
    // - Toasts are never covered: the pointer reaches them (Law 6).
    //
    // **Every `Inert` is built whether or not it is on**, so that opening a
    // modal does not reshape the tree and reset the page (R-5).
    let menu_backdrop_up = has_menu && layout.on_close_menus.is_some();
    let skeleton_inert = menu_backdrop_up || has_modal;
    let menus_inert = has_modal;

    let skeleton = build_skeleton(
        layout.direction,
        layout.header,
        layout.side_bar,
        layout.body,
        layout.footer,
    );

    // The shortcuts router (RFC-116 R-2) wraps the whole skeleton, **inside** its
    // `Inert`: a router in the page is silent under every overlay that blocks the
    // pointer, through the mechanism the page already has. It is built whether or
    // not the application asked for routes (an empty router answers nothing), so
    // setting or clearing `shortcuts` does not reshape the tree and reset the page.
    let routes = layout
        .shortcuts
        .as_ref()
        .map(|s| router_routes(s, layout.direction))
        .unwrap_or_default();
    let skeleton = Router::new(skeleton, routes);

    // A menu open in the page (found in the tree, not told by a slot) covers it too,
    // unless a modal does (RFC-124). Escape closes it with the application's escape
    // message for menus if it set one, else the close message (Law 7).
    let menu_cover = if has_modal {
        None
    } else {
        layout.on_close_menus.clone()
    };
    let mut layers = stack![
        Inert::new(skeleton, skeleton_inert).beneath_menus(menu_cover, layout.escape.menu.clone())
    ];

    // -----------------------------------------------------------------
    // Layers 1-3 — light overlays (menus).
    // -----------------------------------------------------------------
    if has_menu {
        if let Some(on_close) = layout.on_close_menus {
            layers = layers.push(Inert::new(transparent_backdrop(on_close), menus_inert));
        }
        // Escape on a menu's own layer (RFC-116 R-2), inside the `Inert`, so it is
        // silent while a modal covers the menu. The layer is built either way.
        if let Some(header_menu) = layout.header_menu {
            layers = layers.push(Inert::new(
                OnEscape::maybe(header_menu, layout.escape.menu.clone()),
                menus_inert,
            ));
        }
        if let Some(context_menu) = layout.context_menu {
            layers = layers.push(Inert::new(
                OnEscape::maybe(context_menu, layout.escape.menu.clone()),
                menus_inert,
            ));
        }
    }

    // -----------------------------------------------------------------
    // Layers 4-6 — modals.
    // -----------------------------------------------------------------
    if has_modal {
        if let Some(on_close) = layout.on_close_modals {
            layers = layers.push(dim_backdrop(on_close, style.dim));
        } else {
            // No click-to-close requested — still paint the dim to signal
            // "this is modal", but don't capture clicks.
            layers = layers.push(dim_without_capture(style.dim));
        }

        // A dialog and a sheet are layers of their own for where focus goes
        // after a removal (RFC-120): the group is never wider than the layer,
        // so a control removed in a dialog never hands focus to the page.
        // Never inert: nothing covers them.
        // Escape on the overlay's own layer (RFC-116 R-2): the overlay's escape
        // message if one is set; otherwise nothing here, and the backdrop's own
        // answer (its click-outside message) is reached.
        if let Some(dialog) = layout.dialog {
            layers = layers.push(Inert::new(
                OnEscape::maybe(
                    render_dialog(dialog, style.dialog_card.as_ref()),
                    layout.escape.modal.clone(),
                ),
                false,
            ));
        }
        if let Some(sheet) = layout.sheet {
            layers = layers.push(Inert::new(
                OnEscape::maybe(
                    render_sheet(sheet, layout.direction),
                    layout.escape.modal.clone(),
                ),
                false,
            ));
        }
    }

    // -----------------------------------------------------------------
    // Layer 7 — toasts.
    // -----------------------------------------------------------------
    if let Some(toast_layer) = render_toasts(
        layout.toasts,
        layout.toast_position,
        layout.direction,
        style.toast,
        layout.on_toast_attention,
    ) {
        layers = layers.push(toast_layer);
    }

    // PROTOTYPE (RFC-123 Unit 0): arrows in an open menu.
    snora_interaction::MenuKeys::new(layers).into()
}

// -----------------------------------------------------------------------
// The shortcuts router's keys (RFC-116 R-2).
// -----------------------------------------------------------------------

/// The keys of an action on a platform. **Every binding has a modifier**, so
/// WCAG 2.1.4 (single-character shortcuts) does not apply.
///
/// Digits and punctuation are matched on the physical key, named keys on the
/// logical one (`snora_interaction::KeyMatch`).
fn bindings(action: ShortcutAction, platform: Platform) -> Vec<(KeyMatch, Modifiers)> {
    use iced::keyboard::key::{Code, Named};
    let mac = platform == Platform::MacOs;
    // The key that "commands": Cmd on macOS, Ctrl elsewhere.
    let command = if mac {
        Modifiers::LOGO
    } else {
        Modifiers::CTRL
    };
    match action {
        ShortcutAction::NextTab if mac => vec![
            (KeyMatch::Named(Named::Tab), Modifiers::CTRL),
            (
                KeyMatch::Physical(Code::BracketRight),
                Modifiers::LOGO | Modifiers::SHIFT,
            ),
        ],
        ShortcutAction::PreviousTab if mac => vec![
            (
                KeyMatch::Named(Named::Tab),
                Modifiers::CTRL | Modifiers::SHIFT,
            ),
            (
                KeyMatch::Physical(Code::BracketLeft),
                Modifiers::LOGO | Modifiers::SHIFT,
            ),
        ],
        ShortcutAction::NextTab => vec![
            (KeyMatch::Named(Named::Tab), Modifiers::CTRL),
            (KeyMatch::Named(Named::PageDown), Modifiers::CTRL),
        ],
        ShortcutAction::PreviousTab => vec![
            (
                KeyMatch::Named(Named::Tab),
                Modifiers::CTRL | Modifiers::SHIFT,
            ),
            (KeyMatch::Named(Named::PageUp), Modifiers::CTRL),
        ],
        ShortcutAction::Parent => vec![(
            KeyMatch::Named(Named::ArrowUp),
            if mac { Modifiers::LOGO } else { Modifiers::ALT },
        )],
        ShortcutAction::Digit(n) => vec![(KeyMatch::Digit(n), command)],
        // An action this version does not know has no keys.
        _ => Vec::new(),
    }
}

/// The router's routes for a layout: each action's keys on the platform, each
/// key carrying the message a click on the target publishes and the target's tag.
fn router_routes<Message: Clone>(
    shortcuts: &Shortcuts<Message>,
    direction: LayoutDirection,
) -> Vec<Route<Message>> {
    let platform = shortcuts.platform_in_use();
    let mut routes = Vec::new();
    for route in shortcuts.routes(direction) {
        let role = match route.role {
            TargetRole::SideBarItem => Role::SideBarItem,
            TargetRole::Tab => Role::Tab,
            TargetRole::Crumb => Role::Crumb,
            // A role this version does not know has no control to find.
            _ => continue,
        };
        for (key, modifiers) in bindings(route.action, platform) {
            routes.push(Route {
                key,
                modifiers,
                message: route.message.clone(),
                target: Tag::new(role, route.index),
            });
        }
    }
    routes
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
/// **A press and Escape publish `on_press`; a wheel over it does not** (RFC-125
/// R-2): the covered layer beneath captures the wheel, without publishing.
fn transparent_backdrop<'a, Message>(on_press: Message) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let surface = container(space())
        .width(Length::Fill)
        .height(Length::Fill)
        .id(identifiers::MENU_BACKDROP);
    // Escape closes the menu as a click outside does (RFC-119 R-4): the
    // keyboard cannot reach "outside", so without this a keyboard user is
    // trapped in the menu.
    OnEscape::new(mouse_area(surface).on_press(on_press.clone()), on_press).into()
}

/// A full-window dim click target, filled with `dim_color`. Used above
/// menus and below modals so that clicking outside a dialog / sheet
/// dismisses it and signals "this is modal" by dimming the background
/// content.
///
/// **A press on it, and Escape, publish `on_press`; a wheel over it does not**
/// (RFC-125 R-2). RFC-084 F-03 asked that the dim block the wheel, and it was
/// built by publishing the close message on every wheel event, so a stray scroll
/// closed a dialog, once per event, with whatever the reader had typed. It is now
/// the covered layer beneath that captures the wheel (`Inert`), without
/// publishing: the dim has no `on_scroll`.
fn dim_backdrop<'a, Message>(on_press: Message, fill: DimFill) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    use iced::{Background, widget::container::Style};

    let dim = container(space())
        .width(Length::Fill)
        .height(Length::Fill)
        .id(identifiers::MODAL_DIM)
        .style(move |theme| Style {
            background: Some(Background::Color(fill.resolve(theme))),
            ..Default::default()
        });
    // Escape closes the modal as a click on the dim does (RFC-119 R-4).
    OnEscape::new(mouse_area(dim).on_press(on_press.clone()), on_press).into()
}

/// Same visual as [`dim_backdrop`] but without the click sink — used when
/// the application chose not to provide `on_close_modals`.
///
/// Wrapped in [`opaque`] (RFC-084 F-02 / Q-1) so it still blocks pointer
/// input from reaching content beneath it, matching Law 8's unconditional
/// "Pointer blocking — yes" — blocking must not depend on whether a
/// dismiss message was provided. This also blocks wheel-scroll (F-03),
/// measured rather than assumed — `opaque`'s own `update` method only
/// checks for a mouse button press, but `iced_widget::Stack`'s dispatch
/// separately consults every layer's `mouse_interaction()` after each
/// event and stops passing the cursor to layers beneath one that reports
/// non-`None`, which `opaque` does unconditionally whenever hovered. See
/// `modal_with_no_close_sink_also_blocks_wheel_scroll` in
/// `crates/snora/tests/render_semantics.rs` for the measurement, including
/// the record of the wrong first assumption it corrected.
fn dim_without_capture<'a, Message>(fill: DimFill) -> Element<'a, Message>
where
    Message: 'a,
{
    use iced::widget::opaque;
    use iced::{Background, widget::container::Style};

    opaque(
        container(space())
            .width(Length::Fill)
            .height(Length::Fill)
            .id(identifiers::MODAL_DIM)
            .style(move |theme| Style {
                background: Some(Background::Color(fill.resolve(theme))),
                ..Default::default()
            }),
    )
}

#[cfg(test)]
mod tests;
