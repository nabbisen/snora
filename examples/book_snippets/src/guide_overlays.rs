//! Anchored source for `docs/src/guides/overlays.md` (RFC-136).

use iced::Element;
use snora::AppLayout;

#[derive(Clone)]
enum Message {
    CloseModals,
    CloseMenus,
}

fn my_card_element() -> Element<'static, Message> {
    iced::widget::space().into()
}

fn my_drawer_content() -> Element<'static, Message> {
    iced::widget::space().into()
}

fn my_floating_menu(point: iced::Point) -> Element<'static, Message> {
    iced::widget::space().into()
}

#[rustfmt::skip]
fn close(body: Element<'static, Message>) -> Element<'static, Message> {
// ANCHOR: overlays_close
let layout = AppLayout::new(body)
    .on_close_modals(Message::CloseModals)   // dialog / sheet
    .on_close_menus(Message::CloseMenus);    // context / header menus
// ANCHOR_END: overlays_close
snora::render(layout)
}

#[rustfmt::skip]
fn dialog(body: Element<'static, Message>) -> Element<'static, Message> {
// ANCHOR: overlays_dialog
use snora::{AppLayout, Dialog};

let layout = AppLayout::new(body)
    .dialog(Dialog::new(my_card_element()))
    .on_close_modals(Message::CloseModals);
// ANCHOR_END: overlays_dialog
snora::render(layout)
}

#[rustfmt::skip]
fn sheet(body: Element<'static, Message>) -> Element<'static, Message> {
// ANCHOR: overlays_sheet
use snora::{AppLayout, Sheet, SheetEdge, SheetSize};

let sheet = Sheet::new(my_drawer_content())
    .at(SheetEdge::Bottom)
    .with_size(SheetSize::Half);

let layout = AppLayout::new(body)
    .sheet(sheet)
    .on_close_modals(Message::CloseModals);
// ANCHOR_END: overlays_sheet
snora::render(layout)
}

#[rustfmt::skip]
fn context_menu(body: Element<'static, Message>, point: iced::Point) -> Element<'static, Message> {
// ANCHOR: overlays_context_menu
let layout = AppLayout::new(body)
    .context_menu(my_floating_menu(point))
    .on_close_menus(Message::CloseMenus);
// ANCHOR_END: overlays_context_menu
snora::render(layout)
}
