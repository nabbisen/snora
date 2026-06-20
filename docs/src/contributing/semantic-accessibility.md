# Semantic accessibility and construction policy

This document defines the construction rules that Snora Design primitives must
follow to keep their widgets keyboard-reachable and semantically meaningful,
and states the iced 0.14 focus-state limitation honestly.

See the [accessibility checklist](accessibility-checklist.md) for the full
review gate. This document is referenced from primitive RFCs (RFC-028 onward)
and from the CI acceptance criteria.

---

## Core rule

> **Prefer native iced interactive widgets** for interactive primitives.
> Avoid visual-only clickable containers — a `mouse_area` wrapping a
> `container` — when an iced-native control (button, checkbox, pick_list,
> slider, …) covers the use case.

Native iced controls inherit iced's event routing, keyboard handling, and any
accessibility-tree information iced exposes to the OS. A visual-only container
with a `mouse_area` loses all of that.

---

## Primitive construction table

| Primitive | Required construction | Keyboard reachability |
|---|---|---|
| Button (primary / secondary / ghost / danger) | `iced::widget::button` | Inherited from iced |
| Interactive card | Semantic control (iced button styled as a card) where possible; or documented limitation | Inherited if button; document if not |
| Notice dismiss / action | Real `iced::widget::button` controls | Inherited from iced |
| Chip (interactive) | `iced::widget::button` styled as chip where possible | Inherited from iced |
| Progress bar | `iced::widget::progress_bar` where possible | N/A (display only) |
| Non-interactive card | `iced::widget::container` | N/A (display only) |

When a primitive cannot use a native control for a justified reason, the
limitation must be stated in the primitive's RFC under a **"Semantic
limitation"** heading and echoed in the accessibility checklist for that
primitive.

---

## Five required questions (primitive RFC / PR)

Every RFC or PR that introduces a new interactive primitive must answer
these questions before merge:

1. **What native iced primitive is used?**
   Name the iced widget type. If none is used, justify why.

2. **Is it keyboard reachable?**
   State whether and how the primitive can be activated without a mouse.
   For `iced::widget::button`, the answer is "yes, inherited from iced."

3. **How is focus visible?**
   State whether a focus ring is rendered. If not (see the iced 0.14
   limitation below), state that explicitly.

4. **What semantic limitation remains?**
   List any known gaps: missing focus ring, no ARIA role, missing screen-reader
   announcement, etc.

5. **What example demonstrates usage?**
   Name the example crate or workbench section where the primitive can be
   exercised and visually verified.

---

## iced 0.14 focus-state limitation

This is a hard constraint of the pinned iced version, not a design choice.

### What iced 0.14 exposes

`iced::widget::button::Status` has exactly four variants:

```rust,ignore
pub enum Status {
    Active,
    Hovered,
    Pressed,
    Disabled,
    // No Focused variant.
}
```

`iced::widget::container` has **no interaction status at all** in iced 0.14;
the style closure receives only `&Theme`.

### Consequence for the style bridge

The Snora Design style bridge (`snora_widgets::design::style::button` and
`snora_widgets::design::style::container`) maps the statuses iced does expose.
It **cannot** render a custom focus ring on a standard button or card surface
through `button::Style` / `container::Style` in iced 0.14.

`FocusTokens` (`tokens.focus.ring_color`, `.ring_width`, `.ring_offset`)
remain valid vocabulary for:

- future iced versions that do expose focus state;
- custom widgets built outside the standard button/container path;
- any iced mechanism that separately surfaces focus (if one becomes available).

### What this means for QA

A **missing focus ring on a standard button or card** is a **known,
documented limitation**, not a QA regression. Do not file it as a bug. Do
record it in the primitive's accessibility checklist under "Known limitations"
with severity `BLOCKED (iced 0.14 — no focus variant in button::Status)`.

When iced exposes focus state in a future version, layer the focus ring
separately rather than reworking the status mapping.

### What iced does provide

iced's default button behavior does respond to keyboard input (Enter / Space
activates a focused button in iced's internal event handling). The gap is
exclusively in **visual focus styling**: iced does not tell the style closure
"this button is focused," so a custom ring cannot be drawn.

---

## Keyboard ownership table

| Behavior | Owner |
|---|---|
| Button activation via Enter / Space | iced (inherited) |
| Snora visual focus ring | Snora, where iced exposes focus state |
| Application keyboard shortcuts | Application |
| Escape-key overlay dismissal | Application (via `snora::keyboard::dismiss_on_escape`) |
| Focus trapping in modals | Out of v0.20 scope (deferred, RFC-014-B) |
| Screen-reader announcements | iced accessibility layer + OS; Snora does not override |

---

## Scope statement

Snora Design is an **opt-in layer** that provides contrast-tested tokens,
ABDD layout discipline, and shallow iced-native primitive helpers.

The allowed claim is:

> Snora Design provides accessibility-oriented defaults and ABDD layout
> discipline.

The disallowed claim is:

> Applications using Snora are automatically accessible.

Application developers remain responsible for: accessible content wording,
sufficient pointer target sizes in their layouts, correct landmark structure,
screen-reader-friendly labels, and any OS accessibility feature synchronization
not covered by snora.
