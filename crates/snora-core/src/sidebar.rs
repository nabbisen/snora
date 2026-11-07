//! Vertical navigation rail.
//!
//! A [`SideBar`] is a pure data contract describing a strip of named
//! icon buttons plus the currently active one. The engine renders it,
//! and pressing a button emits the item's `on_press` message. Where an
//! item's name is shown — beneath its icon, beside it, or only in a
//! tooltip — is the engine's business and the application's choice, not
//! part of this contract (RFC-114).
//!
//! # Keyboard
//!
//! The engine's rail is reachable by keyboard in every form it draws
//! (RFC-116): each item is a focus target, Enter and Space press the
//! focused one and publish the item's `on_press`, and a ring shows which
//! has focus. iced binds no Tab of its own, so the application routes the
//! key (`snora::keyboard::traverse`); Tab past the last item leaves the
//! rail rather than wrapping to the first.
//!
//! That is a property of the engine's rail. An application that supplies
//! its own `side_bar` element in the slot is responsible for its own
//! keyboard access.
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
    /// **It is not an accessibility route, in either form.** No screen
    /// reader reads this string: iced 0.14 exposes no accessibility tree.
    /// What it gives a reader is a name they can see (and a keyboard user
    /// reaches the item itself, by Tab, without needing the name to find
    /// it). Choose it accordingly — it is the only thing that says what
    /// the icon means (RFC-114).
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
