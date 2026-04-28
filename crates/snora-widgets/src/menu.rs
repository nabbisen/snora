//! Render a [`snora_core::Menu`] into an `iced::Element`.
//!
//! A menu renders as a column: the trigger button on top, and — if the
//! menu is currently active — the item list below. The engine installs
//! the click-outside backdrop at the application level, so this widget
//! only paints the menu's own surface.

use std::fmt::Debug;

use iced::{
    Alignment::Center,
    Element,
    widget::{button, column, container, row, text},
};

use snora_core::{Menu, MenuAction};

use crate::style::menu_button_style;
use crate::icon::icon_element;

/// Render a single menu (header button + dropdown when active).
///
/// `on_action` is called for every interaction and must map the resulting
/// [`MenuAction`] into the application's message type. A plain function
/// item works well here because it has a zero-sized `'static` type and
/// can be passed by reference.
pub fn render_menu<'a, Message, MenuId, MenuItemId, F>(
    menu: Menu<MenuId, MenuItemId>,
    on_action: &'a F,
    is_active: bool,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    MenuId: Clone + Debug + PartialEq + 'a,
    MenuItemId: Clone + Debug + 'a,
    F: Fn(MenuAction<MenuId, MenuItemId>) -> Message + 'a,
{
    // Trigger button (always visible).
    let mut header_content = row![].spacing(6).align_y(Center);
    if let Some(ref ic) = menu.icon {
        header_content = header_content.push(icon_element(ic));
    }
    header_content = header_content.push(text(menu.label).size(14));

    let trigger_msg = on_action(MenuAction::MenuPressed(menu.id.clone()));

    let mut stack = column![];
    stack = stack.push(
        button(header_content)
            .style(menu_button_style)
            .on_press(trigger_msg),
    );

    if !is_active {
        return container(stack).into();
    }

    // Dropdown — only rendered when this menu is active.
    for item in menu.items {
        let mut btn_content = row![].spacing(6).align_y(Center);
        if let Some(ref ic) = item.icon {
            btn_content = btn_content.push(icon_element(ic));
        }
        btn_content = btn_content.push(text(item.label).size(14));

        let msg = on_action(MenuAction::MenuItemPressed {
            menu_id: item.menu_id.clone(),
            menu_item_id: item.id.clone(),
        });

        stack = stack.push(
            button(btn_content)
                .style(menu_button_style)
                .on_press(msg),
        );
    }

    container(stack).into()
}
