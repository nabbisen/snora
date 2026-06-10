# Overlay interaction semantics

This page is the normative reference for how Snora's overlay surfaces
coexist, how outside-click dismissal works, and what remains
application-owned. Future RFCs that touch keyboard behavior (RFC-014-A) or
accessibility boundaries (RFC-014-B) extend this page — they do not replace
it.

## Z-stack order (Law 1)

The engine renders layers from bottom to top. This order is part of the
framework contract; it must not change without an RFC.

```text
0. skeleton       — header + (side_bar | body) + footer
1. menu backdrop  — transparent click sink (if a menu is open)
2. header_menu    — dropdown under the header bar
3. context_menu   — floating menu at click point
4. modal dim      — 40%-dim click sink (if a modal is present)
5. dialog         — centered card
6. sheet          — edge-anchored panel
7. toasts         — always on top, RTL-aware anchor
```

Layers 1–6 are conditional on the corresponding `AppLayout` fields being
populated. Layer 7 is always evaluated but emits nothing when the toast
queue is empty.

## The laws

### Law 2 — Menus are lightweight and below modal state

Header and context menus are lightweight overlays. If a modal exists, modal
state dominates menus visually and interactively. Recommended app behavior:
close menus before opening a modal; do not intentionally keep menus open
under a modal.

### Law 3 — Dialog and sheet may coexist (advanced)

If both `dialog` and `sheet` are present, both render and the sheet is above
the dialog. This is supported (the z-stack guarantees it), but it is
documented as **advanced**. Prefer one modal surface at a time.

### Law 4 — Close sinks are global per overlay class

Snora exposes exactly two outside-click close sinks:

- `on_close_menus` — dispatched when the user clicks outside an open
  header or context menu.
- `on_close_modals` — dispatched when the user clicks the dim backdrop of
  a dialog or sheet.

Individual overlay values (`Dialog`, `Sheet`) do **not** carry their own
outside-click close messages. This is intentional: it makes wiring
impossible to get subtly wrong, and it means close behavior always has
exactly two channels.

### Law 5 — Missing close sink does not hide content

If an overlay is populated but its close sink is `None`, the engine still
renders the content:

- **Modal overlays** — the dim layer still paints (to signal "this is
  modal"), but outside clicks are not captured. The application must
  provide explicit close controls inside the overlay content.
- **Menu overlays** — the transparent outside-click backdrop is omitted;
  the menu still renders.

Content is never silently dropped.

### Law 6 — Toasts are above modal state

Toasts render above modals (layer 7). Operational feedback stays visible
even during a modal workflow. Use persistent error toasts sparingly — they
visually compete with a simultaneously open modal.

### Law 7 — Keyboard dismissal is application-owned (for now)

**Snora does not own application shortcut routing.** `Escape` behavior is
not wired by the engine. Applications may map `Escape` to `CloseMenus` or
`CloseModals` using iced subscriptions or event handlers.

A future RFC (RFC-014-A) may add a documented recipe or a small optional
helper. Any such addition will remain opt-in and will not change the
existing two-sink model.

### Law 8 — Focus management is out of scope until a concrete path exists

The modal dim/backdrop provides **visual modality** and **pointer blocking**.
It does not promise keyboard focus trapping or screen-reader modal semantics.
These are distinct concerns:

| Concern | Snora provides? |
|---|:--:|
| Visual modality (dim layer) | yes |
| Pointer blocking (backdrop capture) | yes |
| Keyboard dismissal (Escape) | no — application-owned (Law 7) |
| Focus trapping | no — deferred (RFC-014-B) |

ABDD is a **layout discipline**, not a complete accessibility or
localization stack.

## Combination table

| Combination | Supported? | Recommended? | Notes |
|---|:--:|:--:|---|
| header menu only | ✓ | ✓ | Normal menu use. |
| context menu only | ✓ | ✓ | Normal right-click menu use. |
| header + context menu | ✓ | rare | Usually only one active menu surface. |
| dialog only | ✓ | ✓ | Normal modal use. |
| sheet only | ✓ | ✓ | Normal workflow panel use. |
| dialog + sheet | ✓ | advanced | Sheet renders above dialog. |
| menu + dialog/sheet | ✓ | discouraged | Modal dim dominates menus; close menus first. |
| toast + anything | ✓ | ✓ | Toasts always on top; use persistent toasts sparingly. |

## Recommended state transitions

### Opening a modal (close menus first)

```rust,ignore
match msg {
    Message::OpenSettingsDialog => {
        // Law 2: clear menus before opening a modal.
        self.header_menu = None;
        self.context_menu = None;
        self.dialog = Some(DialogState::Settings);
    }
    Message::CloseModals => {
        self.dialog = None;
        self.sheet = None;
    }
    Message::CloseMenus => {
        self.header_menu = None;
        self.context_menu = None;
    }
    _ => {}
}
```

### Escape dismissal recipe (application-owned, Law 7)

```rust,ignore
match msg {
    // Prioritize modal over menu when both are present.
    Message::EscapePressed if self.dialog.is_some() || self.sheet.is_some() => {
        self.dialog = None;
        self.sheet = None;
    }
    Message::EscapePressed => {
        self.header_menu = None;
        self.context_menu = None;
    }
    _ => {}
}
```

## What Snora does not do

- **Escape handling** — Snora does not capture keyboard events. Wire
  `Escape` in your application's `subscription` or `update` using the
  recipe above. (RFC-014-A covers future helpers.)
- **Focus trapping** — The modal dim does not trap keyboard focus.
  Applications that need focus management must implement it using iced's
  focus primitives. (RFC-014-B covers the accessibility boundary
  definition.)
- **Per-overlay close hooks** — There is no `on_close` on `Dialog` or
  `Sheet`. Use `AppLayout::on_close_modals` (Law 4).
- **Collision detection for popovers** — Not yet a Snora concept.
  (RFC-013-A is the design study.)
