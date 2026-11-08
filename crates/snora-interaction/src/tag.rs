//! What a control says about itself so that a shortcut can find it (RFC-116 R-2).
//!
//! A shortcut "does exactly what a click on that control does": it needs **that
//! control**, and a position or a count cannot name it (a control that is ever
//! disabled shifts the count, and iced gives a control a new `Id` on every
//! `view`). So each snora widget **tags its own controls as it builds them**:
//! the role it plays and its index within the widget, in the widget's own order
//! (a tab's index is its place in the `TabBar`, whatever the reading direction
//! does to the row).
//!
//! The tag is reported through `operation.custom`, as `start_here` is. A
//! disabled control reports nothing, so it cannot be found, and a shortcut to it
//! does nothing.

/// The part a tagged control plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Role {
    /// An item of the side bar.
    SideBarItem,
    /// A tab of a tab bar.
    Tab,
    /// An ancestor crumb of a breadcrumb trail.
    Crumb,
}

/// A role and an index within it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tag {
    /// The part the control plays.
    pub role: Role,
    /// Its index within that part, in the widget's own (logical) order.
    pub index: usize,
}

impl Tag {
    /// The control at `index` of `role`.
    #[must_use]
    pub const fn new(role: Role, index: usize) -> Self {
        Self { role, index }
    }
}

/// What a tagged control reports, and what a shortcut asks of it.
#[derive(Debug)]
pub(crate) struct TagReport {
    pub(crate) tag: Tag,
    /// Set by the router: make this control Tab's starting point (RFC-121),
    /// as a click on it would.
    pub(crate) start_here: bool,
}
