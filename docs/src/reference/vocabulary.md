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

- `ToastLifetime::reading_time(title, message)` — what `Toast::new` gives every
  intent except `Error`: 2 s + 1 s per 6 characters, rounded up, at least 6 s
  (RFC-113). `Error` gets `Persistent`.
- `ToastLifetime::DEFAULT` — **removed in 0.57.0** (deprecated since 0.54.0); use
  `ToastLifetime::reading_time`, which `Toast::new` uses.
- `ToastAttention::{PointerEntered, PointerLeft, FocusGained, FocusLost}` —
  the reader's attention coming to or leaving a toast, by pointer or keyboard,
  as `AppLayout::on_toast_attention` reports it and
  `snora::toast::Holds::attend` takes it (0.56, RFC-120). `#[non_exhaustive]`.
- `ToastHover::{Entered, Left}` — **removed in 0.59.0** (deprecated since 0.56.0), with
  `AppLayout::on_toast_hover` and `Holds::hover`. Use `ToastAttention`, `on_toast_attention`
  and `Holds::attend`.
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

## Zone navigation

```rust,ignore
pub enum Cycle { Forward, Backward }
```

**`Cycle`** is the direction F6 moves in: `Forward` / `Backward`, in the logical order Header →
SideBar → Body → Footer, wrapping. It does not mirror under RTL: the sidebar is the start-edge
rail after the header in either direction. `snora::keyboard::cycle_zones(key, modifiers)` turns
F6 and Shift+F6 into one, and `snora::keyboard::focus_zone(cycle)` moves focus (RFC-126). See
[Accessibility](../guides/accessibility.md).

**Deprecated in 0.61.0, removed in 0.64.0:** `FocusZone`, `ZonePresence` and
`focus::next_zone`. They let an application hold the current zone and the slots it filled, so
that `next_zone` could decide where F6 goes. From 0.61, snora reads both from the tree when the
key is pressed, so there is nothing left for the application to hold.

`Tab` and `Crumb` are **not** zones: they are content an application places *inside* a zone.
The [0.38 → 0.39 migration guide](../guides/migration-0.38-to-0.39.md) says why the vocabulary
arrived (RFC-060).

## Defaults at a glance

```text
LayoutDirection::default()  → Ltr
ToastPosition::default()    → TopEnd
Toast::new(Error, …)        → Persistent
Toast::new(other, …)        → reading_time(title, message), ≥ 6 s
SheetEdge::default()        → Bottom
SheetSize::DEFAULT          → OneThird
```
