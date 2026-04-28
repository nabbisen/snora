# Built-in widgets

snora ships a small set of prefab `iced::Element` builders for the
common chrome — header, sidebar, footer, menu, icon. They are all
plain functions in `snora::widget` and entirely **optional**: any
`iced::Element` works in an `AppLayout` slot.

## When to use the prefabs

Use them to get a working app on screen quickly, or when your
chrome is indistinguishable from generic desktop UI. Skip them the
moment you want to customize beyond what the helper exposes — write
your own iced row and put it in the slot. Snora's value is the
skeleton + overlay machinery, not the styling of these specific
widgets.

## Inventory

### `app_header`

```rust
pub fn app_header<'a, Message, MenuId, MenuItemId, F>(
    title: &'a str,
    menus: Vec<Menu<MenuId, MenuItemId>>,
    on_menu_action: &'a F,
    active_menu_id: Option<&MenuId>,
    end_controls: Option<Element<'a, Message>>,
    direction: LayoutDirection,
) -> Element<'a, Message>
```

Bold title on the start edge, drop-down menus next to it, optional
controls anchored to the end edge. Direction-aware.

### `app_side_bar`

```rust
pub fn app_side_bar<'a, Message, ViewId>(
    side_bar: SideBar<Message, ViewId>,
    direction: LayoutDirection,
) -> Element<'a, Message>
```

Vertical icon rail with tooltips. The active item gets a subtle
background highlight. Tooltip side flips with direction.

### `app_footer`

```rust
pub fn app_footer<'a, Message>(
    content: Element<'a, Message>,
) -> Element<'a, Message>
```

Thin chrome-styled container. Direction is the caller's
responsibility — pass a `row_dir`-built row if you need start/end
layout inside.

### `render_menu`

```rust
pub fn render_menu<'a, Message, MenuId, MenuItemId, F>(
    menu: Menu<MenuId, MenuItemId>,
    on_action: &'a F,
    is_active: bool,
) -> Element<'a, Message>
```

Used internally by `app_header`. You normally do not call this
directly — `app_header` consumes a `Vec<Menu>` and renders all of
them. Direct calls are for non-header menus.

### `icon_element` / `icon_element_sized`

```rust
pub fn icon_element<'a, Message>(icon: &Icon) -> Element<'a, Message>;
pub fn icon_element_sized<'a, Message>(icon: &Icon, size: f32) -> Element<'a, Message>;
```

Resolve an `Icon` to an iced element at the default (14 px) or a
specified size. Honors all enabled icon backends.

## Direction helpers

In `snora::direction`:

```rust
pub fn row_dir<'a, M>(direction, start, end) -> iced::widget::Row<'a, M>;
pub fn row_dir_three<'a, M>(direction, start, center, end) -> iced::widget::Row<'a, M>;
```

The smallest tool for writing your own direction-aware widgets — see
[Direction and ABDD](../guides/direction.md).

## Style hooks

In `snora::style`:

- `chrome_container_style(theme)` — the bordered chrome look used by
  `app_header` and `app_footer`.
- `menu_button_style(theme, status)` — text-only button styling for
  menu entries.
- `sidebar_active_color(theme)` — the highlight color used for the
  active sidebar item.

These are exposed so that custom widgets can match the prefab look
when desired.
