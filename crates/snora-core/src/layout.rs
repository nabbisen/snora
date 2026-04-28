//! The application skeleton — [`AppLayout`].
//!
//! `AppLayout` is the **only** shape an engine consumes. It is a plain
//! data structure with public fields plus a builder-style API. Every slot
//! is a `Node` of the same generic type — when rendered with snora, that
//! binds to `iced::Element<'a, Message>`, so all four layout slots accept
//! any iced element regardless of how the application organized its view
//! code.
//!
//! # Filling slots
//!
//! `AppLayout::new(body)` is the minimum — just a body element. Every
//! other slot has a sensible default and is set via a chainable method:
//!
//! ```ignore
//! let layout = AppLayout::new(my_body())
//!     .header(my_header())
//!     .side_bar(my_sidebar())
//!     .footer(my_footer())
//!     .direction(LayoutDirection::Rtl)
//!     .on_close_menus(Message::CloseMenus)
//!     .on_close_modals(Message::CloseModals);
//! ```
//!
//! # Why no `PageContract`?
//!
//! Earlier drafts of snora required layout slots to implement a
//! `PageContract` trait that declared `view()`, `dialog()`, `toasts()`,
//! and close hooks. The engine never actually consumed the non-`view`
//! methods, so users were forced to plumb them manually anyway, and the
//! trait's associated-type machinery forced all four slots to share a
//! single type — a painful tax that produced the `Section` enum pattern.
//!
//! v0.4 drops the trait. Every slot is a `Node` value of the same generic
//! type — in practice, `iced::Element<'a, Message>`. Because any function
//! can return an `Element`, each slot can be built by a different piece of
//! application code without any wrapping trait or enum, and all overlay /
//! close state lives as plain fields here.

use crate::{
    direction::LayoutDirection,
    overlay::{BottomSheet, Dialog},
    toast::{Toast, ToastPosition},
};

/// The complete declarative description of what should be on screen.
///
/// Type parameters:
/// * `Node` — the element type your engine consumes. With the `snora`
///   engine, this is `iced::Element<'a, Message>`.
/// * `Message` — your application's top-level message type.
///
/// Fields are intentionally `pub` so that direct struct literal syntax is
/// available for advanced callers. The `new` + chainable setters are the
/// *canonical* path; direct construction is a power-user escape hatch.
pub struct AppLayout<Node, Message>
where
    Message: Clone,
{
    // -----------------------------------------------------------------
    // Primary skeleton slots.
    // -----------------------------------------------------------------
    /// The main content area. Required.
    pub body: Node,
    /// Top header bar (typically built with [`crate::menu::Menu`] entries).
    pub header: Option<Node>,
    /// Vertical navigation rail. Renders on the start edge by default and
    /// flips with [`Self::direction`].
    pub side_bar: Option<Node>,
    /// Status bar at the bottom of the window.
    pub footer: Option<Node>,

    // -----------------------------------------------------------------
    // Light-weight overlays (menus).
    //
    // These render above the skeleton but below the modal dim layer.
    // Outside-click dismissal is wired via `on_close_menus`.
    // -----------------------------------------------------------------
    /// Optional header-attached dropdown (e.g. File menu's item list).
    /// When `Some`, the engine installs a transparent backdrop that
    /// dispatches [`Self::on_close_menus`] on any outside click.
    pub header_menu: Option<Node>,
    /// Optional floating context menu (right-click menu). Same backdrop
    /// behavior as `header_menu`.
    pub context_menu: Option<Node>,

    // -----------------------------------------------------------------
    // Modal overlays.
    //
    // These render above everything except toasts. The engine paints a
    // dimmed backdrop behind them (when any modal is present) and wires
    // outside-click to `on_close_modals`.
    // -----------------------------------------------------------------
    /// A centered modal card.
    pub dialog: Option<Dialog<Node, Message>>,
    /// A modal drawer anchored to the bottom of the window.
    pub bottom_sheet: Option<BottomSheet<Node, Message>>,

    // -----------------------------------------------------------------
    // Toasts.
    //
    // Always rendered at the top of the z-stack so they are visible even
    // when a modal is open. The anchor corner is controlled by
    // `toast_position`; horizontal mirroring under RTL is automatic
    // because positions are expressed in logical (Start / End) terms.
    // -----------------------------------------------------------------
    /// The toast queue, owned by the application. snora does not mutate
    /// this slice — see `snora::toast::sweep_expired` for in-place
    /// expiration handling.
    pub toasts: Vec<Toast<Message>>,

    /// Anchor corner of the toast stack. Defaults to
    /// [`ToastPosition::TopEnd`] (top-right under LTR, top-left under RTL).
    pub toast_position: ToastPosition,

    // -----------------------------------------------------------------
    // Global configuration.
    // -----------------------------------------------------------------
    /// Reading direction. Drives sidebar side, header start/end ordering,
    /// and toast anchor mirroring (when the position is `*Start` or `*End`).
    pub direction: LayoutDirection,

    // -----------------------------------------------------------------
    // Close sinks.
    //
    // Single source of truth for outside-click dismissal. Individual
    // overlay values do *not* carry their own close messages — the
    // engine dispatches through these two channels.
    // -----------------------------------------------------------------
    /// Dispatched when the user clicks outside an open menu (header or
    /// context). If `None`, menus still render but the click-outside-to-
    /// close backdrop is not installed — the application must then
    /// provide explicit close buttons inside its menu content.
    pub on_close_menus: Option<Message>,

    /// Dispatched when the user clicks the dim backdrop of a dialog or
    /// bottom sheet. Semantics mirror [`Self::on_close_menus`].
    pub on_close_modals: Option<Message>,
}

