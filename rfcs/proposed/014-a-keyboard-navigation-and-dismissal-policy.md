# RFC-014-A — Keyboard Navigation and Dismissal Policy

Status: Proposed  
Target release: v0.14 discussion; may be partly implemented earlier as recipes  
Priority: Medium-high  
Type: Interaction semantics / documentation / optional runtime helper

## 1. Summary

Define what Snora promises around keyboard dismissal and navigation, especially `Escape` behavior for menus and modal surfaces, without turning Snora into a global shortcut manager.

## 2. Motivation

Snora currently has clear mouse/backdrop dismissal channels through `on_close_menus` and `on_close_modals`. Keyboard behavior is less explicit. Desktop users expect `Escape` to close menus and modals, but applications also own their own key bindings. Without a policy, each app will invent behavior and Snora examples may become inconsistent. The goal is to give applications a safe, documented recipe and possibly a small helper while keeping shortcut routing app-owned.

## 3. Goals

- Define normative keyboard expectations for Snora surfaces.
- Keep global shortcut handling in the application.
- Provide a recipe that maps `Escape` to the same close messages used by outside-click.
- Avoid adding per-overlay close hooks or a general command system.
- Keep behavior compatible with the existing two close sinks: `on_close_menus` and `on_close_modals`.

## 4. Non-Goals

- Do not implement a full keyboard-navigation framework.
- Do not own application shortcuts.
- Do not add focus traversal management.
- Do not add command palette behavior.
- Do not add per-overlay close messages.

## 5. External Design

Recommended policy:

| Key | Surface state | Recommended behavior | Owner |
|---|---|---|---|
| `Escape` | menu open, no modal | emit `on_close_menus` | application, via recipe/helper |
| `Escape` | modal open | emit `on_close_modals` | application, via recipe/helper |
| `Escape` | menu and modal both present | emit `on_close_modals`; app should normally clear menus before opening modal | application |
| Arrow keys | menus/tabs/sidebar | app/widget-owned | application or prefab widget if implemented locally |
| Enter/Space | focused controls | iced/application-owned | application |

Possible helper, if repeated examples justify it:

```rust
pub fn dismiss_on_escape<Message: Clone>(
    has_menu: bool,
    has_modal: bool,
    on_close_menus: Option<Message>,
    on_close_modals: Option<Message>,
    key: iced::keyboard::Key,
) -> Option<Message>
```

However, the preferred first step is documentation and examples, not a public helper.

## 6. Internal Design

No required internal code change for the first phase.

If a helper is accepted later, place it in `crates/snora/src/keyboard.rs` and re-export as `snora::keyboard::dismiss_on_escape`. It must not depend on `snora-widgets`.

The helper should operate on plain booleans/messages rather than on `AppLayout` directly, so it can be called from an app's `update` or subscription/event handler without borrowing view-only data.

Pseudo-code:

```rust
if key != Escape {
    return None;
}
if has_modal {
    return on_close_modals;
}
if has_menu {
    return on_close_menus;
}
None
```

The policy must preserve the existing idea that close behavior has exactly two channels.

## 7. Testing and Acceptance

Acceptance criteria:

- The overlay interaction docs contain a keyboard section.
- The workbench example demonstrates `Escape` closing menus and modals.
- If a helper is added, unit tests cover: no surface, menu only, modal only, both menu and modal, and missing close messages.
- Render-semantics tests should continue to treat keyboard behavior as app-driven unless a helper becomes public API.

## 8. Documentation Updates

Update or add:

- `docs/src/reference/overlay-interaction-semantics.md`
- `docs/src/guides/overlays.md`
- `docs/src/guides/testing.md`
- Workbench example README or inline comments

The documentation must state that Snora does not own application shortcut routing.

## 9. Compatibility and Migration

Documentation-only phase is fully compatible.

A helper would be additive. It must not change `render`, `AppLayout`, or existing close semantics.

## 10. Open Questions

- Should Snora provide a public helper, or should examples be enough?
- Should `Escape` prioritize modals over menus when both are present, or should this state be documented as application-invalid?
- Should any prefab widgets eventually own arrow-key behavior, or should they remain pointer-first simple visuals?
