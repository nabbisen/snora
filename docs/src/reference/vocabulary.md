# Vocabulary cheatsheet

Every public enum in snora-core, with one-line semantics. Use this as
a quick scan when you forget a variant name.

## Direction and edges

```rust,ignore
{{#include ../../../examples/book_snippets/src/listings.rs:vocab_direction}}
```

`LayoutDirection` is the framework-wide reading direction. `Edge`
expresses logical position on the horizontal axis; resolve to physical
left/right via `Edge::is_left_under(direction)`.

## Toasts

```rust,ignore
{{#include ../../../examples/book_snippets/src/listings.rs:vocab_toast}}
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
{{#include ../../../examples/book_snippets/src/listings.rs:vocab_sheet}}
```

`SheetSize` is interpreted along the axis perpendicular to the edge —
height for top/bottom, width for start/end.

Helpers:

- `SheetSize::DEFAULT`, `as_ratio()`, `as_pixels()`.
- `SheetEdge::is_vertical()` / `is_horizontal()` — partition helpers.

## Icons

```rust,ignore
{{#include ../../../examples/book_snippets/src/listings.rs:vocab_icon}}
```

## Menu actions

```rust,ignore
{{#include ../../../examples/book_snippets/src/listings.rs:vocab_menu_action}}
```

`MenuId` and `MenuItemId` are application-defined. snora is generic
over both.

## Tabs

```rust,ignore
{{#include ../../../examples/book_snippets/src/listings.rs:vocab_tab}}
```

`TabId` is application-defined (typically a small enum). The
widget renders the entry whose `id == active` with an underline.

## Breadcrumbs

```rust,ignore
{{#include ../../../examples/book_snippets/src/listings.rs:vocab_crumb}}
```

Helpers:

- `Crumb::ancestor(id, label)` — clickable ancestor.
- `Crumb::leaf(id, label)` — current (last) entry, plain text.

## Zone navigation

```rust,ignore
{{#include ../../../examples/book_snippets/src/listings.rs:vocab_cycle}}
```

**`Cycle`** is the direction F6 moves in: `Forward` / `Backward`, in the logical order Header →
SideBar → Body → Footer, wrapping. It does not mirror under RTL: the sidebar is the start-edge
rail after the header in either direction. `snora::keyboard::cycle_zones(key, modifiers)` turns
F6 and Shift+F6 into one, and `snora::keyboard::focus_zone(cycle)` moves focus (RFC-126). See
[Accessibility](../guides/accessibility.md).

**Removed in 0.64.0** (deprecated in 0.61.0): `FocusZone`, `ZonePresence` and
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