impl<Node, Message> AppLayout<Node, Message>
where
    Message: Clone,
{
    /// Start a layout with only a body. All other slots default to their
    /// empty / `None` states.
    pub fn new(body: Node) -> Self {
        Self {
            body,
            header: None,
            side_bar: None,
            footer: None,
            header_menu: None,
            context_menu: None,
            dialog: None,
            bottom_sheet: None,
            toasts: Vec::new(),
            toast_position: ToastPosition::default(),
            direction: LayoutDirection::default(),
            on_close_menus: None,
            on_close_modals: None,
        }
    }

    // ---------------------------------------------------------------
    // Skeleton slot setters.
    // ---------------------------------------------------------------
    /// Set the header element.
    #[must_use]
    pub fn header(mut self, header: Node) -> Self {
        self.header = Some(header);
        self
    }

    /// Set the sidebar element. Renders on the start edge by default.
    #[must_use]
    pub fn side_bar(mut self, side_bar: Node) -> Self {
        self.side_bar = Some(side_bar);
        self
    }

    /// Set the footer element.
    #[must_use]
    pub fn footer(mut self, footer: Node) -> Self {
        self.footer = Some(footer);
        self
    }

    // ---------------------------------------------------------------
    // Overlay setters.
    // ---------------------------------------------------------------
    /// Set the header dropdown menu. Setting any value (typically an
    /// empty `Space`) opts the application into the click-outside
    /// backdrop; the actual dropdown items are drawn inline by the
    /// header widget.
    #[must_use]
    pub fn header_menu(mut self, menu: Node) -> Self {
        self.header_menu = Some(menu);
        self
    }

    /// Set the floating context menu. Pass a positioned element.
    #[must_use]
    pub fn context_menu(mut self, menu: Node) -> Self {
        self.context_menu = Some(menu);
        self
    }

    /// Show a modal dialog.
    #[must_use]
    pub fn dialog(mut self, dialog: Dialog<Node, Message>) -> Self {
        self.dialog = Some(dialog);
        self
    }

    /// Show a modal bottom sheet.
    #[must_use]
    pub fn bottom_sheet(mut self, sheet: BottomSheet<Node, Message>) -> Self {
        self.bottom_sheet = Some(sheet);
        self
    }

    /// Replace the toast queue. Each frame the application typically passes
    /// `state.toasts.clone()` here; snora does not mutate the slice. See
    /// `snora::toast::subscription` and `snora::toast::sweep_expired` for
    /// framework-managed lifetime handling.
    #[must_use]
    pub fn toasts(mut self, toasts: Vec<Toast<Message>>) -> Self {
        self.toasts = toasts;
        self
    }

    /// Override the toast anchor corner. Defaults to
    /// [`ToastPosition::TopEnd`].
    #[must_use]
    pub fn toast_position(mut self, position: ToastPosition) -> Self {
        self.toast_position = position;
        self
    }

    // ---------------------------------------------------------------
    // Configuration setters.
    // ---------------------------------------------------------------
    /// Override the reading direction.
    #[must_use]
    pub fn direction(mut self, direction: LayoutDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Wire the click-outside-to-close handler for header / context menus.
    #[must_use]
    pub fn on_close_menus(mut self, msg: Message) -> Self {
        self.on_close_menus = Some(msg);
        self
    }

    /// Wire the click-outside-to-close handler for dialog / bottom sheet.
    #[must_use]
    pub fn on_close_modals(mut self, msg: Message) -> Self {
        self.on_close_modals = Some(msg);
        self
    }
}


