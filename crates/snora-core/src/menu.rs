//! Header menus (File / Edit / View / ... drop-downs).
//!
//! A menu is a **pure data contract**. The application supplies a list of
//! [`Menu`] values, the engine renders them into a header bar, and
//! interaction is reported back via [`MenuAction`] messages. snora-core
//! has no opinion on how the menu is rendered.
//!
//! `MenuId` and `MenuItemId` are application-defined types. They must
//! implement `Clone` for message dispatch and `PartialEq` for tracking
//! which menu is currently open.
//!
//! The recommended pattern is a pair of application-owned enums:
//!
//! ```rust
//! #[derive(Clone, Debug, PartialEq, Eq)]
//! enum MyMenuId { File, View, Help }
//!
//! #[derive(Clone, Debug, PartialEq, Eq)]
//! enum MyMenuItemId { New, Open, Quit, ToggleLogs, About }
//! ```

use std::fmt::Debug;

use crate::icon::Icon;

/// An event emitted by a header menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction<MenuId, MenuItemId> {
    /// The menu header (not an item) was pressed. Convention: toggle open /
    /// closed, or switch the currently-open menu.
    MenuPressed(MenuId),

    /// An item within a menu was chosen.
    MenuItemPressed {
        menu_id: MenuId,
        menu_item_id: MenuItemId,
    },
}

/// A top-level menu in the header (e.g. "File", "View").
#[derive(Debug, Clone)]
pub struct Menu<MenuId, MenuItemId>
where
    MenuId: Clone + Debug + PartialEq,
    MenuItemId: Clone + Debug,
{
    pub id: MenuId,
    pub label: String,
    pub icon: Option<Icon>,
    pub items: Vec<MenuItem<MenuId, MenuItemId>>,
}

/// A single entry in a menu's dropdown.
#[derive(Debug, Clone)]
pub struct MenuItem<MenuId, MenuItemId>
where
    MenuId: Clone + Debug + PartialEq,
    MenuItemId: Clone + Debug,
{
    /// The id of the parent menu. Stored on each item so that
    /// [`MenuAction::MenuItemPressed`] can carry it without a second lookup.
    pub menu_id: MenuId,
    pub id: MenuItemId,
    pub label: String,
    pub icon: Option<Icon>,
}
