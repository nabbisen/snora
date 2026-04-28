//! Vertical navigation rail (icon-only sidebar).
//!
//! A [`SideBar`] is a pure data contract describing a strip of
//! icon-and-tooltip buttons plus the currently active one. The engine
//! renders it, and pressing a button emits the item's `on_press` message.
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
    /// Tooltip shown on hover. Required for accessibility — keyboard and
    /// screen-reader users rely on it.
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
