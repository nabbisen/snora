# Overlay interaction semantics

This page is the normative reference for how Snora's overlay surfaces
coexist, how outside-click dismissal works, and what remains
application-owned. RFC-014-A (keyboard behavior) and RFC-014-B
(accessibility boundaries) — both landed at v0.14.0 — and RFC-060
(frame-level keyboard navigation, landed at v0.35.0) extend this page;
they do not replace it.

## Z-stack order (Law 1)

The engine renders layers from bottom to top. This order is part of the
framework contract; it must not change without an RFC.

```text
0. skeleton       — header + (side_bar | body) + footer
1. menu backdrop  — transparent click sink (if a menu is open)
2. header_menu    — dropdown under the header bar
3. context_menu   — floating menu at click point
4. modal dim      — dim click sink (if a modal is present): 40% via
                     `snora::render` (unstyled), 44% via
                     `snora::design::render` (`DIM_ALPHA`, RFC-065)
5. dialog         — centered; token-styled card via design::render
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

### Law 7 — Keyboard dismissal is application-owned

**Resolved (RFC-060): the "(for now)" this law previously carried is
retired, narrowly.** `Escape` dismissal itself is unchanged and stays
application-wired — `snora::keyboard::dismiss_on_escape` remains a pure
helper the application calls, not a subscription snora installs. What
changed is that keyboard navigation now has a **second** decision, and
it is snora's: frame-level zone cycling (`snora_core::focus::next_zone`,
recommended binding F6/Shift+F6 via `snora::keyboard::cycle_zones`, see
[Accessibility](../guides/accessibility.md)). Two keyboard concerns exist
now, with two different owners — this is not "snora owns the keyboard,"
it is one narrow addition alongside the one hedge this law used to carry.

**Snora does not own application shortcut routing.** `Escape` behavior is
not wired by the engine. Applications may map `Escape` to `CloseMenus` or
`CloseModals` using iced subscriptions or event handlers.

RFC-014-A added exactly this: a documented recipe and a small opt-in
helper, `snora::keyboard::dismiss_on_escape` (see [Keyboard
dismissal](#keyboard-dismissal) below). It remains opt-in and does not
change the existing two-sink model.

### Law 8 — Modal focus trapping is staged, not shipped

The modal dim/backdrop provides **visual modality** and **pointer blocking**.
It does not promise keyboard focus trapping or screen-reader modal semantics.
These are distinct concerns:

| Concern | Snora provides? |
|---|:--:|
| Visual modality (dim layer) | yes |
| Pointer blocking (backdrop capture) | yes |
| Keyboard dismissal (Escape) | no — application-owned (Law 7) |
| Frame-level zone navigation | **yes (RFC-060)** — `snora_core::focus::next_zone`; **suspended** while a modal is open (it reports this rather than guessing where focus inside the modal should go), unaffected while only a menu is open |
| Focus trapping *inside* an open modal | no — staged behind a separate decision (RFC-060 Q-1); see [design decisions](../contributing/design-decisions.md#why-focus-trapping-is-deferred-v014) for the constraint it inherits |

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

## Keyboard dismissal

Snora does not own application shortcut routing (Law 7). The recommended
pattern for `Escape` is to use `snora::keyboard::dismiss_on_escape`:

```rust,ignore
fn subscription(&self) -> iced::Subscription<Message> {
    let key_sub = iced::keyboard::listen().map(|event| {
        if let iced::keyboard::Event::KeyPressed { key, .. } = event {
            Message::KeyPressed(key)
        } else {
            Message::NoOp
        }
    });
    Subscription::batch([snora::toast::subscription(&self.toasts, || Message::ToastTick), key_sub])
}

fn update(&mut self, msg: Message) -> Task<Message> {
    match msg {
        Message::KeyPressed(key) => {
            if let Some(msg) = snora::keyboard::dismiss_on_escape(
                self.show_dialog || self.show_sheet,
                self.menu_open,
                Some(Message::CloseModals),
                Some(Message::CloseMenus),
                key,
            ) {
                return self.update(msg);
            }
        }
        // ...
    }
    Task::none()
}
```

| State | `Escape` behavior |
|---|---|
| Modal open | emit `on_close_modals` |
| Menu open, no modal | emit `on_close_menus` |
| Both open | modal takes priority |
| No overlay | no-op |
| Sink is `None` | no-op |

The workbench example demonstrates this pattern end-to-end.

## What Snora does not do

- **Escape handling** — Snora does not capture keyboard events. Wire
  `Escape` in your application's `subscription` or `update` using the
  recipe above — `snora::keyboard::dismiss_on_escape` (RFC-014-A,
  v0.14.0). The same non-capture policy applies to
  `snora::keyboard::cycle_zones` (RFC-060, below) —
  snora supplies pure decision functions, never a subscription.
- **Focus trapping *inside an open modal*** — The modal dim does not
  trap keyboard focus once it is inside the modal's own content. This is
  narrower than "focus management" generally: snora *does* now supply
  frame-level zone navigation between the skeleton's own slots
  (`snora_core::focus::next_zone`, RFC-060) — see [Law
  8](#law-8--modal-focus-trapping-is-staged-not-shipped) — and that
  navigation is correctly suspended while a modal is open. What remains
  unsupplied is bounding Tab *inside* the modal's own application-owned
  content, which needs iced's `advanced` feature and is staged behind a
  separate decision (RFC-060 Q-1), not implemented here.
- **Per-overlay close hooks** — There is no `on_close` on `Dialog` or
  `Sheet`. Use `AppLayout::on_close_modals` (Law 4).
- **Collision detection for popovers** — Not yet a Snora concept.
  (RFC-013-A is the design study.)
