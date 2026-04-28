# Overlays

snora has three overlay surfaces. They differ in how modal they are
and how the user dismisses them.

| Surface | Modal? | Default dismiss | Layer |
|---|---|---|---|
| `Dialog` | yes | click backdrop or close button you provide | above modal dim |
| `Sheet` | yes | click backdrop or close button you provide | above modal dim |
| `context_menu` slot | no (light overlay) | click anywhere outside | below modal dim |
| `header_menu` slot | no (light overlay) | click anywhere outside | below modal dim |

## One close sink, two channels

```rust
let layout = AppLayout::new(body)
    .on_close_modals(Message::CloseModals)   // dialog / sheet
    .on_close_menus(Message::CloseMenus);    // context / header menus
```

These are the **only** two close sinks. Individual `Dialog` and `Sheet`
values do not carry their own close messages — there is exactly one
place per channel.

If you set an overlay but leave its sink `None`, the overlay still
renders. The framework simply omits the click-outside-to-close
backdrop, and you must provide an explicit close button inside the
overlay content. snora never silently drops a populated overlay.

## Dialog

A centered modal card. Snora paints the dim backdrop and centers your
content; everything else is your decision.

```rust
use snora::{AppLayout, Dialog};

let layout = AppLayout::new(body)
    .dialog(Dialog::new(my_card_element()))
    .on_close_modals(Message::CloseModals);
```

`Dialog` does not own the card chrome — you decide whether the dialog
content is a plain `container`, a styled card with a border, an entire
form. snora is a positioner, not a styler.

## Sheet

A modal panel anchored to one of the four window edges, occupying a
configurable size along the perpendicular axis.

```rust
use snora::{AppLayout, Sheet, SheetEdge, SheetSize};

let sheet = Sheet::new(my_drawer_content())
    .at(SheetEdge::Bottom)
    .with_size(SheetSize::Half);

let layout = AppLayout::new(body)
    .sheet(sheet)
    .on_close_modals(Message::CloseModals);
```

### Edges

| Variant | Where it slides from |
|---|---|
| `SheetEdge::Bottom` *(default)* | bottom of the window |
| `SheetEdge::Top` | top of the window |
| `SheetEdge::Start` | logical start (LTR=left, RTL=right) |
| `SheetEdge::End` | logical end (LTR=right, RTL=left) |

`Start` and `End` mirror automatically under
[`LayoutDirection::Rtl`](direction.md), like every other axis-aligned
piece of snora vocabulary.

The engine rounds only the *inside-facing* corners — the corners that
sit against the application content, not against the window edge. So
a bottom-anchored sheet rounds its top corners; a start-anchored sheet
under LTR rounds its right corners; etc.

### Size

The size is interpreted along the axis perpendicular to the edge: it
is a *height* for top/bottom edges and a *width* for start/end edges.

| Variant | Resolved size |
|---|---|
| `SheetSize::OneThird` *(default)* | 33 % of the relevant axis |
| `SheetSize::Half` | 50 % |
| `SheetSize::TwoThirds` | 67 % |
| `SheetSize::Ratio(f32)` | clamped to `0.0..=1.0` |
| `SheetSize::Pixels(f32)` | fixed pixels, ignores window size |

Pixel sizes ignore window resize and are usually wrong; prefer ratio
variants unless you have a hard pixel budget.

## Context menu

A floating menu (right-click style). It uses `on_close_menus`, not
`on_close_modals`, so it can coexist with an open dialog without one
dismissing the other.

```rust
let layout = AppLayout::new(body)
    .context_menu(my_floating_menu(point))
    .on_close_menus(Message::CloseMenus);
```

iced 0.14 does not surface the click coordinate alongside a button
press, so `Point`-based positioning of a context menu currently
requires either a `mouse_area` subscription or the iced advanced
widget API. The
[`examples/context_menu`](https://github.com/nabbisen/snora/tree/main/examples/context_menu)
demo uses fixed positions for clarity; treat it as a starting point
rather than a complete recipe.

## Header menu

Drop-down menus attached to a header bar (File / Edit / View …). See
the dedicated [Menus](menus.md) guide.

## Z-order recap

From bottom of the stack to top:

```text
0. skeleton           header / body / sidebar / footer
1. menu backdrop      transparent click sink for header/context menus
2. header_menu / context_menu
3. modal dim          40 % black click sink for dialog/sheet
4. dialog
5. sheet
6. toasts             always on top so they survive over modals
```

Toasts are deliberately on top of modals — a long-running export
finishing while a dialog is open should not be invisible.
