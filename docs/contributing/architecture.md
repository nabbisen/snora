# Internal architecture

This page is for people changing snora's source. For consumers, see
[reference/architecture.md](../reference/architecture.md), which is
shorter and stops at the public surface.

## Source layout

```text
crates/
├── snora-core/                  # vocabulary
│   src/
│     lib.rs                     # re-exports only
│     direction.rs               # LayoutDirection, Edge
│     icon.rs                    # Icon enum + From conversions
│     layout.rs                  # AppLayout struct + builder
│     menu.rs                    # Menu / MenuItem / MenuAction
│     overlay.rs                 # Dialog / BottomSheet / SheetHeight
│     sidebar.rs                 # SideBar / SideBarItem
│     toast.rs                   # Toast / ToastIntent / ToastLifetime
│                                # / ToastPosition
└── snora/                       # iced engine
    src/
      lib.rs                     # re-exports + module wiring
      direction.rs               # row_dir / row_dir_three
      render.rs                  # the only entry point: render(layout)
      style.rs                   # shared style functions
      toast.rs                   # toast layer + lifecycle helpers
      overlay.rs                 # module declaration
      overlay/
        bottom_sheet.rs
        dialog.rs
      widget.rs                  # module declaration
      widget/
        footer.rs
        header.rs
        sidebar.rs
        menu.rs
        icon.rs
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
