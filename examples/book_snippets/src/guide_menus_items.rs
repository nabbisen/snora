//! Anchored source for `docs/src/guides/menus.md` (RFC-136): the ids and the menus built from them.

// ANCHOR: menus_ids
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
// ANCHOR_END: menus_ids

#[rustfmt::skip]
fn build_menus() {
// ANCHOR: menus_menus
use snora::{Menu, MenuItem};

let menus = vec![
    Menu {
        id: MyMenuId::File,
        label: "File".into(),
        icon: None,
        items: vec![
            MenuItem { menu_id: MyMenuId::File, id: MyMenuItemId::New,  label: "New".into(),  icon: None },
            MenuItem { menu_id: MyMenuId::File, id: MyMenuItemId::Open, label: "Open…".into(), icon: None },
            MenuItem { menu_id: MyMenuId::File, id: MyMenuItemId::Quit, label: "Quit".into(),  icon: None },
        ],
    },
    // …View, Help…
];
// ANCHOR_END: menus_menus
}
