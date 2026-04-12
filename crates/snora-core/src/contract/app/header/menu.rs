use std::fmt::Debug;

use crate::contract::ui::Icon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction<MenuId, MenuItemId> {
    MenuPressed(MenuId),
    MenuItemPressed {
        menu_id: MenuId,
        menu_item_id: MenuItemId,
    },
}

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

#[derive(Debug, Clone)]
pub struct MenuItem<MenuId, MenuItemId>
where
    MenuId: Clone + Debug + PartialEq,
    MenuItemId: Clone + Debug,
{
    pub menu_id: MenuId,
    pub id: MenuItemId,
    pub label: String,
    pub icon: Option<Icon>,
}
