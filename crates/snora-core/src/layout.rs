//! The application skeleton — [`AppLayout`].
//!
//! `AppLayout` is the **only** shape an engine consumes. It is a plain
//! data structure with `pub` fields plus a builder-style API. Every slot
//! is a `Node` of the same generic type — when rendered with snora, that
//! binds to `iced::Element<'a, Message>`, so all four layout slots accept
//! any iced element regardless of how the application organized its view
//! code.
//!
//! # Canonical construction
//!
//! `AppLayout::new(body)` is the minimum — just a body element. Every
//! other slot has a sensible default and is set via a chainable method:
//!
//! ```rust,no_run
//! use snora_core::{AppLayout, LayoutDirection};
//!
//! #[derive(Clone)]
//! enum Message { CloseMenus, CloseModals }
//!
//! fn my_body() -> () {}
//! fn my_header() -> () {}
//! fn my_sidebar() -> () {}
//! fn my_footer() -> () {}
//!
//! let layout = AppLayout::new(my_body())
//!     .header(my_header())
//!     .side_bar(my_sidebar())
//!     .footer(my_footer())
//!     .direction(LayoutDirection::Rtl)
//!     .on_close_menus(Message::CloseMenus)
//!     .on_close_modals(Message::CloseModals);
//! ```
//!
//! The struct is `#[non_exhaustive]` so future top-level surfaces can be
//! added without breaking downstream callers. Fields remain `pub` for
//! readability; builder methods are the stable construction contract.
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
    overlay::{Dialog, Sheet},
    shortcuts::{Escape, Shortcuts},
    toast::{Toast, ToastAttention, ToastPosition},
};

