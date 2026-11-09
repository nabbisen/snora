# Built-in widgets

snora ships a small set of prefab `iced::Element` builders for the
common chrome — header, sidebar, footer, menu, icon. They are all
plain functions, available under `snora::widget` (re-exported from
the [`snora-widgets`](https://docs.rs/snora-widgets) crate when
the `widgets` feature is enabled, which is the default), and they
are entirely **optional**: any `iced::Element` works in an
`AppLayout` slot.

## When to use the prefabs

Use them to get a working app on screen quickly, or when your
chrome is indistinguishable from generic desktop UI. Skip them the
moment you want to customize beyond what the helper exposes — write
your own iced row and put it in the slot. Snora's value is the
skeleton + overlay machinery, not the styling of these specific
widgets.

If you want to skip the widget compilation entirely:

```toml
snora = { version = "0.58", default-features = false }
```

In that configuration `snora::widget`, `snora::direction`, and
`snora::style` do not exist; you supply your own elements for
every `AppLayout` slot.

## Inventory

### `app_header`

```rust,ignore
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

```rust,ignore
pub fn app_side_bar<'a, Message, ViewId>(
    side_bar: SideBar<Message, ViewId>,
    direction: LayoutDirection,
) -> Element<'a, Message>
```

The navigation rail. **Since 0.54 (RFC-114) each item shows its name,
beneath its icon by default.** The rail is as wide as its longest name,
capped at 240 px, and a name wraps rather than clipping. The active item
gets a highlight, and its name and icon always clear 4.5:1 against it: the
theme's own colour where that reaches AA, otherwise black or white,
whichever contrasts more.

The application chooses the form with `app_side_bar_with(side_bar,
direction, labels)`:

| `SideBarLabels::` | Item | Rail width |
|---|---|---|
| `Beneath` (default) | icon, with the name centred beneath it | 64 px, wider only for a name that needs it |
| `Beside` | icon, then the name, mirrored under RTL | icon + longest name |
| `Tooltip` | icon only; the name in a tooltip under the pointer (the pre-0.54 rail) | 64 px |

**`Tooltip` names items only under a resting pointer.** A touch user, or
anyone who cannot hold the pointer still, gets no names. **Every form is
reachable from the keyboard from 0.55** (RFC-116): each item is a focus
target once the application routes Tab with `snora::keyboard::traverse`, and
Enter and Space press it. A focused item in the `Tooltip` form still shows no
name, because iced opens tooltips only under the pointer.
`design::widget::app_side_bar_with` is the token-styled equivalent.

### `app_footer`

```rust,ignore
pub fn app_footer<'a, Message>(
    content: Element<'a, Message>,
) -> Element<'a, Message>
```

Thin chrome-styled container. Direction is the caller's
responsibility — pass a `row_dir`-built row if you need start/end
layout inside.

### `render_menu`

```rust,ignore
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

```rust,ignore
pub fn icon_element<'a, Message>(icon: &Icon) -> Element<'a, Message>;
pub fn icon_element_sized<'a, Message>(icon: &Icon, size: f32) -> Element<'a, Message>;
```

Resolve an `Icon` to an iced element at the default (14 px) or a
specified size. Honors all enabled icon backends.

### `app_tab_bar`

```rust,ignore
pub fn app_tab_bar<'a, Message, TabId, F>(
    bar: TabBar<TabId>,
    on_action: &'a F,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    F: Fn(TabAction<TabId>) -> Message + 'a;
```

Horizontal tab strip for peer-level navigation. The active tab gets
a colored underline; inactive tabs are flat text. Direction-aware:
the entire tab order mirrors under `Rtl`.

### `app_breadcrumb`

```rust,ignore
pub fn app_breadcrumb<'a, Message, CrumbId, F>(
    crumbs: Vec<Crumb<CrumbId>>,
    on_action: &'a F,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    F: Fn(BreadcrumbAction<CrumbId>) -> Message + 'a;
```

Hierarchical position indicator. Ancestors render as clickable
text; the leaf (current page, marked with `Crumb::leaf(...)`) is
plain text. The separator glyph flips with direction (`›` / `‹`).

## Direction helpers

In `snora::direction`:

```rust,ignore
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
