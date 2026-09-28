//! An open menu is distinguished by its dropdown (RFC-103 R-1).
//!
//! "Open" is one of the states `snora-widgets`' state register covers,
//! and its cue is a shape: the dropdown's items exist in the rendered
//! tree when the menu is active and not when it is not. Nothing asserted
//! that. The menu tests in `render_semantics.rs` look like they do, but
//! they cover the **engine's** menu layers — the `context_menu` slot, the
//! backdrop, the close sink — which is a different code path from
//! `snora_widgets::menu::render_menu` and its `if !is_active` early
//! return.
//!
//! Rendered rather than structural, because the cue is the item's
//! presence, and a user's evidence that the menu is open is that the
//! items are there and can be pressed.

#![cfg(feature = "widgets")]

use iced_test::simulator;

use snora::{Menu, MenuAction, MenuItem};

const TRIGGER: &str = "File";
const ITEM: &str = "Open";

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Menu(MenuAction<u8, u8>),
}

static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = Msg::Menu;

fn menu() -> Menu<u8, u8> {
    Menu {
        id: 0,
        label: TRIGGER.to_owned(),
        icon: None,
        items: vec![MenuItem {
            menu_id: 0,
            id: 7,
            label: ITEM.to_owned(),
            icon: None,
        }],
    }
}

/// Closed: the trigger is there, the dropdown is not.
#[test]
fn a_closed_menu_draws_no_dropdown() {
    let mut ui = simulator(snora::widget::render_menu(menu(), &ON_MENU, false));
    ui.find(TRIGGER)
        .expect("a closed menu still draws its trigger");
    assert!(
        ui.find(ITEM).is_err(),
        "a closed menu drew its dropdown item {ITEM:?} — the open state's only cue is that the \
         dropdown exists, so drawing it while closed leaves nothing to distinguish the two",
    );
}

/// Open: the dropdown is there, and its items are pressable.
#[test]
fn an_open_menu_draws_its_dropdown() {
    let mut ui = simulator(snora::widget::render_menu(menu(), &ON_MENU, true));
    ui.find(ITEM)
        .expect("an open menu draws its dropdown items — the cue for the open state");

    ui.click(ITEM).expect("a dropdown item is pressable");
    let messages: Vec<Msg> = ui.into_messages().collect();
    assert!(
        messages.contains(&Msg::Menu(MenuAction::MenuItemPressed {
            menu_id: 0,
            menu_item_id: 7,
        })),
        "pressing a dropdown item must emit its action; got {messages:?}",
    );
}
