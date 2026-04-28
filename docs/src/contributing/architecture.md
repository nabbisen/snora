# Internal architecture

This page is for people changing snora's source. For consumers, see
[reference/architecture.md](../reference/architecture.md), which is
shorter and stops at the public surface.

## Source layout

```text
crates/
├── snora-core/                  # vocabulary (no iced dep)
│   src/
│     lib.rs                     # re-exports only
│     direction.rs               # LayoutDirection, Edge
│     crumb.rs                   # Crumb / BreadcrumbAction
│     icon.rs                    # Icon enum + From conversions
│     layout.rs                  # AppLayout struct + builder
│     menu.rs                    # Menu / MenuItem / MenuAction
│     overlay.rs                 # Dialog / Sheet / SheetEdge / SheetSize
│     sidebar.rs                 # SideBar / SideBarItem
│     tab.rs                     # Tab / TabBar / TabAction
│     toast.rs                   # Toast / ToastIntent / ToastLifetime
│                                # / ToastPosition
├── snora-widgets/               # optional prefab widgets
│   src/
│     lib.rs                     # re-exports
│     direction.rs               # row_dir / row_dir_three
│     style.rs                   # shared style functions
│     crumb.rs                   # app_breadcrumb
│     footer.rs                  # app_footer
│     header.rs                  # app_header
│     icon.rs                    # icon_element / icon_element_sized
│     menu.rs                    # render_menu
│     sidebar.rs                 # app_side_bar
│     tab.rs                     # app_tab_bar
└── snora/                       # iced engine
    src/
      lib.rs                     # vocabulary re-exports + widget bridge
      render.rs                  # the only entry point: render(layout)
      toast.rs                   # toast layer + lifecycle helpers
      overlay.rs                 # module declaration
      overlay/
        sheet.rs                 # render_sheet (all 4 edges)
        dialog.rs                # render_dialog
```

No `mod.rs` files; we use the `my_module.rs + my_module/` layout
introduced in Rust 2018.

## Crate boundaries — what goes where

When in doubt, ask: *can this be done without iced?*

- If yes, it belongs in `snora-core`. Examples: enum definitions,
  struct field shapes, sweep logic on `Vec<Toast>`, the partition
  helpers `is_top()` / `is_left_under()`.
- If no, it belongs in `snora`. Examples: anything that returns or
  consumes `iced::Element`, anything that touches `iced::Theme`,
  any `Subscription`.

There are no exceptions. `snora-core`'s `Cargo.toml` does not list
iced as a dependency, and `cargo check -p snora-core` confirms this
before each merge.

## The render flow

`render` is the only place where layers get assembled into a stack.
Nothing else in either crate composes z-layers; downstream code
only ever produces individual elements. This keeps the layer order
in one place — see the comment block at the top of `render.rs`.

## Internal vs public visibility

- `pub` — appears in `snora-core` re-exports (and via `snora`'s
  re-exports). Consumer-visible.
- `pub(crate)` — engine internals. Used by `render` to call into the
  individual overlay renderers without exposing them.
- private — module-internal helpers.

When adding a new overlay or vocabulary item, default to private and
escalate when needed; do not start at `pub`.

## Adding a new vocabulary type

1. Add the enum / struct to `snora-core/src/<topic>.rs`.
2. Add unit tests next to it (variants partition cleanly, defaults are
   what they say, etc.).
3. Add it to `snora-core/src/lib.rs`'s top-level re-exports.
4. Add it to `snora/src/lib.rs`'s `pub use snora_core::{ ... }`.
5. Document it in `docs/reference/vocabulary.md`.

## Adding a new prefab widget

1. Add the function in `snora/src/widget/<name>.rs`.
2. Declare the module in `snora/src/widget.rs`.
3. Re-export from `snora/src/widget.rs` for ergonomic access.
4. Document it in `docs/reference/widgets.md`.

## Why no `Cargo.lock` in version control

Snora ships as libraries; Cargo's convention for libraries is to leave
`Cargo.lock` out of git so consumers' lockfiles win. Examples are
internal binaries but they share the workspace lockfile, which still
follows the library convention.
