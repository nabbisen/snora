# Migrating from 0.6 to 0.7

Snora 0.7 has two themes:

1. **Removal of the deprecation aliases** introduced in 0.6
   (`BottomSheet`, `SheetHeight`, `AppLayout::bottom_sheet()`).
2. **Two new navigation widgets**: `app_tab_bar` and
   `app_breadcrumb`, with their respective vocabulary types.

If you migrated all the way through the 0.6 deprecation hints,
your 0.6 code compiles unchanged on 0.7.

## At a glance

| Change | Severity | Action |
|---|---|---|
| `BottomSheet` removed | Breaking | Use `Sheet` (introduced in 0.6) |
| `SheetHeight` removed | Breaking | Use `SheetSize` (introduced in 0.6) |
| `AppLayout::bottom_sheet(...)` removed | Breaking | Use `AppLayout::sheet(...)` (introduced in 0.6) |
| `Tab`, `TabBar`, `TabAction`, `app_tab_bar` | Additive | New widget — use it if you want tabs |
| `Crumb`, `BreadcrumbAction`, `app_breadcrumb` | Additive | New widget — use it if you want breadcrumbs |

## Removing the 0.6 aliases

If your 0.6 build was clean of `#[deprecated]` warnings, no edit is
needed. If you suppressed the warnings or skipped 0.6, do the
renames now:

```rust
// Before (0.5 / 0.6 with deprecation warnings)
use snora::{BottomSheet, SheetHeight};
let sheet = BottomSheet::new(content).with_height(SheetHeight::Half);
let layout = AppLayout::new(body).bottom_sheet(sheet);

// After (0.6 / 0.7)
use snora::{Sheet, SheetSize};
let sheet = Sheet::new(content).with_size(SheetSize::Half);
let layout = AppLayout::new(body).sheet(sheet);
```

The full vocabulary of `Sheet` (including `SheetEdge` for
non-bottom anchors) is documented in
[guides/overlays.md](overlays.md).

## New: tab bar

For peer-level navigation — three to seven sibling views — use
[`TabBar`] and [`app_tab_bar`].

```rust
use snora::{
    AppLayout, Tab, TabAction, TabBar, render,
    widget::app_tab_bar,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceTab { Library, Editor, Settings }

#[derive(Debug, Clone)]
enum Message {
    TabAction(TabAction<WorkspaceTab>),
    /* ... */
}

fn view(state: &State) -> iced::Element<'_, Message> {
    let tabs = TabBar {
        tabs: vec![
            Tab { id: WorkspaceTab::Library,  label: "Library".into(),  icon: None },
            Tab { id: WorkspaceTab::Editor,   label: "Editor".into(),   icon: None },
            Tab { id: WorkspaceTab::Settings, label: "Settings".into(), icon: None },
        ],
        active: state.active_tab,
    };

    let body = iced::widget::column![
        app_tab_bar(tabs, &Message::TabAction, state.direction),
        state.body(),
    ];

    render(AppLayout::new(body.into()))
}
```

Key points:

- `TabBar` is generic over `TabId`. Use a small `Copy + PartialEq`
  enum.
- The active tab is highlighted with an underline drawn from the
  theme's primary color.
- Direction-aware: under `LayoutDirection::Rtl` the entire tab
  order mirrors.

The full guide on choosing between tabs and a sidebar is in
[guides/menus.md](menus.md) (note: tabs and menus are different
beasts; the guide spans both).

## New: breadcrumb

For ancestor-level navigation — "where am I in the hierarchy" —
use [`Crumb`] and [`app_breadcrumb`].

```rust
use snora::{
    BreadcrumbAction, Crumb, render,
    widget::app_breadcrumb,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrumbId { Home, Library, Books }

let crumbs = vec![
    Crumb::ancestor(CrumbId::Home,    "Home"),
    Crumb::ancestor(CrumbId::Library, "Library"),
    Crumb::leaf(CrumbId::Books,       "Books"),       // current page
];

let crumb_row = app_breadcrumb(crumbs, &Message::Breadcrumb, state.direction);
```

Key points:

- The application is responsible for marking exactly one entry as
  the leaf. Leaves render as plain text and do not emit events.
- Ancestors are clickable and emit
  `BreadcrumbAction::Pressed(CrumbId)`.
- The separator glyph flips with direction (`›` under LTR,
  `‹` under RTL).

## Examples

Two focused example crates ship with the repo:

```text
cargo run -p snora-example-tab          # peer-level navigation
cargo run -p snora-example-breadcrumb   # ancestor-level navigation
```

Each example is single-purpose so it stays under ~150 lines and
reads as documentation; combine them in your own application as
appropriate.

## A note on per-widget feature gates

Snora 0.7 still has only one `widgets` feature, not one feature
per widget. The criteria for revisiting that decision are
documented in
[contributing/feature-gating-criteria.md](../contributing/feature-gating-criteria.md).

If you have a use case that does not fit the current coarse gate,
that document explains what kind of evidence would justify a
finer split.

## Crate version pins

Bump:

```toml
snora = "0.7"
# (rare) snora-core, snora-widgets if depended on directly
```

There are no other breaking changes in 0.7.

[`TabBar`]: ../reference/vocabulary.md
[`app_tab_bar`]: ../reference/widgets.md
[`Crumb`]: ../reference/vocabulary.md
[`app_breadcrumb`]: ../reference/widgets.md
