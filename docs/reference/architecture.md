# Architecture overview

Snora is two crates with a strict dependency direction.

```text
your application
       │
       ▼
   snora              (engine — depends on iced)
       │
       ▼
   snora-core         (vocabulary — no iced dependency)
```

## `snora-core` — vocabulary

This crate owns the **shape of the conversation** between an
application and a renderer. It contains:

- `AppLayout<Node, Message>` — the data structure describing what
  should be on screen.
- Vocabulary enums — `LayoutDirection`, `Edge`, `ToastIntent`,
  `ToastLifetime`, `ToastPosition`, `SheetHeight`, `Icon`.
- Plain-data overlay types — `Dialog<Node, Message>`,
  `BottomSheet<Node, Message>`, `Menu`, `MenuItem`, `MenuAction`,
  `SideBar`, `SideBarItem`, `Toast`.

`snora-core` has zero dependency on iced. It is, in principle, a
candidate for being driven by an alternative engine (a test double,
a WGPU frontend, an HTML renderer). In practice the only engine that
exists is the `snora` crate.

## `snora` — engine

This crate binds the vocabulary to iced 0.14:

- `render(layout)` — the single entry point. Consumes
  `AppLayout<iced::Element<'_, M>, M>` and returns
  `iced::Element<'_, M>`.
- Toast layer — builds the stacked toast column and resolves
  `ToastPosition` to a physical anchor.
- Overlay renderers — `dialog`, `bottom_sheet`. They paint the
  centered card / drawer surface; the dim backdrop is owned by
  `render` itself.
- Prefab widgets — `app_header`, `app_side_bar`, `app_footer`,
  `render_menu`, `icon_element`. These are *optional*: any
  `iced::Element` works in a layout slot.
- Lifecycle helpers — `snora::toast::subscription`,
  `snora::toast::sweep_expired`.

## Why this split

Two reasons matter in practice:

1. **One iced upgrade only touches one crate.** When iced 0.15 ships,
   only `snora` recompiles its dependency line. `snora-core`'s
   vocabulary stays the same — applications that depend only on the
   re-exported names see no churn.

2. **The vocabulary is the smallest reviewable surface.** Reading
   `snora-core`'s ~600 lines is a quick way to understand what
   *can* be on screen in a snora application. Engine implementation
   details (z-stacks, dim layers, padding constants) live in `snora`
   and stay out of the conceptual model.

The split is not for runtime modularity — it is a documentation and
upgrade-management tool.

## Layer-by-layer rendering

The `render` function composes layers in this order, bottom to top:

```text
0. skeleton          header / body+sidebar / footer
1. menu backdrop     transparent click sink (if a menu is open)
2. header_menu
3. context_menu
4. modal dim         40 % black click sink (if a modal is present)
5. dialog
6. bottom_sheet
7. toasts            always on top, even over modals
```

Layers are conditional: each one materializes only when the
corresponding `AppLayout` field is populated. The dim layer's
click-to-close behavior is driven by `on_close_modals` /
`on_close_menus`; if those are `None`, the layers still render but
without click-outside dismissal.

## What is not in either crate

- Form widgets (validation, fields). Use iced's primitives.
- Theming definitions. snora consumes the active iced `Theme` to
  resolve intent colors and chrome styling; the theme itself is
  iced's concern.
- Persistence, networking, business logic. snora is a presentation
  layer.
