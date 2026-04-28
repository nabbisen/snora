# Menus

snora has two menu shapes:

- **Header menu** — drop-down attached to the header bar (File / Edit /
  View). Triggered by clicking a labeled button. Item list renders
  inline below the button.
- **Context menu** — floating menu, typically right-click. Renders at a
  caller-chosen point. See [overlays](overlays.md#context-menu).

Both share the `Menu` / `MenuItem` / `MenuAction` vocabulary from
`snora-core`.

## Application-defined ids

You define two enums for menu identities:

```rust
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
```

snora is generic over both. It does not impose any string-based naming
or numeric tagging — your enum is the source of truth.

## Building menus

```rust
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
```

`MenuItem::menu_id` repeats the parent menu id so that the
[`MenuAction::MenuItemPressed`] event carries it without a second
lookup.

## Wiring into a header

```rust
use snora::{
    AppLayout, LayoutDirection, MenuAction,
    render, widget::app_header,
};

#[derive(Debug, Clone)]
enum Message {
    HeaderAction(MenuAction<MyMenuId, MyMenuItemId>),
    CloseMenus,
}

fn view(state: &State) -> iced::Element<'_, Message> {
    let header = app_header(
        "My App",
        menus,                              // built above
        &Message::HeaderAction,             // map MenuAction → Message
        state.active_menu.as_ref(),         // which menu is open
        None,                               // no end-controls
        LayoutDirection::Ltr,
    );

    let mut layout = AppLayout::new(state.body())
        .header(header)
        .on_close_menus(Message::CloseMenus);

    // Snora installs the click-outside backdrop only when
    // `header_menu` is `Some`. The actual dropdown is drawn inline by
    // `app_header`; this slot just opts the backdrop in.
    if state.active_menu.is_some() {
        layout = layout.header_menu(iced::widget::space().into());
    }

    render(layout)
}
```

## Handling the actions

`MenuAction` has two variants:

```rust
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
```

The "click the button to toggle, click an item to close" pattern is the
most common; you are free to use a different model (e.g. hover-to-open)
since snora only emits the events.

## Why `header_menu` takes an empty `Space`

iced 0.14's element tree does not expose absolute positioning at the
overlay layer; the dropdown is drawn *inside* `app_header` and what
populates `AppLayout::header_menu` is just the opt-in signal that "a
menu is open, please install the click-outside backdrop". Using
`Space` (zero-sized, transparent) is the canonical idiom.

This may become an actual element in a future version if iced exposes
absolute overlay positioning. The application-facing shape stays the
same.
