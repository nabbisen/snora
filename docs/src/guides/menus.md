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

    // From 0.59 that is all: snora sees the open dropdown in the tree, covers the
    // page beneath it, and publishes `CloseMenus` on a click outside or Escape.
    let layout = AppLayout::new(state.body())
        .header(header)
        .on_close_menus(Message::CloseMenus);

    render(layout)
}
```

**How the open dropdown behaves (0.59, RFC-124):** it floats over the page, anchored
below its trigger, at the trigger's start edge (the end edge under RTL), and kept inside
the window.
- **It takes no room,** so opening it moves nothing.
- **A dropdown taller than the window scrolls,** and the item with keyboard focus is kept
  in view.
- **While it is open,** the page beneath it, the header included, is covered for the
  pointer and the keyboard: no click, no key, no hover, no wheel. From 0.60 the same holds
  for a context menu or a `header_menu` with its backdrop.
- **A click outside, on the trigger too, or Escape** publishes `CloseMenus`, or your
  `AppLayout::escape` message for Escape if you set one.

*(Fixed in 0.59.0: with the old wiring, a click on a dropdown item closed the menu instead
of pressing the item, and from 0.55 the keyboard could not reach the items. Opening a
menu also pushed the page down. An empty `Space` in `header_menu`, which that wiring
needed, is now harmless and no longer needed.)*

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

## What `header_menu` is for

`AppLayout::header_menu` holds **menu content of your own** in its own layer above the
page, as the `starter` example does: the starter passes `app_header` an empty `items` list,
and draws its menu itself. `app_header`'s own dropdown does not use the slot.

*(Corrected 2026-11-10: this section said `header_menu` took an empty `Space` because
"iced 0.14's element tree does not expose absolute positioning at the overlay layer". That
was false: `iced_core::overlay` is public, and from 0.59 the dropdown is an overlay
anchored to its trigger. The idiom it taught caused the defect fixed in 0.59 (RFC-124).)*

## The keyboard inside an open menu (0.60, RFC-123)

snora's own menus, `app_header`'s dropdown and a `render_menu` context menu, answer the keys
every desktop menu answers:

| Key | What it does |
|---|---|
| `↓` / `↑` | The next / previous item. **Wraps** from the last to the first, and back. Held, it keeps moving |
| `Home` / `End` | The first / last item |
| `↓` / `↑` with **nothing focused** | The first / last item |
| `Tab` / `Shift+Tab` | As everywhere: the next / previous control. With the menu's outside-click close, the page beneath is out of reach (RFC-119) |
| `Enter`, `Space` | Presses the focused item, publishing what a click publishes. **Moving never presses** |
| `Escape` | Closes the menu, and focus returns to the trigger |

**The rules a reader can rely on:**
- **Opening a menu never moves focus,** however it was opened: by the pointer, by Enter on the
  trigger, or by your application for a context menu. So from a focused trigger, **Enter then
  ↓** reaches the first item. A mouse user never sees a ring they did not ask for.
- **The arrows are the menu's only while focus is on its trigger or one of its items, or
  nowhere.** A focused text field keeps its arrows, as does a page control under a menu
  without a backdrop, or a toast's ×. Then the key goes on as if no menu were open.
- **Only the plain keys.** `Shift+↓` and `Ctrl+Home` are left alone, since a text field uses
  them to select.
- **An arrow the menu answers is captured,** so your `iced::keyboard::listen()` never receives
  it.
- **A long dropdown scrolls** to keep the focused item in view.
- **The arrows are not mirrored under RTL:** a menu is a vertical list. Home and End are first and
  last in reading order.
- **Under a dialog, an open menu answers nothing,** as it answers no click.

**What it needs:** nothing to wire. The arrows come with `snora::render` (or
`snora::design::render`), as Escape does. **A `render_menu` you draw outside `snora::render`
gets Tab, Enter and Space, but no arrows.**

**Not in 0.60:**
- **type-ahead** (a letter jumping to the item that starts with it);
- **arrow keys in the tab bar.**

Both are "future wanted" (RFC-116's long-term plan, L-10 and L-11).

**If you build the dropdown yourself,** passing your own element to
`AppLayout::header_menu` or `AppLayout::context_menu`, arrow-key traversal is yours to add:
track a highlighted index in your own state. `snora::control` (0.58) makes each of your
items a focus target, with the same rules as snora's.

*History.* RFC-082 recorded arrow-key traversal inside a menu as a concern separate from the
two keyboard axes snora owned: zone cycling between skeleton regions (F6,
RFC-060, which moves focus itself from 0.61, RFC-126) and Escape. Until 0.60, the arrows did nothing inside a snora menu.

*(Corrected 2026-11-09: this section said snora's prefab menus had no way to show a
highlighted item, and that keyboard traversal meant building your own dropdown. From 0.55,
the focus ring does that, and Tab reaches every item.)*