/// The complete declarative description of what should be on screen.
///
/// Type parameters:
/// * `Node` — the element type your engine consumes. With the `snora`
///   engine, this is `iced::Element<'a, Message>`.
/// * `Message` — your application's top-level message type.
///
/// # Canonical construction
///
/// Use [`AppLayout::new`] plus chainable builder methods. This is the
/// stable, long-term construction path:
///
/// ```rust,no_run
/// use snora_core::AppLayout;
///
/// #[derive(Clone)]
/// enum Message { CloseMenus, CloseModals }
///
/// let body: () = ();
/// let header: () = ();
/// let sidebar: () = ();
///
/// let layout = AppLayout::new(body)
///     .header(header)
///     .side_bar(sidebar)
///     .on_close_menus(Message::CloseMenus)
///     .on_close_modals(Message::CloseModals);
/// ```
///
/// Fields are `pub` for readability and in-crate access. Direct struct
/// literal construction from *outside* `snora-core` is not supported
/// (the struct is `#[non_exhaustive]`) so that future top-level surfaces
/// can be added as additive changes. Any new field ships with a
/// corresponding `#[must_use]` builder method.
#[non_exhaustive]
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
    /// Optional menu content of the application's own, under the header. When
    /// `Some`, the engine installs a transparent backdrop that dispatches
    /// [`Self::on_close_menus`] on any outside click.
    ///
    /// **`app_header`'s own dropdown does not need this:** it floats over the page
    /// from its trigger, and the engine learns that it is open from the tree, so
    /// setting [`Self::on_close_menus`] is enough (an empty element here, which
    /// older guides taught, is harmless and no longer needed).
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
    /// Centered modal content. The engine centers `Dialog`'s content and
    /// paints the dim backdrop around it; by default no card chrome is
    /// drawn — the application supplies its own, or opts into a
    /// token-styled card via `snora::design::render` (see the `snora`
    /// crate's overlays guide).
    pub dialog: Option<Dialog<Node, Message>>,
    /// A modal panel anchored to one of the four window edges. The
    /// specific edge is configured on the [`Sheet`] itself.
    pub sheet: Option<Sheet<Node, Message>>,

    // -----------------------------------------------------------------
    // Toasts.
    //
    // Always rendered at the top of the z-stack so they are visible even
    // when a modal is open. The anchor corner is controlled by
    // `toast_position`; horizontal mirroring under RTL is automatic
    // because positions are expressed in logical (Start / End) terms.
    // -----------------------------------------------------------------
    /// The toast queue, owned by the application. snora does not mutate
    /// this slice — see `snora::toast::Holds::sweep` for in-place
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
    ///
    /// **Published on a press, and on Escape. Not on a wheel** (RFC-125): a scroll
    /// over the backdrop is captured and does nothing.
    pub on_close_menus: Option<Message>,

    /// Dispatched when the user clicks the dim backdrop of a dialog or
    /// bottom sheet. Semantics mirror [`Self::on_close_menus`].
    ///
    /// **Published on a press, and on Escape. Not on a wheel** (RFC-125): before 0.60 a
    /// wheel scroll over the dim published it too, once per wheel event, so a scroll that
    /// strayed off a dialog closed it.
    pub on_close_modals: Option<Message>,

    /// `f(toast_id, attention)`: what the reader's pointer and keyboard focus
    /// did to a toast (RFC-120 R-3).
    ///
    /// Wire it to hold a toast's countdown while it is being read, by pointer
    /// or keyboard: pass the message to `snora::toast::Holds::attend`. Leave it
    /// unset and the toast layer carries no tracking.
    pub on_toast_attention: Option<fn(u64, ToastAttention) -> Message>,

    /// The shortcuts router (RFC-116 R-2): keys with a modifier that jump, built
    /// in `view` from the same side bar, tab bar and crumbs the widgets get. The
    /// engine places it around the skeleton **inside the page's `Inert`**, so it
    /// is silent under every overlay that blocks the pointer.
    pub shortcuts: Option<Shortcuts<Message>>,

    /// What Escape does on each overlay (RFC-116 R-2). The rule: the overlay's
    /// escape message if set, otherwise its click-outside message.
    pub escape: Escape<Message>,
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
            sheet: None,
            toasts: Vec::new(),
            toast_position: ToastPosition::default(),
            direction: LayoutDirection::default(),
            on_close_menus: None,
            on_close_modals: None,
            on_toast_attention: None,
            shortcuts: None,
            escape: Escape::default(),
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
    ///
    /// While it is open, **the keyboard reaches only what the pointer can
    /// press** (RFC-119): everything under the dim is out of the keyboard's
    /// reach, and Tab moves between the dialog, any sheet, and the toasts.
    ///
    /// **When a modal closes, focus returns to the control that had it, unless
    /// the page now has a different number of controls, or the application has
    /// focused something itself. Then focus is wherever the application put
    /// it, or nowhere.**
    #[must_use]
    pub fn dialog(mut self, dialog: Dialog<Node, Message>) -> Self {
        self.dialog = Some(dialog);
        self
    }

    /// Show a modal sheet anchored to one of the window edges.
    /// Configure the anchor with `Sheet::at(...)` on the value passed in.
    ///
    /// The keyboard is bound as for [`Self::dialog`], and focus returns to
    /// the control that had it when the sheet closes, on the same terms.
    #[must_use]
    pub fn sheet(mut self, sheet: Sheet<Node, Message>) -> Self {
        self.sheet = Some(sheet);
        self
    }

    /// Replace the toast queue. Each frame the application typically passes
    /// `state.toasts.clone()` here; snora does not mutate the slice. See
    /// `snora::toast::subscription` and `snora::toast::Holds::sweep` for
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

    /// The shortcuts router (RFC-116 R-2): `Ctrl/Cmd` + digits, next and previous
    /// tab, the breadcrumb's parent. **Every route is opt-in**, and every binding
    /// has a modifier. See [`Shortcuts`] for the rules and the bindings.
    ///
    /// ```
    /// use snora_core::{AppLayout, Digits, Shortcuts};
    ///
    /// #[derive(Clone, Debug)]
    /// enum Message {}
    ///
    /// let layout: AppLayout<(), Message> =
    ///     AppLayout::new(()).shortcuts(Shortcuts::new().digits(Digits::Tabs));
    /// assert!(layout.shortcuts.is_some());
    /// ```
    #[must_use]
    pub fn shortcuts(mut self, shortcuts: Shortcuts<Message>) -> Self {
        self.shortcuts = Some(shortcuts);
        self
    }

    /// What Escape does on each overlay, with named fields (RFC-116 R-2). Escape
    /// publishes the overlay's escape message if one is set, and otherwise its
    /// click-outside message; the click-outside message still answers clicks.
    /// Replaced `snora::keyboard::dismiss_on_escape`, removed in 0.60.0. See [`Escape`].
    ///
    /// ```
    /// use snora_core::{AppLayout, Escape};
    ///
    /// #[derive(Clone, Debug)]
    /// enum Message { CloseDialog }
    ///
    /// let layout: AppLayout<(), Message> = AppLayout::new(()).escape(Escape {
    ///     modal: Some(Message::CloseDialog),
    ///     menu: None,
    /// });
    /// assert!(layout.escape.modal.is_some());
    /// ```
    #[must_use]
    pub fn escape(mut self, escape: Escape<Message>) -> Self {
        self.escape = escape;
        self
    }

    /// Hold a toast's countdown while it is being read: by the pointer **or**
    /// by keyboard focus (RFC-120 R-3).
    ///
    /// `f(id, attention)` is emitted with [`ToastAttention::PointerEntered`] and
    /// [`ToastAttention::PointerLeft`] as the pointer crosses a toast, and with
    /// [`ToastAttention::FocusGained`] and [`ToastAttention::FocusLost`] as
    /// keyboard focus arrives on and leaves the toast's controls. Hand each to
    /// `snora::toast::Holds::attend`, which holds the toast while either is
    /// present and restarts its countdown in full when both have gone.
    ///
    /// ```
    /// use snora_core::{AppLayout, ToastAttention};
    ///
    /// #[derive(Clone, Debug)]
    /// enum Message {
    ///     ToastAttention(u64, ToastAttention),
    /// }
    ///
    /// let layout: AppLayout<(), Message> = AppLayout::new(());
    /// let layout = layout.on_toast_attention(Message::ToastAttention);
    /// assert!(layout.on_toast_attention.is_some());
    /// ```
    ///
    /// Unset, the toast layer carries no tracking at all.
    #[must_use]
    pub fn on_toast_attention(mut self, f: fn(u64, ToastAttention) -> Message) -> Self {
        self.on_toast_attention = Some(f);
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
    ///
    /// **Also published on Escape** (RFC-119): the keyboard cannot reach
    /// "outside", because an open menu's backdrop covers the page, so Escape
    /// closes the menu as a click outside does. The event is captured, so an
    /// application's own `iced::keyboard::listen()` does not also receive it:
    /// there is one message, not two. **Not published on a wheel** (RFC-125).
    #[must_use]
    pub fn on_close_menus(mut self, msg: Message) -> Self {
        self.on_close_menus = Some(msg);
        self
    }

    /// Wire the click-outside-to-close handler for dialog / bottom sheet.
    ///
    /// **Also published on Escape** (RFC-119), as for [`Self::on_close_menus`].
    /// With no handler here, neither a click outside nor Escape closes the
    /// modal: Escape is then the application's, through its own `keyboard::listen()`
    /// (the function that used to wrap that, `dismiss_on_escape`, was removed in 0.60.0).
    /// **Not published on a wheel** (RFC-125).
    #[must_use]
    pub fn on_close_modals(mut self, msg: Message) -> Self {
        self.on_close_modals = Some(msg);
        self
    }
}
