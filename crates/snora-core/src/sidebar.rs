//! Vertical navigation rail.
//!
//! A [`SideBar`] is a pure data contract describing a strip of named
//! icon buttons plus the currently active one. The engine renders it,
//! and pressing a button emits the item's `on_press` message. Where an
//! item's name is shown — beneath its icon, beside it, or only in a
//! tooltip — is the engine's business and the application's choice, not
//! part of this contract (RFC-114).
//!
//! # No form of this rail is reachable by keyboard
//!
//! iced 0.14's `button` takes no keyboard input: its `update` handles
//! mouse and touch only, and its `operate` registers no focus target.
//! Only `text_input` and `text_editor` implement `Focusable`. So a rail
//! item cannot be focused and no key presses it, whichever form the
//! engine draws. RFC-116 is the plan for that; until it lands, an
//! application that must be operable by keyboard needs a route to each
//! view that does not go through the rail.
//!
//! This is the minimum-viable navigation affordance. If your app needs
//! collapsible groups or nested navigation, compose your own element and
//! put it in the `side_bar` slot of [`crate::AppLayout`] directly —
//! snora-core does not force you through [`SideBar`].

use crate::icon::Icon;

/// One entry in a sidebar.
///
/// `ViewId` is the application's enum of addressable views. The sidebar
/// highlights the item whose `view_id` equals [`SideBar::active`].
#[derive(Debug, Clone)]
pub struct SideBarItem<Message, ViewId>
where
    Message: Clone,
    ViewId: Clone + PartialEq,
{
    /// The view this item points to. Compared against [`SideBar::active`]
    /// to decide whether to apply the highlight.
    pub view_id: ViewId,
    /// Visual icon for the rail button.
    pub icon: Icon,
    /// The item's name.
    ///
    /// A labelled rail — the engine's default — draws it on the rail
    /// itself, beneath the icon or beside it. An icon rail shows it only
    /// in a tooltip, which appears only under the pointer, so a touch
    /// user never sees it.
    ///
    /// **It is not an accessibility route, in either form.** Keyboard
    /// users cannot reach the rail at all, because iced 0.14's buttons
    /// take no keyboard input, and no screen reader reads this string:
    /// iced 0.14 exposes no accessibility tree. What it gives a reader is
    /// a name they can see. Choose it accordingly — it is the only thing
    /// that says what the icon means (RFC-114).
    pub tooltip: String,
    /// Message emitted when the user activates this item.
    pub on_press: Message,
}

/// The vertical navigation rail as a whole.
#[derive(Debug, Clone)]
pub struct SideBar<Message, ViewId>
where
    Message: Clone,
    ViewId: Clone + PartialEq,
{
    /// The list of rail entries, in display order.
    pub items: Vec<SideBarItem<Message, ViewId>>,
    /// The id of the view that is currently displayed in the body slot.
    /// The engine uses this to apply an "active" visual treatment.
    pub active: ViewId,
}
