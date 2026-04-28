# Architecture overview

Snora is three crates with a strict dependency direction.

```text
your application
       │
       ▼
   snora                 (engine — depends on iced)
       │
       ├──► snora-widgets  (optional, prefab UI parts — depends on iced)
       │        │
       ▼        ▼
   snora-core            (vocabulary — no iced dependency)
```

Applications normally depend on a single crate, `snora`, which
re-exports the vocabulary from `snora-core` and (when its `widgets`
feature is enabled, the default) the prefab widgets from
`snora-widgets`.

## `snora-core` — vocabulary

This crate owns the **shape of the conversation** between an
application and a renderer. It contains:

- `AppLayout<Node, Message>` — the data structure describing what
  should be on screen.
- Vocabulary enums — `LayoutDirection`, `Edge`, `ToastIntent`,
  `ToastLifetime`, `ToastPosition`, `SheetEdge`, `SheetSize`, `Icon`.
- Plain-data overlay types — `Dialog<Node, Message>`,
  `Sheet<Node, Message>`, `Menu`, `MenuItem`, `MenuAction`,
  `SideBar`, `SideBarItem`, `Toast`.

`snora-core` has zero dependency on iced. It is, in principle, a
candidate for being driven by an alternative engine (a test double,
a WGPU frontend, an HTML renderer).

## `snora-widgets` — optional prefab widgets

This crate owns the **visuals of the prefab parts** — the bordered
header bar, the icon-rail sidebar, the chrome-styled footer, the
drop-down menu rendering, the icon resolver. Each is a function
returning an `iced::Element`, so they slot into any `AppLayout`
position by hand.

`snora-widgets` depends on `snora-core` (vocabulary) and `iced`. It
does **not** depend on `snora` — the widgets work against any engine
that consumes `snora-core`.

Applications normally do not depend on `snora-widgets` directly.
They are pulled in transparently by `snora`'s default `widgets`
feature, which re-exports them under `snora::widget`.

## `snora` — engine

This crate binds the vocabulary to iced 0.14:

- `render(layout)` — the single entry point. Consumes
  `AppLayout<iced::Element<'_, M>, M>` and returns
  `iced::Element<'_, M>`.
- Toast layer — builds the stacked toast column and resolves
  `ToastPosition` to a physical anchor.
- Overlay renderers — `dialog`, `sheet`. They paint the centered
  card / edge-anchored panel; the dim backdrop is owned by `render`
  itself.
- Lifecycle helpers — `snora::toast::subscription`,
  `snora::toast::sweep_expired`.
- Re-exports of `snora-widgets` (when the `widgets` feature is on)
  under the path `snora::widget`.

## Why this split

Three reasons matter in practice:

1. **One iced upgrade only touches the iced-dependent crates.** When
   iced 0.15 ships, `snora-core`'s vocabulary stays the same; only
   `snora` and `snora-widgets` need their dependency line bumped.
   Applications that depend only on the re-exported names see no
   churn.

2. **Engine and widgets evolve at different paces.** `snora` (engine)
   is conservative — z-stack rules and overlay machinery should
   change rarely. `snora-widgets` (visuals) is freer to add new
   prefab parts on a faster cadence. Splitting them lets each move
   without dragging the other.

3. **The vocabulary is the smallest reviewable surface.** Reading
   `snora-core`'s few hundred lines is a quick way to understand
   what *can* be on screen in a snora application. Implementation
   details (z-stacks, dim layers, padding constants, widget styles)
   stay out of the conceptual model.

The split is not for runtime modularity — it is a documentation and
upgrade-management tool. Applications that supply 100 % of their UI
parts can opt out of `snora-widgets` via `default-features = false`
on `snora` to avoid pulling its compilation in.

## Layer-by-layer rendering

The `render` function composes layers in this order, bottom to top:

```text
0. skeleton          header / body+sidebar / footer
1. menu backdrop     transparent click sink (if a menu is open)
2. header_menu
3. context_menu
4. modal dim         40 % black click sink (if a modal is present)
5. dialog
6. sheet
7. toasts            always on top, even over modals
```

Layers are conditional: each one materializes only when the
corresponding `AppLayout` field is populated. The dim layer's
click-to-close behavior is driven by `on_close_modals` /
`on_close_menus`; if those are `None`, the layers still render but
without click-outside dismissal.

## What is not in any of these crates

- Form widgets (validation, fields). Use iced's primitives.
- Data-table or chart components. Use iced's `canvas` or a
  data-visualization crate.
- Theming definitions. snora consumes the active iced `Theme` to
  resolve intent colors and chrome styling; the theme itself is
  iced's concern.
- Persistence, networking, business logic. snora is a presentation
  layer.
