//! Anchored source for `docs/src/reference/widgets.md` (RFC-136, R-2): the signature listings, **as calls** with each
//! argument named. A listing that restated a signature could not be checked; a call that names every argument is a
//! compile error when the signature changes. The arguments are the parameters, with the types the page used to list
//! in the comments beside them.

use iced::Element;
use snora::design::Tokens;
use snora::widget::{
    app_breadcrumb, app_footer, app_header, app_side_bar, app_tab_bar, icon_element,
    icon_element_sized, render_menu,
};
use snora::{
    BreadcrumbAction, Crumb, Icon, LayoutDirection, Menu, MenuAction, SideBar, TabAction, TabBar,
};

#[derive(Clone)]
enum Message {
    Menu(MenuAction<u8, u8>),
    Tab(TabAction<u8>),
    Crumb(BreadcrumbAction<u8>),
}

#[rustfmt::skip]
fn header_call<'a>(
    title: &'a str,
    menus: Vec<Menu<u8, u8>>,
    on_menu_action: &'a impl Fn(MenuAction<u8, u8>) -> Message,
    active_menu_id: Option<&u8>,
    end_controls: Option<Element<'a, Message>>,
    direction: LayoutDirection,
) -> Element<'a, Message> {
// ANCHOR: widgets_app_header
app_header(
    title,            // &'a str
    menus,            // Vec<Menu<MenuId, MenuItemId>>
    on_menu_action,   // &'a F, where F: Fn(MenuAction<MenuId, MenuItemId>) -> Message
    active_menu_id,   // Option<&MenuId>
    end_controls,     // Option<Element<'a, Message>>
    direction,        // LayoutDirection
)
// ANCHOR_END: widgets_app_header
}

#[rustfmt::skip]
fn side_bar_call<'a>(side_bar: SideBar<Message, u8>, direction: LayoutDirection) -> Element<'a, Message> {
// ANCHOR: widgets_app_side_bar
app_side_bar(
    side_bar,   // SideBar<Message, ViewId>
    direction,  // LayoutDirection
)
// ANCHOR_END: widgets_app_side_bar
}

#[rustfmt::skip]
fn footer_call<'a>(content: Element<'a, Message>, direction: LayoutDirection) -> Element<'a, Message> {
// ANCHOR: widgets_app_footer
app_footer(
    content,    // Element<'a, Message>
    direction,  // LayoutDirection
)
// ANCHOR_END: widgets_app_footer
}

#[rustfmt::skip]
fn menu_call<'a>(
    menu: Menu<u8, u8>,
    on_action: &'a impl Fn(MenuAction<u8, u8>) -> Message,
    is_active: bool,
    direction: LayoutDirection,
) -> Element<'a, Message> {
// ANCHOR: widgets_render_menu
render_menu(
    menu,        // Menu<MenuId, MenuItemId>
    on_action,   // &'a F, where F: Fn(MenuAction<MenuId, MenuItemId>) -> Message
    is_active,   // bool
    direction,   // LayoutDirection
)
// ANCHOR_END: widgets_render_menu
}

#[rustfmt::skip]
fn icon_calls<'a>(icon: &Icon, size: f32) -> (Element<'a, Message>, Element<'a, Message>) {
let (a, b) = (
// ANCHOR: widgets_icons
icon_element(icon),               // &Icon
icon_element_sized(icon, size)    // &Icon, f32
// ANCHOR_END: widgets_icons
);
(a, b)
}

#[rustfmt::skip]
fn tab_bar_call<'a>(bar: TabBar<u8>, on_action: &'a impl Fn(TabAction<u8>) -> Message, direction: LayoutDirection) -> Element<'a, Message> {
// ANCHOR: widgets_app_tab_bar
app_tab_bar(
    bar,         // TabBar<TabId>
    on_action,   // &'a F, where F: Fn(TabAction<TabId>) -> Message
    direction,   // LayoutDirection
)
// ANCHOR_END: widgets_app_tab_bar
}

#[rustfmt::skip]
fn breadcrumb_call<'a>(crumbs: Vec<Crumb<u8>>, on_action: &'a impl Fn(BreadcrumbAction<u8>) -> Message, direction: LayoutDirection) -> Element<'a, Message> {
// ANCHOR: widgets_app_breadcrumb
app_breadcrumb(
    crumbs,      // Vec<Crumb<CrumbId>>
    on_action,   // &'a F, where F: Fn(BreadcrumbAction<CrumbId>) -> Message
    direction,   // LayoutDirection
)
// ANCHOR_END: widgets_app_breadcrumb
}

#[rustfmt::skip]
fn row_dir_calls<'a>(direction: LayoutDirection, start: &'static str, center: &'static str, end: &'static str) -> (snora::direction::LogicalRow<'a, Message>, snora::direction::LogicalRow<'a, Message>) {
use snora::direction::{row_dir, row_dir_three};
let (a, b) = (
// ANCHOR: widgets_row_dir
row_dir(direction, start, end),               // direction, start, end
row_dir_three(direction, start, center, end)    // direction, start, center, end
// ANCHOR_END: widgets_row_dir
);
(a, b)
}
