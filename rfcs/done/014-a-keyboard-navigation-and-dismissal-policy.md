# RFC-014-A — Keyboard Navigation and Dismissal Policy

**Status.** Implemented (v0.14.0)
**Tracks.** Interaction semantics / documentation / runtime helper.
**Touches.** `crates/snora/src/keyboard.rs` (new, public),
`crates/snora/src/lib.rs` (re-export),
`docs/src/reference/overlay-interaction-semantics.md` (keyboard section),
`examples/workbench/src/main.rs` (Escape wiring),
`docs/src/SUMMARY.md` (no change — keyboard section extends existing page).

> Builds directly on RFC-011-E Law 7. The overlay semantics page already
> states Escape is application-owned and includes the recipe. This RFC
> extends that with a formal helper and workbench demonstration.

## 1. [Decision] Provide the `dismiss_on_escape` helper

The planning draft said "preferred first step is documentation and
examples, not a public helper." The workbench now provides the first
example use site. The helper is three pure lines — it is a policy
expression, not a mechanism — and its value is identical to
`render_order_for` in RFC-011-B: documenting the decision as code.

Adopted: add `snora::keyboard::dismiss_on_escape` as a public helper.

## 2. External design

```rust,ignore
/// Returns the message to emit when `Escape` is pressed, following the
/// Snora overlay dismissal priority: modals before menus.
///
/// Pass `true` for `has_modal` when `AppLayout::dialog` or
/// `AppLayout::sheet` is populated. Pass `true` for `has_menu` when
/// `AppLayout::header_menu` or `AppLayout::context_menu` is populated.
///
/// Wire in your `update` or keyboard-event subscription:
///
/// ```rust,ignore
/// if let Some(msg) = snora::keyboard::dismiss_on_escape(
///     self.show_dialog || self.show_sheet,
///     self.open_menu.is_some(),
///     Some(Message::CloseModals),
///     Some(Message::CloseMenus),
///     key,
/// ) {
///     return self.update(msg);
/// }
/// ```
pub fn dismiss_on_escape<Message: Clone>(
    has_modal: bool,
    has_menu: bool,
    on_close_modals: Option<Message>,
    on_close_menus: Option<Message>,
    key: iced::keyboard::Key,
) -> Option<Message>
```

Priority: modal over menu when both are open (the state that the
overlay semantics document says apps should normally avoid, but which
the helper handles gracefully).

## 3. Internal design

New file `crates/snora/src/keyboard.rs`:

```rust,ignore
use iced::keyboard::Key;

pub fn dismiss_on_escape<Message: Clone>(
    has_modal: bool,
    has_menu: bool,
    on_close_modals: Option<Message>,
    on_close_menus: Option<Message>,
    key: Key,
) -> Option<Message> {
    if key != Key::Named(iced::keyboard::key::Named::Escape) {
        return None;
    }
    if has_modal {
        return on_close_modals;
    }
    if has_menu {
        return on_close_menus;
    }
    None
}
```

Re-exported as `snora::keyboard::dismiss_on_escape`. No `snora-widgets`
dependency. The module is always present (not feature-gated) because it
depends only on `iced::keyboard`, which is part of the engine crate's
existing dependency.

Unit tests in `keyboard.rs` cover:
- no surface open → `None`
- menu only → `on_close_menus`
- modal only → `on_close_modals`
- both → `on_close_modals` (modal priority)
- `on_close_modals = None` with modal → `None`
- non-Escape key → `None`

## 4. Workbench wiring

Add to `Workbench::update`:

```rust,ignore
Message::KeyPressed(key) => {
    if let Some(msg) = snora::keyboard::dismiss_on_escape(
        self.show_dialog || self.show_sheet,
        self.open_menu.is_some(),
        Some(Message::CloseModals),
        Some(Message::CloseMenus),
        key,
    ) {
        return self.update(msg);
    }
}
```

Add subscription in `Workbench::subscription`:

```rust,ignore
iced::keyboard::on_key_press(|key, _mods| {
    Some(Message::KeyPressed(key))
})
```

## 5. Overlay semantics doc extension

Add a "Keyboard" section to `overlay-interaction-semantics.md` after the
combination table, with the dismissal priority table and a link to the
helper.

## 6. ABDD check

`dismiss_on_escape` is not direction-sensitive. ABDD does not apply.

## 7. Acceptance criteria

- `snora::keyboard::dismiss_on_escape` exists and is documented.
- Unit tests cover all six cases.
- Workbench wires Escape via the helper.
- Overlay semantics doc has a keyboard section referencing the helper.
