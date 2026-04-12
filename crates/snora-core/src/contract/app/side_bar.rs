use std::fmt::Debug;

use crate::contract::ui::Icon;

#[derive(Debug, Clone)]
pub struct AppSideBarItem<Message, ViewId>
where
    ViewId: PartialEq,
{
    pub view_id: ViewId,
    pub icon: Icon,
    pub tooltip: String,
    pub action: Message,
}

#[derive(Debug, Clone)]
pub struct AppSideBar<Message, ViewId>
where
    ViewId: PartialEq,
{
    pub items: Vec<AppSideBarItem<Message, ViewId>>,
    pub active_view_id: ViewId,
}
