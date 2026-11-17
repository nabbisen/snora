//! Anchored source for `docs/src/getting-started/03-add-a-header.md` (RFC-136).
//!
//! The page's `State`, `Message` and `ViewId` are the reader's own; they are defined here, outside the anchors.

use iced::Element;

#[derive(Clone, Copy, PartialEq)]
enum ViewId {
    Home,
}

#[derive(Clone)]
enum Message {
    HeaderAction(snora::MenuAction<(), ()>),
    Switch(ViewId),
}

struct State {
    active_view: ViewId,
}

impl State {
    fn body(&self) -> Element<'_, Message> {
        iced::widget::space().into()
    }
}

fn my_header_row() -> Element<'static, Message> {
    iced::widget::space().into()
}

// ANCHOR: header_view
use snora::{
    AppLayout, LayoutDirection, SideBar, SideBarItem, render,
    widget::{app_footer, app_header, app_side_bar},
};

fn view(state: &State) -> iced::Element<'_, Message> {
    let header = app_header(
        "My App",
        Vec::<snora::Menu<(), ()>>::new(), // no menus yet
        &Message::HeaderAction,
        None, // no menu open
        None, // no end-of-row controls
        LayoutDirection::Ltr,
    );

    let sidebar = app_side_bar(
        SideBar {
            items: vec![
                SideBarItem {
                    view_id: ViewId::Home,
                    icon: "🏠".into(),
                    tooltip: "Home".into(),
                    on_press: Message::Switch(ViewId::Home),
                },
                // …
            ],
            active: state.active_view,
        },
        LayoutDirection::Ltr,
    );

    let footer = app_footer(iced::widget::text("status").into(), LayoutDirection::Ltr);

    let body: iced::Element<'_, Message> = state.body();

    let layout = AppLayout::new(body)
        .header(header)
        .side_bar(sidebar)
        .footer(footer);

    render(layout)
}
// ANCHOR_END: header_view

#[rustfmt::skip]
fn custom(body: Element<'static, Message>) -> Element<'static, Message> {
use snora::AppLayout;
// ANCHOR: header_custom
let custom_header = iced::widget::container(my_header_row())
    .width(iced::Length::Fill)
    .padding(12)
    .into();

let layout = AppLayout::new(body).header(custom_header);
// ANCHOR_END: header_custom
snora::render(layout)
}
