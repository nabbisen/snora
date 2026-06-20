# Vocabulary cheatsheet

Every public enum in snora-core, with one-line semantics. Use this as
a quick scan when you forget a variant name.

## Direction and edges

```rust,ignore
pub enum LayoutDirection { Ltr, Rtl }
pub enum Edge            { Start, End }
```

`LayoutDirection` is the framework-wide reading direction. `Edge`
expresses logical position on the horizontal axis; resolve to physical
left/right via `Edge::is_left_under(direction)`.

## Toasts

```rust,ignore
pub enum ToastIntent {
    Debug,
    Info,
    Success,
    Warning,
    Error,
}

pub enum ToastLifetime {
    Transient(std::time::Duration),
    Persistent,
}

pub enum ToastPosition {
    TopEnd,        // default
    TopStart,
    TopCenter,
    BottomEnd,
    BottomStart,
    BottomCenter,
}
```

`ToastIntent` maps to theme colors (intent → palette pair). Helpers:

- `ToastLifetime::DEFAULT` — 4-second transient.
- `ToastLifetime::seconds(n)` / `ToastLifetime::millis(ms)`.
- `ToastPosition::is_top()` / `is_bottom()` — partition helpers.

`ToastPosition` invariant: the **newest toast is always closest to the
anchor edge**. Applications push new toasts to the back of their queue
in chronological order; the engine honors the invariant automatically.
Top anchors render newest at the top; bottom anchors render newest at
the bottom.

`ToastIntent::Warning` note: iced's extended palette has no warning
semantic pair. Snora uses a private fallback color (stable amber/orange)
for this intent. The fallback is an implementation detail and may
change when iced adds a warning semantic — it is not a public design
token and cannot be overridden through the theme API.

## Sheets

```rust,ignore
pub enum SheetEdge {
    Bottom,             // default
    Top,
    Start,              // logical (LTR=left, RTL=right)
    End,                // logical (LTR=right, RTL=left)
}

pub enum SheetSize {
    OneThird,           // default
    Half,
    TwoThirds,
    Ratio(f32),         // clamped 0.0..=1.0
    Pixels(f32),
}
```

`SheetSize` is interpreted along the axis perpendicular to the edge —
height for top/bottom, width for start/end.

Helpers:

- `SheetSize::DEFAULT`, `as_ratio()`, `as_pixels()`.
- `SheetEdge::is_vertical()` / `is_horizontal()` — partition helpers.

## Icons

```rust,ignore
pub enum Icon {
    Text(String),
    #[cfg(feature = "lucide-icons")] Lucide(lucide_icons::Icon),
    #[cfg(feature = "svg-icons")]    Svg(std::path::PathBuf),
}
```

## Menu actions

```rust,ignore
pub enum MenuAction<MenuId, MenuItemId> {
    MenuPressed(MenuId),
    MenuItemPressed { menu_id: MenuId, menu_item_id: MenuItemId },
}
```

`MenuId` and `MenuItemId` are application-defined. snora is generic
over both.

## Tabs

```rust,ignore
pub struct Tab<TabId: Clone + PartialEq> {
    pub id: TabId,
    pub label: String,
    pub icon: Option<Icon>,
}

pub struct TabBar<TabId: Clone + PartialEq> {
    pub tabs: Vec<Tab<TabId>>,
    pub active: TabId,
}

pub enum TabAction<TabId> {
    Pressed(TabId),
}
```

`TabId` is application-defined (typically a small enum). The
widget renders the entry whose `id == active` with an underline.

## Breadcrumbs

```rust,ignore
pub struct Crumb<CrumbId: Clone> {
    pub id: CrumbId,
    pub label: String,
    pub is_leaf: bool,
}

pub enum BreadcrumbAction<CrumbId> {
    Pressed(CrumbId),
}
```

Helpers:

- `Crumb::ancestor(id, label)` — clickable ancestor.
- `Crumb::leaf(id, label)` — current (last) entry, plain text.

## Defaults at a glance

```text
LayoutDirection::default()  → Ltr
ToastPosition::default()    → TopEnd
ToastLifetime::DEFAULT      → Transient(4 s)
SheetEdge::default()        → Bottom
SheetSize::DEFAULT          → OneThird
```
