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

```rust,ignore
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

```rust,ignore
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

```rust,ignore
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

```rust,ignore
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

## Keyboard navigation within an open menu (RFC-082)

Arrow-key traversal **between items inside an open menu** — moving the
highlighted item down the list with `↓`, wrapping, picking with
`Enter` — is a different concern from the two keyboard axes snora
already owns: `snora::keyboard::cycle_zones` moves **between**
skeleton regions (RFC-060), and Escape **closes** an open menu or modal
that has an escape or close message (Law 7; `AppLayout::escape` gives one
an Escape of its own). Neither moves between items inside one.

**What snora's prefab menus do today (0.55 on):**
- **Every menu item is a focus target,** with a focus ring, and the header's menu trigger is
  one too. **Tab and Shift+Tab** move between the items of an open menu: with its
  outside-click backdrop, the page beneath is out of reach (RFC-119). **Enter or Space**
  presses the focused item. **Escape** closes the menu, and focus returns to the trigger.
- **Arrow keys do nothing inside a menu yet.** Native menus answer ↓ and ↑, and a keyboard
  user may reach for them first. RFC-116's long-term plan records it.
- **Opening a menu from the keyboard does not move focus into it.** The trigger's focus
  is set aside while the menu covers the page, and comes back when it closes. One Tab
  reaches the menu's first item.

**If you build the dropdown yourself,** passing your own element to
`AppLayout::header_menu` or `AppLayout::context_menu`, arrow-key traversal is yours to add:
track a highlighted index in your own state. `snora::control` (0.58) makes each of your
items a focus target, with the same rules as snora's.

*(Corrected 2026-11-09: this section said snora's prefab menus had no way to show a
highlighted item, and that keyboard traversal meant building your own dropdown. From 0.55,
the focus ring does that, and Tab reaches every item.)*
