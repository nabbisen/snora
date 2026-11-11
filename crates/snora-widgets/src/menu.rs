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
    widget::{column, container, row, scrollable, text},
};

use snora_core::{Menu, MenuAction};
use snora_interaction::{MenuAnchor, MenuPart};

use crate::icon::icon_beside_label;
use crate::style::{
    BUTTON_DEFAULT_PADDING, ControlGeometry, chrome_container_style, control, menu_button_style,
    page_background,
};

/// Geometry parameter [`build_menu`] takes, letting [`render_menu`]
/// (unstyled) and the `design`-gated styled variant (RFC-040) share one
/// implementation.
pub(crate) struct MenuGeometry {
    /// Gap between a menu/item's icon and its label.
    pub(crate) gap: f32,
    /// Trigger and item label size, or `None` to inherit the host's
    /// `default_text_size` (RFC-104). The unstyled path inherits; the
    /// styled path maps it to `Typography::label`.
    pub(crate) label_size: Option<f32>,
    /// The minimum-target token and the focus ring (RFC-116).
    pub(crate) control: ControlGeometry,
    /// `Some(end_aligned)` floats the dropdown as an overlay anchored to the trigger
    /// (the header's menus, RFC-124); `None` draws it inline below the trigger (a menu
    /// in a layer of its own: the context menu, RFC-124 R-4).
    pub(crate) anchored: Option<bool>,
}

impl MenuGeometry {
    /// Today's literal, unmodified.
    pub(crate) const fn unstyled() -> Self {
        Self {
            gap: 6.0,
            label_size: None,
            control: ControlGeometry::unstyled(),
            anchored: None,
        }
    }
}

/// Render a single menu (header button + dropdown when active).
///
/// `on_action` is called for every interaction and must map the resulting
/// [`MenuAction`] into the application's message type. A plain function
/// item works well here because it has a zero-sized `'static` type and
/// can be passed by reference.
///
/// # Keyboard
///
/// Tab reaches the trigger and each item; Enter and Space press the focused one. **↓, ↑, Home
/// and End move between the items** (RFC-123), but only when the menu is rendered through
/// `snora::render` (as the header's menus and `AppLayout::context_menu` are): the engine's
/// layer stack answers them. A `render_menu` placed in an application's own widget tree gets
/// Tab, Enter and Space and **no arrows**.
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
    build_menu(menu, on_action, is_active, MenuGeometry::unstyled())
}

pub(crate) fn build_menu<'a, Message, MenuId, MenuItemId, F>(
    menu: Menu<MenuId, MenuItemId>,
    on_action: &'a F,
    is_active: bool,
    geometry: MenuGeometry,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    MenuId: Clone + Debug + PartialEq + 'a,
    MenuItemId: Clone + Debug + 'a,
    F: Fn(MenuAction<MenuId, MenuItemId>) -> Message + 'a,
{
    // Trigger button (always visible).
    let mut header_content = row![].spacing(geometry.gap).align_y(Center);
    if let Some(ref ic) = menu.icon {
        header_content = header_content.push(icon_beside_label(ic, geometry.label_size));
    }
    // No literal size (RFC-104): inherited unless the geometry maps it.
    let mut trigger_label = text(menu.label);
    if let Some(size) = geometry.label_size {
        trigger_label = trigger_label.size(size);
    }
    header_content = header_content.push(trigger_label);

    let trigger_msg = on_action(MenuAction::MenuPressed(menu.id.clone()));

    // RFC-116: `control` in place of `button`, so the trigger is a focus
    // target and Enter and Space open the menu. Padding is `button`'s own
    // default, set explicitly because `control` has none.
    let trigger = control(
        header_content,
        menu_button_style,
        geometry.control,
        geometry.label_size,
        page_background,
    )
    .padding(BUTTON_DEFAULT_PADDING)
    .on_press(trigger_msg);

    // Dropdown items — only built when this menu is active.
    let mut items: Vec<Element<'a, Message>> = Vec::new();
    if is_active {
        for item in menu.items {
            let mut btn_content = row![].spacing(geometry.gap).align_y(Center);
            if let Some(ref ic) = item.icon {
                btn_content = btn_content.push(icon_beside_label(ic, geometry.label_size));
            }
            let mut item_label = text(item.label);
            if let Some(size) = geometry.label_size {
                item_label = item_label.size(size);
            }
            btn_content = btn_content.push(item_label);

            let msg = on_action(MenuAction::MenuItemPressed {
                menu_id: item.menu_id.clone(),
                menu_item_id: item.id.clone(),
            });

            items.push(
                control(
                    btn_content,
                    menu_button_style,
                    geometry.control,
                    geometry.label_size,
                    page_background,
                )
                .padding(BUTTON_DEFAULT_PADDING)
                .on_press(msg)
                .into(),
            );
        }
    }

    match geometry.anchored {
        None => {
            let mut stack = column![MenuPart::trigger(trigger)];
            if !items.is_empty() {
                stack = stack.push(MenuPart::items(column(items)));
            }
            container(stack).into()
        }
        Some(end_aligned) => {
            // A menu with no items opens nothing: there is no empty card. The items are in
            // a `scrollable`, which is what lets a dropdown taller than the window
            // scroll (RFC-124); it is always there, so the tree does not change shape
            // with the item count.
            //
            // **The items are inset by the ring's reach inside the scrollable**, because a
            // scrollable clips what it holds: an item's focus ring is drawn outside it,
            // and without the inset the ring would lose its left and right sides, and its
            // top on the first item and bottom on the last.
            let reach = geometry.control.ring_offset;
            let dropdown = (!items.is_empty()).then(|| {
                container(scrollable(container(column(items)).padding(reach)))
                    .style(dropdown_card)
                    .into()
            });
            MenuAnchor::new(trigger, dropdown, end_aligned, geometry.control.ring_offset).into()
        }
    }
}

/// The dropdown's own surface: it floats over the page, so it has a fill of its own
/// (inline, it sat on the header's).
fn dropdown_card(theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(page_background(theme))),
        ..chrome_container_style(theme)
    }
}
