# Direction and ABDD

snora's first principle is **ABDD** — Accessible By Default and by
Design. Layout is described in **logical edges**, not physical
directions, so an app written for English readers also works for
Arabic / Hebrew / Persian readers without a per-screen rewrite.

## Two switches

```rust,ignore
use snora::{LayoutDirection, ToastPosition};

let layout = AppLayout::new(body)
    .direction(LayoutDirection::Rtl)            // body row mirrors
    .toast_position(ToastPosition::TopEnd);      // toast anchor mirrors
```

`direction` controls how snora composes the **skeleton row** (where
the sidebar lands), and reaches the sheet and the toast layer. **It does
not reach the prefab widgets by itself:** each one (`app_header`,
`app_side_bar`, …) takes its own `direction` argument, and you pass it the
same value. *(Corrected 2026-11-15, from the 0.63 audit: this said `AppLayout::direction` was passed to them.)*

## Logical vs physical

`Edge` names the two logical sides. snora's own APIs take their own types
(`SheetEdge`, `ToastPosition`), so `Edge` is for your code. *(Corrected 2026-11-15, from the 0.63 audit: this
called it the central vocabulary type; nothing in snora takes one.)*

```rust,ignore
pub enum Edge { Start, End }
```

| Direction | `Edge::Start` | `Edge::End` |
|---|---|---|
| `Ltr` | left | right |
| `Rtl` | right | left |

Use `Edge::is_left_under(direction)` if you ever need to translate to
a physical side.

## Direction-aware rows

For your own widgets, `snora::direction::row_dir` (and `row_dir_three`)
build a **`LogicalRow`**: **the order is logical, and the positions mirror**
(0.63, RFC-132). The children stay in reading order in the widget tree under
both directions; under `Rtl` they are placed from the right edge.

```rust,ignore
use snora::direction::row_dir;

let bar = row_dir(
    state.direction,
    iced::widget::text("File: untitled"),       // start
    iced::widget::button("Save")                // end
        .on_press(Message::Save),
);
```

Under `Ltr`, "File:" is on the left and the Save button on the right;
under `Rtl`, they are mirrored. You write the row once. Because the tree order
does not change:
- **Tab visits them in reading order** under both directions: "File:" first, then
  Save. On screen that is right to left under `Rtl`.
- **A live flip keeps their state:** focus, a scroll offset, an open menu.

**Give a mirrored row a width, or align its container to the start edge.** A
`LogicalRow` that is only as wide as its content is mirrored about its own
width, so where it sits is decided by its container's alignment, as for any
widget. A row with `width(Length::Fill)` places its children from the start
edge of the space it fills.

**`LogicalRow` has the builders a mirrored row means:** `push`, `spacing`,
`align_y`, `width`, `height`, `padding`, and `Into<Element>`. Before 0.63 these
helpers returned iced's `Row`. Code that names `Row`, or calls another `Row`
method on the result, no longer compiles. See the 0.62 → 0.63 migration guide.

## Built-in widgets that take direction

| Widget | What flips |
|---|---|
| `app_header` | Title group (title, then menus) at the `Start` edge, end-controls at the `End` edge: a mirror image (0.63, RFC-132; before, under RTL the end-controls sat beside the title) |
| `app_tab_bar` | The tabs, from the start edge; a tab's icon and label |
| `app_breadcrumb` | The trail, from the start edge; the separator (`›` / `‹`) |
| Menus in the header | A trigger's and an item's icon and label |
| `app_side_bar` | Tooltip side; the sidebar position is determined by `AppLayout::direction` |
| Toast layer | Anchor side, when `ToastPosition` is `*Start` or `*End`; a toast's close mark (0.63) |
| `Sheet` | Anchor side, when `SheetEdge` is `Start` or `End`. Inside-facing rounded corner also flips. `Top` / `Bottom` edges are unaffected. |
| `Dialog` | Unaffected — centered, not edge-anchored |

**Five widgets take no direction yet, and draw left-to-right under RTL:** `design::notice::Notice`,
`design::chip::removable` (and `removable_with_tooltip`), `design::progress` (its fill also runs
from the left), `app_footer`, and `render_menu` used on its own (the header's menus do mirror).
RFC-133 gives each a `direction` parameter.

## Live flip

Live LTR ↔ RTL flipping is a one-line accessibility setting. Keep
`direction: LayoutDirection` on your state, mutate it on a user
action, and re-pass it on each `view`. Snora re-renders the whole
skeleton mirrored, **and keeps every widget's state**: focus, a scroll offset,
an open menu. Since 0.63 snora mirrors positions, not the tree (RFC-132).
Before 0.63 a flip lost focus and scroll offsets.

The [`examples/rtl`](https://github.com/nabbisen/snora/tree/main/examples/rtl)
demo flips on a button press.

## Intentionally non-mirroring elements

Some elements should *not* mirror:

- Numbers, currency, ISO dates — readers of any direction parse these
  left-to-right within their text.
- Code, logs, file paths — same.

Keep these in plain `text`; iced + snora will not mirror their
internal contents, only the surrounding layout.

## What snora does *not* do

- Bidirectional text shaping. iced 0.14 handles BiDi at the text-layout
  layer; snora does not augment that.
- Locale-driven number formatting. Use the `icu` or `num-format`
  crates.
- Mirrored icons. Lucide ships icons that already include
  direction-aware variants where appropriate; pick the right name.
  snora does not flip raster or SVG content.

ABDD is a layout discipline, not a complete i18n stack. snora gets the
skeleton right; you bring the rest of the i18n story.

## Contributing direction-sensitive changes

When adding or modifying any direction-sensitive surface, follow the
[ABDD compliance checklist](../contributing/abdd-checklist.md). The
checklist covers logical-edge API, RTL examples, tests, and wording
conventions.
