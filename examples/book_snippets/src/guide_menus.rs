//! Anchored source for `docs/src/guides/menus.md` (RFC-136): the view and the update.
//!
//! The page's `State` and `menus` are the reader's own: `menus` is "built above" (the snippet before), so it is
//! a `const` here, outside the anchors. The ids are the page's own, repeated from `guide_menus_items`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MyMenuId {
    File,
    View,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MyMenuItemId {
    New,
    Open,
    Quit,
    ToggleStatus,
    About,
}

struct State {
    active_menu: Option<MyMenuId>,
}

impl State {
    fn body(&self) -> iced::Element<'_, Message> {
        iced::widget::space().into()
    }

    fn dispatch(&mut self, menu_id: MyMenuId, menu_item_id: MyMenuItemId) {}
}

#[allow(non_upper_case_globals)]
const menus: Vec<snora::Menu<MyMenuId, MyMenuItemId>> = Vec::new();

// ANCHOR: menus_view
use snora::{AppLayout, LayoutDirection, MenuAction, render, widget::app_header};

#[derive(Debug, Clone)]
enum Message {
    HeaderAction(MenuAction<MyMenuId, MyMenuItemId>),
    CloseMenus,
}

fn view(state: &State) -> iced::Element<'_, Message> {
    let header = app_header(
        "My App",
        menus,                      // built above
        &Message::HeaderAction,     // map MenuAction → Message
        state.active_menu.as_ref(), // which menu is open
        None,                       // no end-controls
        LayoutDirection::Ltr,
    );

    // From 0.59 that is all: snora sees the open dropdown in the tree, covers the
    // page beneath it, and publishes `CloseMenus` on a click outside or Escape.
    let layout = AppLayout::new(state.body())
        .header(header)
        .on_close_menus(Message::CloseMenus);

    render(layout)
}
// ANCHOR_END: menus_view

#[rustfmt::skip]
fn on_action(state: &mut State, action: snora::MenuAction<MyMenuId, MyMenuItemId>) {
use snora::MenuAction;
// ANCHOR: menus_update
match action {
    MenuAction::MenuPressed(id) => {
        // Toggle: same id closes; different id switches.
        state.active_menu = if state.active_menu == Some(id) { None } else { Some(id) };
    }
    MenuAction::MenuItemPressed { menu_id, menu_item_id } => {
        state.active_menu = None;            // close after pick
        state.dispatch(menu_id, menu_item_id);
    }
}
// ANCHOR_END: menus_update
}
