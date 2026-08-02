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
├── snora-design/                # design tokens (no iced dep)
│   src/
│     lib.rs                     # re-exports
│     color.rs                   # Color + conversions
│     contrast.rs                # WCAG AA contrast utilities
│     focus.rs                   # focus ring tokens
│     palette.rs                 # Palette (18 semantic roles)
│     presets.rs                 # preset selection
│     presets/                   # light / dark / high-contrast presets
│     radius.rs                  # corner-radius tokens
│     spacing.rs                 # spacing scale
│     tokens.rs                  # Tokens bundle
│     typography.rs              # type scale
│     variants.rs                # status/intent variants
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
│     design.rs                  # module declaration (feature = "design")
│     design/                    # style bridge + shallow primitives
│       style.rs                 # style bridge module declaration
│       style/                   # per-widget style functions (&Tokens -> iced style)
│       button.rs                # design::button
│       card.rs                  # design::card
│       notice.rs                # design::notice
│       chip.rs                  # design::chip
│       progress.rs              # design::progress
└── snora/                       # iced engine
    src/
      lib.rs                     # vocabulary re-exports + widget bridge
      render.rs                  # the only entry point: render(layout)
      toast.rs                   # toast layer + lifecycle helpers
      overlay.rs                 # module declaration
      overlay/
        sheet.rs                 # render_sheet (all 4 edges)
        dialog.rs                # render_dialog
      design.rs                  # module declaration (feature = "design")
      design/                    # re-exports snora-design + snora-widgets::design
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
- If it is a **design value with no iced type in its signature** —
  a color, a spacing scale, a contrast calculation, a radius or
  typography token — it belongs in `snora-design`, not `snora-core`
  and not `snora-widgets`. The style *bridge* that turns a token into
  an `iced::*::Style` belongs in `snora-widgets/src/design/style/`.

There are no exceptions. `snora-core`'s and `snora-design`'s
`Cargo.toml` files do not list iced as a dependency, and
`cargo check -p snora-core` / `cargo check -p snora-design` confirm
this before each merge (the CI `design-isolation` job enforces it).

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
5. Document it in `docs/src/reference/vocabulary.md`.

## Adding a new prefab widget

1. Add the function in `crates/snora-widgets/src/<name>.rs`.
2. Declare the module and re-export it in
   `crates/snora-widgets/src/lib.rs`.
3. Re-export from `crates/snora/src/widget.rs` so it reaches
   `snora::widget::*`.
4. Document it in `docs/src/reference/widgets.md`.

## Adding a new design primitive

1. Add the module under `crates/snora-widgets/src/design/<name>.rs`,
   with tests in `crates/snora-widgets/src/design/<name>/tests.rs`.
2. Declare it in `crates/snora-widgets/src/design.rs`.
3. Re-export it from `crates/snora/src/design.rs` under a named
   submodule.
4. Document it under `docs/src/design/`.
5. Confirm it against `docs/src/contributing/api-governance.md` before
   promoting it out of experimental status.

## Why `Cargo.lock` **is** in version control

Snora ships as libraries, and Cargo's convention for libraries is to leave
`Cargo.lock` out of git so consumers' lockfiles win — this was snora's own
policy from `b7af344` ("remove Cargo.lock from vcs") until RFC-042. The
workspace is also a measurement harness, and that second role is what
changed the calculus:

- **Measurement attributability.** The workspace carries 17 example and
  size-probe binaries feeding the binary-size and build-cost budgets,
  which exist to track drift *between releases*. With no lockfile, CI
  resolves dependencies fresh on every run, so a measured delta mixes
  snora's own change with whatever upstream published in the interim — the
  budgets end up measuring partly noise. A committed lockfile fixes
  resolution so a delta is attributable to snora's own change.
- **MSRV floor visibility.** The workspace's declared `rust-version` is
  only meaningful relative to a specific dependency resolution: `iced`
  and `wgpu` set the actual floor (see `versioning-policy.md`), and
  because `iced = "0.14"` is a caret range, an unpinned resolution lets a
  future patch release of `iced` silently raise that floor with no change
  on snora's side. A lockfile turns that into a reviewable diff instead of
  a surprise — this is exactly how the floor moved from the documented
  1.85 to the actual 1.88 unnoticed (RFC-041).
- **Reproducible workspace builds.** `cargo run -p
  snora-example-workbench` producing different dependency versions on two
  machines is a debugging cost with no compensating benefit.

**The cost accepted.** Committing a lockfile means CI stops noticing, on
its own, that a fresh dependency resolution would have broken — the
continuous early warning an unpinned tree provides is lost. RFC-042
accepts that cost deliberately and replaces it with a scheduled
`unpinned-build` workflow (weekly, plus manual dispatch) that runs `cargo
update` in a throwaway checkout, checks the workspace still builds and the
declared MSRV still holds, and fails loudly without ever committing the
updated lockfile. A failure means "upstream moved — investigate," not
"auto-merge."

**No effect on downstream consumers.** A library's committed `Cargo.lock`
is ignored by Cargo when the library is used as a dependency — downstream
applications resolve against their own lockfile regardless of what snora
commits. This is purely about this repository's own builds and
measurements.

This decision is revisitable, same as its predecessor: if the
measurement-harness rationale stops applying, the lockfile can be
untracked again with good reason, recorded the same way this reversal was.
