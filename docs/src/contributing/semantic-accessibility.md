# Semantic accessibility and construction policy

This document defines the construction rules that Snora Design primitives must
follow to keep their widgets keyboard-reachable and semantically meaningful,
and states the iced 0.14 focus-state limitation honestly.

See the [accessibility checklist](accessibility-checklist.md) for the full
review gate. This document is referenced from primitive RFCs (RFC-028 onward)
and from the CI acceptance criteria.

---

## Position on assistive technology (RFC-045)

> **snora will integrate an accessibility tree when iced exposes one.**
> Until then, ABDD means layout-direction correctness and visual
> accessibility — contrast, logical edges, and an asserted non-colour cue
> for every state snora draws itself — and snora states that boundary
> plainly rather than implying more.
>
> *(Corrected at 0.52.0, RFC-103. This line used to say "non-colour status
> encoding". That overstated it: for semantic **variants** (toast intent,
> notice and progress tone) snora contributes colour and the caller supplies
> the words, by ruling (RFC-093). What snora guarantees is a cue for the
> **states** it draws — see [the accessibility guide](../guides/accessibility.md#what-snora-does-not-provide).)*
>
> snora will **not** build a parallel accessibility abstraction of its own
> in the interim.

This is snora's stated position, citable directly rather than inferred from
silence. The gap is real: there is no accessibility tree, no AccessKit
integration, and no semantic identifiers anywhere in the crates, and a screen
reader sees nothing an application does not supply itself. That gap is not a
snora oversight — an accessibility tree requires the widget toolkit to expose
one, and iced 0.14 does not. It is recorded as a decision, with a
reconsideration trigger, in
[design decisions](design-decisions.md#why-snora-has-no-interim-accessibility-tree-v027).

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
| Button (primary / secondary / ghost / danger / link) | snora's `control` (`snora-interaction`, RFC-116) | **Yes, from 0.55:** a focus target; Enter and Space press it; a ring shows focus. Tab moves once the application routes it (`snora::keyboard::traverse`) |
| Interactive card | Semantic control where possible; or documented limitation | **None** if built on an iced `button`, which takes no keyboard input in iced 0.14; document either way |
| Notice dismiss / action | `control` | **Yes** (as for buttons) |
| Chip (interactive), and its remove button | `control` | **Yes** (as for buttons) |
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
   **A new snora control is built on `control`**, which answers yes: Tab
   reaches it (through `traverse`), and Enter and Space press it. Built on
   `iced::widget::button`, the answer is **no** in iced 0.14: it is not
   focusable and takes no keyboard input. *(Corrected 2026-11-07: in iced
   0.14 a `button` takes no keyboard input; see RFC-116.)*

3. **How is focus visible?**
   State whether a focus ring is rendered. `control` draws one, from
   `Tokens::focus` on the design path, at ≥ 3:1 against what it sits on.
   If the primitive has none, state that explicitly.

4. **What semantic limitation remains?**
   List any known gaps: missing focus ring, no ARIA role, missing screen-reader
   announcement, etc.

5. **What example demonstrates usage?**
   Name the example crate or workbench section where the primitive can be
   exercised and visually verified.

---

## iced 0.14 focus-state limitation

This is a hard constraint of the pinned iced version, not a design choice.

**From 0.55 it no longer limits snora's own controls**: they are `control`, which
keeps its own focus state and draws its own ring (RFC-116). It still describes
iced's `button` and `container`, which an application's own code uses, and the
style bridge, which maps their statuses.

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

The Snora Design style bridge (`snora_style::button` and
`snora_style::container`, RFC-055) maps the statuses iced does expose.
It **cannot** render a custom focus ring on a standard button or card surface
through `button::Style` / `container::Style` in iced 0.14 — because iced does
not tell the style closure that the widget is focused, not because a
`container` style closure is somehow incapable of drawing one. A `container`
style closure is an arbitrary `Fn(&Theme) -> Style`; anything the *caller*
already knows (including its own focus state) is available inside it.

`FocusTokens` (`tokens.focus.ring_color`, `.ring_width`, `.ring_offset`)
therefore has a **present-day audience**, not only a future one:

- any application that already owns focus as its own state — a
  focus-zone enum cycled by Tab, say — can read that boolean inside its own
  `container` style closure and set border colour *and* width from it. This
  is not a snora capability; it is ordinary use of an `Fn` closure, and
  `FocusTokens` is the vocabulary for what to set it to.
- future iced versions that expose focus state to `button`/`container`
  styling directly, so snora's own prefab widgets can wire it in without an
  application owning focus itself.
- custom widgets built outside the standard button/container path.

### What this means for QA

**A missing focus ring on a snora control is a bug** from 0.55: every control
snora draws is `control`, which draws its own ring.

A **missing focus ring on an iced `button` or card** — where the widget is
handed to iced and iced owns its focus state — is a **known, documented
limitation**, not a QA regression: record it in the
primitive's accessibility checklist under "Known limitations" with
severity `BLOCKED (iced 0.14 — no focus variant in button::Status)`.

**File it as a bug when it is one:** a missing ring on a widget the
*application* already owns focus for — where the application's own
`container` style closure could set it and does not — is not this
limitation, and treating every focus-ring gap as `BLOCKED` closes the
question before it is asked. This is the second review instruction in two
releases that did that; see RFC-057's line-height checklist item for the
first.

When iced exposes focus state in a future version, layer the focus ring
separately rather than reworking the status mapping.

### What iced does provide

**Pointer and touch only, for buttons.** iced 0.14's `button` handles mouse and touch events only, and registers no focus target: only `text_input` and `text_editor` implement `Focusable` (`iced_widget-0.14.2/src/button.rs`). A probe confirmed
it: a click presses a button and typing reaches a text input, but Tab, Enter and Space
never press a button, even after it has been clicked. So the gap is not only the
missing ring.

**snora's answer, from 0.55 (RFC-116): `control`.** It is snora's own widget, in
`snora-interaction` (named `snora-control` until 0.57), built on the public `iced_core::Widget` and `Focusable` (no
`advanced` feature). It registers a focus target when it has a handler, answers
Enter and Space with the message a click publishes, and draws its own ring in a
layer above its neighbours. Every control snora draws is one. **An application's
own content becomes one through `snora::control(content)` or
`snora::design::control(&tokens, content)`** (0.58, RFC-122). These take iced
`button`'s builder, so a `button` moves over by changing one name. An iced `button`
left as it is stays pointer and touch only.

*Corrected 2026-11-04:* this section said that iced's buttons respond to Enter and
Space when focused, and that the gap was "exclusively" visual. Both were wrong for
iced 0.14.

---

## Keyboard ownership table

**Rewritten for RFC-060, not amended** — the previous "Out of v0.20 scope
(deferred, RFC-014-B)" row for focus trapping is retired below, not left
beside a new one.

| Behavior | Owner |
|---|---|
| Moving focus between controls (Tab / Shift+Tab) | **Application, in one line**: iced binds no Tab key, so `update` calls `snora::keyboard::traverse(key, modifiers)`. Tab past the last control leaves the group; it does not wrap |
| Activation via Enter / Space | **Snora**, for every control it draws (`control`, from 0.55). **Nobody** for an application's own iced `button`, which takes no keyboard input in iced 0.14. *(Corrected 2026-11-04: this row said "iced (inherited)".)* |
| Frame-level zone navigation (Header/SideBar/Body/Footer) | **Snora** — `snora_core::focus::next_zone`, a pure decision function; the application wires the key (`snora::keyboard::cycle_zones`, F6/Shift+F6 recommended) and owns the current zone as its own state (RFC-060). Snora installs no subscription. |
| Snora visual focus ring | **Snora**: `control` draws it, from `Tokens::focus` on the design path, at ≥ 3:1 (0.55). An application that owns focus as its own state can still style its own `container` with `FocusTokens` (RFC-059). |
| Application keyboard shortcuts | Application |
| Escape-key overlay dismissal | **Snora**: the overlay's escape message (`AppLayout::escape`, 0.57), or else its click-outside close message (Law 7, RFC-119). `dismiss_on_escape` is deprecated since 0.57.0 |
| Keyboard shortcuts for snora's controls | **Snora, opt-in** (RFC-116 R-2): `Shortcuts`, through `AppLayout::shortcuts`. A shortcut does what a click does. See the [Keyboard map](../guides/keyboard.md) |
| Keyboard beneath an overlay; modal focus | **Snora**, from 0.55 (RFC-119, Law 8): the keyboard reaches only what the pointer can press. Tab inside a modal is bounded to the modal and the toasts (with iced's one empty press before it starts again), focus returns to the opener on closing, and the page's state survives. A strict trap with no gap is not built (RFC-060 Q-1, restated). Where focus starts in a modal is the application's |
| Screen-reader announcements | **None.** iced 0.14 has no accessibility tree and no AccessKit integration ([accessibility](../guides/accessibility.md)). *(Corrected 2026-11-04: this row named an "iced accessibility layer", which does not exist.)* |

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
