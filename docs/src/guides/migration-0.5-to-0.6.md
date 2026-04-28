# Migrating from 0.5 to 0.6

Snora 0.6 brings two changes that affect existing applications:

1. The `BottomSheet` overlay is now `Sheet` and supports four anchor
   edges (`Top`, `Bottom`, `Start`, `End`).
2. The prefab widgets moved out of the `snora` crate into a new
   `snora-widgets` crate, re-exported under `snora::widget` behind a
   feature gate.

Both changes ship with deprecated aliases so that 0.5.x code keeps
compiling. The aliases will be removed in 0.7.0.

## At a glance

| Change | Severity | Action |
|---|---|---|
| `BottomSheet` → `Sheet` | Source-compat alias kept | Optional rename in code; required before 0.7 |
| `SheetHeight` → `SheetSize` | Source-compat alias kept | Optional rename; required before 0.7 |
| `Sheet::with_height(...)` → `Sheet::with_size(...)` | No alias | Rename calls (one-line edit each) |
| `AppLayout::bottom_sheet(...)` → `AppLayout::sheet(...)` | Deprecated alias kept | Optional rename; required before 0.7 |
| New: `Sheet::at(SheetEdge::...)` | Additive | Use to anchor sheets at non-bottom edges |
| 3-crate split (snora-widgets carved out) | Internal — re-exports preserved | None for typical apps |
| Engine-only opt-out (`default-features = false`) | Additive | Use if you want to skip the widget set |

## `BottomSheet` → `Sheet`

`BottomSheet` was a fixed-bottom-anchor type. `Sheet` generalizes
that into a panel anchored to any of the four window edges, with
the size axis perpendicular to the chosen edge.

### Minimal rename

```rust
// 0.5
use snora::{BottomSheet, SheetHeight};
let sheet = BottomSheet::new(content).with_height(SheetHeight::Half);
let layout = AppLayout::new(body).bottom_sheet(sheet);

// 0.6 (recommended)
use snora::{Sheet, SheetSize};
let sheet = Sheet::new(content).with_size(SheetSize::Half);
let layout = AppLayout::new(body).sheet(sheet);
```

If you do nothing, the 0.5 code still compiles in 0.6 because
`BottomSheet`, `SheetHeight`, and `AppLayout::bottom_sheet` are
deprecated aliases of the new names. You will see `#[deprecated]`
warnings; suppress them with `#[allow(deprecated)]` on a function
or upgrade your code on whatever schedule fits.

`Sheet::with_height(...)` does **not** have an alias — it is
renamed to `with_size(...)` because the parameter type changed.
This is the only edit some applications will need to make to
silence warnings entirely.

### New: anchoring at other edges

```rust
use snora::{Sheet, SheetEdge, SheetSize};

// Sheet from the start edge (LTR=left, RTL=right)
let nav = Sheet::new(my_nav())
    .at(SheetEdge::Start)
    .with_size(SheetSize::Pixels(280.0));

let layout = AppLayout::new(body).sheet(nav);
```

`SheetEdge` is a logical edge type with `Start` / `End` variants
that mirror under [`LayoutDirection::Rtl`](direction.md), so an
app written for LTR readers automatically does the right thing
for RTL readers.

The default edge is `Bottom`, matching the 0.5 behavior — `Sheet::new(c)`
without `.at(...)` produces a bottom-anchored sheet.

### `SheetSize` semantics

`SheetSize` is interpreted along the axis perpendicular to the
edge. For `Top` / `Bottom` edges it is a height; for `Start` /
`End` edges it is a width. The variants are:

| Variant | Meaning |
|---|---|
| `OneThird` (default) | 33 % of the relevant axis |
| `Half` | 50 % |
| `TwoThirds` | 67 % |
| `Ratio(f32)` | clamped to `0.0..=1.0` |
| `Pixels(f32)` | fixed pixels |

## 3-crate split

In 0.6, the prefab widgets (`app_header`, `app_side_bar`,
`app_footer`, `render_menu`, `icon_element`) and their support
modules (`direction`, `style`) moved out of `snora` and into a new
`snora-widgets` crate. See
[reference/architecture.md](../reference/architecture.md) for the
full dependency picture.

### What changes for typical applications: nothing

`snora`'s `widgets` feature is **on by default**, and `snora`'s
lib re-exports `snora-widgets` under the same paths used in 0.5.
Your imports keep working:

```rust
// Both 0.5 and 0.6 — same import paths
use snora::widget::{app_header, app_side_bar, app_footer};
use snora::widget::icon::icon_element;
use snora::direction::row_dir;
use snora::style::chrome_container_style;
```

### What this enables: engine-only builds

Applications that supply 100 % of their UI parts and do not want
the widget set compiled in can opt out:

```toml
[dependencies]
snora = { version = "0.6", default-features = false }
```

This skips compilation of `snora-widgets` entirely. The
`snora::widget` module is not present, and neither is
`snora::direction` / `snora::style`. The engine surface
(`render`, `toast`, `AppLayout`, all vocabulary types) remains
unchanged.

If you mix-and-match — keeping snora-widgets but, say, swapping in
your own header — that still works without any opt-out, because
all `AppLayout` slots accept any `iced::Element`.

### Direct `snora-widgets` dependency (rare)

If you want to depend on `snora-widgets` directly (for example to
reuse widgets in a non-`snora` engine), it is published alongside:

```toml
[dependencies]
snora-widgets = "0.6"
```

This is unusual; the supported pattern is to depend on `snora`
and let it pull in `snora-widgets` transitively.

## Deprecation timeline

| API | 0.6 status | 0.7 plan |
|---|---|---|
| `BottomSheet` (type alias) | Available, deprecated | Removed |
| `SheetHeight` (type alias) | Available, deprecated | Removed |
| `AppLayout::bottom_sheet(...)` (method) | Available, deprecated | Removed |

To prepare for 0.7, do the renames now and run
`cargo clippy --workspace -- -D warnings -A deprecated` to
verify you have no remaining usages of the old names.

## Crate version pins

If your `Cargo.toml` had

```toml
snora = "0.5"
```

bump to

```toml
snora = "0.6"
```

If you have a direct dependency on `snora-core`, bump it too:

```toml
snora-core = "0.6"
```

Direct dependencies on `snora-widgets` are new in 0.6 and only
needed in the rare cases described above.
