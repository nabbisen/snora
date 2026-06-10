# RFC-013-A — Anchored Popover Design Study

> **v0.11 propagation note (2026-06-10).** This RFC's dependency on
> RFC-011-C is now satisfied: RFC-011-C is **decided and implemented in
> v0.11.0** (Option B — `AppLayout` is `#[non_exhaustive]`, builder path
> canonical). A future `pub popover: Option<Popover<…>>` field would
> therefore be an **additive, non-breaking** change, and must ship with a
> `#[must_use]` `.popover(...)` builder per RFC-011-C §7.2. Implementation
> still requires a concrete consuming application (unchanged).

Status: Proposed design study  
Target release: v0.13 discussion; implementation only after concrete demand  
Priority: Medium  
Type: Future feature design

## 1. Summary

Study whether Snora should add an anchored popover overlay: a transient surface positioned relative to a widget, point,
or rectangle. This would fill the gap between menus, centered dialogs, and edge-anchored sheets.

This RFC is a design study first. Implementation should not proceed until at least one real application need exists.

## 2. Motivation

Current Snora overlays cover:

- lightweight menus;
- centered modal dialog;
- edge-anchored modal sheet;
- global toasts.

They do not naturally cover:

- combobox-style dropdowns;
- hover cards;
- inspector bubbles;
- inline contextual details;
- anchored help popovers.

Applications can hand-roll these as positioned `Element`s today, but a general framework-level popover may be justified if
multiple apps need it and if Snora can model anchors without exposing unstable iced internals.

## 3. Goals

- Define whether anchored popover belongs in Snora.
- Compare anchor representation options.
- Preserve ABDD logical placement.
- Avoid broad widget-library drift.
- Avoid overpromising collision detection or focus management.

## 4. Non-Goals

- Do not implement in the design-study phase.
- Do not replace existing menu/dialog/sheet concepts.
- Do not build combobox, tooltip, or hover-card widgets.
- Do not add animation.
- Do not add focus trap.

## 5. External Design Candidate

Potential vocabulary:

```rust
pub struct Popover<Node, Message>
where
    Message: Clone,
{
    pub content: Node,
    pub anchor: PopoverAnchor,
    pub placement: PopoverPlacement,
    pub dismiss: PopoverDismissPolicy<Message>,
}

pub enum PopoverAnchor {
    Point { x: f32, y: f32 },
    Rect { x: f32, y: f32, width: f32, height: f32 },
}

pub enum PopoverPlacement {
    Above,
    Below,
    Start,
    End,
    Auto,
}

pub enum PopoverDismissPolicy<Message> {
    None,
    OutsideClick(Message),
}
```

Possible `AppLayout` addition:

```rust
pub popover: Option<Popover<Node, Message>>,
```

This addition depends on RFC-011-C because it may add a top-level field.

## 6. Placement Semantics

`Above` and `Below` are physical vertical placements.
`Start` and `End` are logical horizontal placements and must mirror under `LayoutDirection`.
`Auto` may initially be rejected because it implies collision detection and layout measurement.

Recommended initial support if implemented:

- `Above`, `Below`, `Start`, `End` only;
- no automatic viewport collision correction;
- application may choose placement based on its own knowledge.

## 7. Layering Semantics Options

### Option A — Popover as Menu-Layer Surface

Layer order:

```text
0. skeleton
1. menu/popover backdrop
2. header_menu
3. context_menu
4. popover
5. modal dim
6. dialog
7. sheet
8. toasts
```

Pros:

- popovers are lightweight;
- modal state dominates them;
- similar to menus.

Cons:

- may complicate menu close sink semantics.

### Option B — Popover Between Menus and Modals

Similar to Option A but with separate close sink:

```rust
on_close_popovers: Option<Message>
```

Pros:

- more precise lifecycle;
- avoids conflating menus and popovers.

Cons:

- adds another close sink;
- violates current simplicity of two close channels.

### Option C — No Framework Popover

Keep popovers as application-positioned context menu/content.

Pros:

- no new API;
- keeps Snora small.

Cons:

- repeated app code if many apps need it;
- no shared ABDD policy for anchored surfaces.

## 8. Internal Design Questions

Before implementation, answer:

1. Can iced reliably provide anchor geometry to the application?
2. Should Snora accept only application-provided geometry?
3. Does popover belong to menu close sink or a new close sink?
4. Should popovers be modal or non-modal?
5. Should multiple popovers be supported or only one?
6. Should collision detection be explicitly out of scope?
7. How does this interact with `AppLayout` construction stability?

## 9. Recommended Decision for Now

Do not implement in v0.13 unless a concrete consuming app exists.

Prepare the design, document the questions, and use application-specific positioned elements until repeated need appears.
If implemented later, start with a single non-modal popover using application-provided `Rect` or `Point` anchor.

## 10. Testing Requirements If Implemented

- placement resolves Start/End under LTR and RTL;
- outside-click dismissal works if configured;
- modal dim renders above popover;
- toast renders above popover;
- missing close sink still renders content;
- examples show LTR and RTL.

## 11. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Anchor API ties Snora to iced internals. | Prefer app-provided geometry. |
| Popover becomes a hidden widget library gateway. | Keep content generic `Node`; no combobox/tooltip implementation. |
| Collision detection scope explodes. | Explicitly defer or reject `Auto` placement initially. |
| Adds `AppLayout` field before construction policy. | Depend on RFC-011-C. |

## 12. Acceptance Criteria for the Design Study

- Anchor options are compared.
- Layering options are compared.
- Close-sink policy is discussed.
- ABDD implications are documented.
- Decision is recorded: implement, defer, or reject.
