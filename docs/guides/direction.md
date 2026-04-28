# Direction and ABDD

snora's first principle is **ABDD** — Accessible By Default and by
Design. Layout is described in **logical edges**, not physical
directions, so an app written for English readers also works for
Arabic / Hebrew / Persian readers without a per-screen rewrite.

## Two switches

```rust
use snora::{LayoutDirection, ToastPosition};

let layout = AppLayout::new(body)
    .direction(LayoutDirection::Rtl)            // body row mirrors
    .toast_position(ToastPosition::TopEnd);      // toast anchor mirrors
```

`direction` controls how snora composes the **skeleton row** (where
the sidebar lands), and is also passed to the prefab widgets so they
mirror their own internal `start` / `end` arrangements.

## Logical vs physical

`Edge` is the central vocabulary type:

```rust
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
build a `row!` whose order is decided by direction:

```rust
use snora::direction::row_dir;

let bar = row_dir(
    state.direction,
    iced::widget::text("File: untitled"),       // start
    iced::widget::button("Save")                // end
        .on_press(Message::Save),
);
```

Under `Ltr`, "File:" is on the left and the Save button on the right;
under `Rtl`, the order is mirrored. You write the row once.

## Built-in widgets that take direction

| Widget | What flips |
|---|---|
| `app_header` | Title group on `Start`, end-controls on `End` |
| `app_side_bar` | Tooltip side; the sidebar position is determined by `AppLayout::direction` |
| Toast layer | Anchor side, when `ToastPosition` is `*Start` or `*End` |
| `Sheet` | Anchor side, when `SheetEdge` is `Start` or `End`. Inside-facing rounded corner also flips. `Top` / `Bottom` edges are unaffected. |
| `Dialog` | Unaffected — centered, not edge-anchored |

## Live flip

Live LTR ↔ RTL flipping is a one-line accessibility setting. Keep
`direction: LayoutDirection` on your state, mutate it on a user
action, and re-pass it on each `view`. Snora re-renders the whole
skeleton mirrored — no per-widget reset needed.

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
