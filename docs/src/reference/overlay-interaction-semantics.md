# Overlay interaction semantics

This page is the normative reference for how Snora's overlay surfaces
coexist, how outside-click dismissal works, and what remains
application-owned. RFC-014-A (keyboard behavior) and RFC-014-B
(accessibility boundaries) — both landed at v0.14.0 — and RFC-060
(frame-level keyboard navigation, landed at v0.35.0) extend this page;
they do not replace it.

## The laws

### Law 1 — Z-stack order

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

**Checked (RFC-096, 2026-09-06): guarded for four specific boundaries, not
for the full sequence.** Each is asserted by a test that would fail if the
two layers it names were pushed in the other order — confirmed by actually
swapping them, not by reading the code and assuming:

- **dialog (5) vs sheet (6)** —
  `sheet_renders_above_dialog_when_both_overlap`. Swapping the push order
  makes `DialogOk` fire where the test asserts it must not.
- **sheet (6) vs modal dim (4)** — `sheet_content_button_reachable`.
  Swapping makes the click land on the dim (`CloseModals`) instead of the
  sheet's own button.
- **toast (7) vs modal state (4–6)** — `toast_dismiss_reachable_above_modal`.
  Swapping makes the dim capture the toast's dismiss click instead.
- **menu backdrop (1) vs modal dim (4)** —
  `menu_and_modal_together_menu_is_dominated_but_dialog_still_works`.
  Swapping lets the menu's own message through instead of the dim's.

**Not asserted: the full 0–7 sequence as one ordered whole**, nor the
relative order among layers 1–3 (menu backdrop, `header_menu`,
`context_menu` — no test constructs enough overlap between them to
distinguish push order). RFC-094 Unit 2 considered and rejected a
decision-shaped test for the complete sequence: every push in
`render_with_style` is guarded by a *presence* condition, never a
*reordering* one, so an extracted "order" value would be a constant and a
test comparing it to itself would assert nothing — see
`docs/src/contributing/api-freeze-review.md`'s semantic-contract row for
the full reasoning. The four boundaries above are the ones a real
application layout can make contest the same pointer event; the untested
ones either never spatially overlap in practice or have no test
constructing them together at all.

### Law 2 — Menus are lightweight and below modal state

Header and context menus are lightweight overlays. If a modal exists, modal
state dominates menus visually and interactively. Recommended app behavior:
close menus before opening a modal; do not intentionally keep menus open
under a modal.

**Checked (RFC-096, 2026-09-06): guarded.**
`menu_and_modal_together_menu_is_dominated_but_dialog_still_works`
(`crates/snora/tests/render_semantics.rs`) constructs a header menu and a
dialog together and asserts a click aimed at the menu item is captured by
the dim instead — the exact "dominates... interactively" claim, not a
consequence of it. Demonstrated failing twice: RFC-094 Unit 2 found the
combination had never been constructed at all before this test existed;
this RFC additionally swapped the menu/dim push order and confirmed the
menu's own message gets through when the dim no longer sits above it.

### Law 3 — Dialog and sheet may coexist (advanced)

If both `dialog` and `sheet` are present, both render and the sheet is above
the dialog. This is supported (the z-stack guarantees it), but it is
documented as **advanced**. Prefer one modal surface at a time.

**Checked (RFC-096, 2026-09-06): guarded, in two parts.** *"Both
render"* — `dialog_and_sheet_coexist_sheet_content_reachable`
(`crates/snora/tests/render_semantics.rs`). Demonstrated failing by
making the sheet push conditional on the dialog's *absence* (a
mutual-exclusivity bug): the sheet stopped rendering at all. *"The sheet
is above the dialog"* — `sheet_renders_above_dialog_when_both_overlap`,
the same test named for Law 1's dialog↔sheet boundary; it is one test
guarding both claims, since "above" here means exactly what Law 1's
z-stack claims for these two layers.

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

**Checked (RFC-096, 2026-09-06): guarded for the functional half, and
the structural half is enforced by the type system rather than a test.**
Each channel firing: `outside_click_on_menu_emits_close_menus` (menu
backdrop) and `outside_click_on_modal_emits_close_modals` (modal dim),
both in `crates/snora/tests/render_semantics.rs`. Demonstrated failing
by dropping each backdrop's `on_press` wiring in turn — each test alone
caught the break it names, with no cross-contamination. *"Individual
overlays do not carry their own close message"* is not a runtime
behavioral claim to assert: `Dialog` and `Sheet`
(`crates/snora-core/src/overlay.rs`) have no such field or method at
all, checked by reading their definitions — an absence the type
signature enforces at compile time, which a passing runtime test could
not add to.

### Law 5 — Missing close sink does not hide content

If an overlay is populated but its close sink is `None`, the engine still
renders the content:

- **Modal overlays** — the dim layer still paints (to signal "this is
  modal"), **and still blocks pointer input from reaching content
  beneath it** (RFC-084) — clicking or scrolling over the dim does
  nothing rather than reaching whatever is underneath. It just produces
  no dismiss message, since there is none to produce. The application
  must provide explicit close controls inside the overlay content.
  Before 0.41.0, a missing close sink meant the dim did not capture
  input at all — a click over it fell through to the layer beneath,
  which contradicted Law 8's own "Pointer blocking — yes" and is what
  RFC-084 corrected.
- **Menu overlays** — the transparent outside-click backdrop is omitted;
  the menu still renders.

Content is never silently dropped.

**Checked (RFC-096, 2026-09-06): guarded, both halves.** Modal:
`no_close_sink_means_no_dismiss_but_content_renders` (no dismiss
message, content still findable),
`modal_with_no_close_sink_still_blocks_pointer_at_dim` and
`modal_with_no_close_sink_also_blocks_wheel_scroll` (still blocks input,
independent of the sink) — all in `render_semantics.rs`. The two
blocking tests were demonstrated failing this session by removing the
`opaque()` wrapper from `dim_without_capture`, reproducing the
pre-RFC-084 F-02 bug exactly.

Menu: `menu_with_no_close_sink_still_renders` (RFC-096 Unit 2) —
constructs a header menu with `on_close_menus` absent and confirms the
content is still findable. Added rather than left as a gap because the
modal-side version of this exact claim was a shipped Critical
(RFC-084 F-02: a missing `on_close_modals` handler blocked nothing at
all, found by an external audit) — this is the untested half of a claim
whose other half was wrong in production once, not merely an
unexercised law. Demonstrated failing by gating the whole menu-content
push on the close sink's presence, the same shape F-02 had on the modal
side; restored once the test caught it.

### Law 6 — Toasts are above modal state

Toasts render above modals (layer 7). Operational feedback stays visible
even during a modal workflow. Use persistent error toasts sparingly — they
visually compete with a simultaneously open modal.

**Checked (RFC-096, 2026-09-06): guarded.** `toast_dismiss_reachable_above_modal`
(`crates/snora/tests/render_semantics.rs`) — the same test named for Law
1's toast↔modal boundary, since "toasts are above modal state" and "the
toast layer is pushed after the modal layers" are the same fact. Swapping
the push order (this RFC's evidence) makes the dim capture the toast's
dismiss click instead of the toast's own button receiving it.

<a id="law-7--an-overlay-that-closes-on-a-click-outside-also-closes-on-escape"></a>

### Law 7 — Escape closes an overlay that has an escape or a close message

**Changed in 0.55 (RFC-119).** When a menu's backdrop or a modal's dim has a close
message (`on_close_menus`, `on_close_modals`), **Escape publishes it**, exactly as a
click on the backdrop does. The engine captures that Escape, so the application's
`iced::keyboard::listen()` never receives it, and an application that also calls
`dismiss_on_escape` gets no second message. Order is Law 1's, from the top: an open
modal closes before a menu beneath it. A focused `text_input` inside a dialog takes
the first Escape itself, as iced does; the second closes the dialog.

**Why it changed.** From 0.55 the keyboard cannot reach beneath a backdrop (Law 8),
and the menu's own trigger is beneath it. Without this law, a keyboard user would be
trapped in any overlay whose application had not wired Escape (WCAG 2.1.2). The rule
is the pointer's: **the keyboard leaves the way the pointer can.**

**Changed in 0.57 (RFC-116 R-2): an escape message of its own.** `AppLayout::escape(Escape
{ modal, menu })` gives an overlay a message for Escape alone. That is right for an overlay
that should close on Escape but not on a stray click outside, such as a destructive
confirmation. The rule:

> **Escape publishes the overlay's escape message if one is set, and otherwise its
> click-outside close message. The click-outside message still answers clicks.**

It is answered on the overlay's own layer, after the overlay's content, so a focused field
still takes the first Escape. With neither message set, Escape does nothing, and the
overlay's own close button is the way out, for the pointer and the keyboard alike.
`snora::keyboard::dismiss_on_escape` is **deprecated since 0.57.0, and removed in 0.60.0**.
Snora installs no subscription. Its other keys are the shortcuts the application opts into
([Keyboard map](../guides/keyboard.md)).

*History.* Until 0.55 this law read "Keyboard dismissal is application-owned", and
the engine answered no key at all. RFC-060 had already added a second keyboard
decision of snora's own, frame-level zone cycling (`snora_core::focus::next_zone`,
F6/Shift+F6 via `snora::keyboard::cycle_zones`).

RFC-014-A added a documented recipe and a small opt-in helper,
`snora::keyboard::dismiss_on_escape` (see [Keyboard
dismissal](#keyboard-dismissal) below). It remains opt-in and does not
change the existing two-sink model.

**Checked (RFC-119, 2026-11-07): the engine's Escape is guarded** in
`crates/snora/tests/modal_keyboard.rs`: once, and `Captured`, for a menu
backdrop and for a modal dim; nothing, and `Ignored`, for a sinkless dim
over a menu; the `text_input` case in two presses at most. Each was shown
failing by its own break.

**Checked (RFC-096, 2026-09-06), and superseded in part by 0.55:** the
ownership claim of the time was not a testable behavior, and the recipe it
recommended is guarded. *"Snora does not capture keyboard events"* was an
absence — there is no installed
subscription to assert against, and `render()`'s output (an `Element`)
carries no subscription state at all in iced's model, so nothing in
this workspace's test harness could observe it either way. What *is*
testable and tested: `dismiss_on_escape` itself
(`crates/snora/src/keyboard.rs`), whose 7 unit tests match the state
table below exactly (modal-before-menu priority, no-op with no overlay,
no-op with no sink). Demonstrated failing this session by reversing the
modal-before-menu priority: `both_open_modal_takes_priority` caught it
immediately (`Some("menu")` where `Some("modal")` was expected).

<a id="law-8--modal-focus-trapping-is-staged-not-shipped"></a>

### Law 8 — The keyboard reaches only what the pointer can press

The modal dim and the menu backdrop provide **visual modality**, **pointer
blocking** and, from 0.55, **keyboard blocking** (RFC-119). Screen-reader modal
semantics are not provided: iced 0.14 has no accessibility tree.

| Concern | Snora provides? |
|---|:--:|
| Visual modality (dim layer) | yes |
| Pointer blocking (backdrop capture) | yes — the backdrop, the dialog, the sheet, and toasts each capture pointer input over their own bounds, so nothing beneath a modal or a toast is reachable through it (RFC-084; negative assertions in `crates/snora/tests/render_semantics.rs`). Covers both clicks and wheel-scroll. Independent of whether a close sink is provided (Law 5). |
| **Keyboard blocking** (what Tab and Enter reach) | **yes, from 0.55 (RFC-119).** Everything beneath the topmost backdrop is out of the keyboard's reach: no focus target, no key press, no text input, and no overlay of its own drawn above it. With a modal up, that is the skeleton and any menu. With a menu up **and** its backdrop (`on_close_menus`), it is the skeleton. A menu **without** a backdrop leaves the page reachable, by pointer and keyboard alike. An open **`app_header` dropdown** covers the page the same way, header included (0.59, RFC-124). The engine sees the open dropdown in the tree, so no empty `header_menu` is needed, and the dropdown floats above the page, anchored to its trigger. **A page covered by an open menu also gets no pointer:** no hover and no tooltip, as no click reaches it. *(A page under a modal's dim still shows hover in 0.59; making it match is scheduled for 0.60.)* Toasts stay reachable (Law 6). Independent of whether the modal has a close sink. Guarded in `crates/snora/tests/modal_keyboard.rs` |
| Keyboard dismissal (Escape) | yes: the overlay's escape message, or else its click-outside close message (Law 7; the escape field from 0.57) |
| **Where focus goes** | **When a modal closes, focus returns to the control that had it**, unless the page now has a different number of controls, or the application has focused something itself. Then focus is wherever the application put it, or **nowhere** (not the first control): the next Tab starts from the top. While the dim is up, a covered control shows no focus ring, and a covered `text_input` has no cursor; both come back on closing |
| **The page's state** | **Kept.** Opening and closing a modal or a menu does not rebuild the page: scroll offsets, a text input's cursor and every widget's state survive |
| Frame-level zone navigation | **yes (RFC-060)** — `snora_core::focus::next_zone`; **suspended** while a modal is open (it reports this rather than guessing where focus inside the modal should go), unaffected while only a menu is open |
| Tab inside an open modal | **bounded (RFC-119):** Tab moves through the dialog, the sheet and the toasts, in that order. Past the last, nothing is focused for one press, then Tab starts again from the modal's first control. This meets the need focus trapping was for; RFC-060 Q-1's strict trap (a wrap with no gap) is not built |

ABDD is a **layout discipline**, not a complete accessibility or
localization stack.

**Checked row by row (RFC-096, 2026-09-06):**

- **Visual modality (dim layer)** — not covered. Nothing in this
  workspace's test suite compares pixels; `render_semantics.rs`
  simulates pointer input, not paint. (One downstream, one-application
  pixel-hash confirmation exists — see
  `docs/src/contributing/api-freeze-review.md`'s gate-5 discussion — but
  that is not a test in this repository.)
- **Pointer blocking (backdrop capture)** — guarded, extensively: the
  entire RFC-084 containment family (`dialog_click_does_not_dismiss_modal`,
  `modal_with_no_close_sink_still_blocks_pointer_at_dim`,
  `modal_dim_with_close_sink_blocks_wheel_scroll`,
  `modal_with_no_close_sink_also_blocks_wheel_scroll`,
  `toast_body_click_does_not_reach_content_beneath` and its RTL variant).
  Demonstrated failing originally by RFC-084 and again this session (the
  no-sink dim cases, by removing their `opaque()` wrapper).
- **Keyboard dismissal (Escape)** — guarded from 0.55; see Law 7's
  checked note.
- **Keyboard blocking, where focus goes, the page's state** — guarded
  (RFC-119), in `crates/snora/tests/modal_keyboard.rs`, on both renderers:
  Tab never finds a covered control; Enter on a control focused before the
  dialog opened publishes nothing, with and without a redraw between; a
  menu without a backdrop leaves the page reachable; focus returns, and the
  application's own focus wins; no ring under the dim, compared as whole
  frames; a covered overlay is not drawn; scroll offset and a text input's
  focus survive. Fourteen breakage rows, each shown failing.
  `render_semantics.rs` is unmodified.
- **Frame-level zone navigation** — guarded.
  `modal_open_suspends_cycling`, `menu_alone_does_not_affect_cycling`,
  and `both_modal_and_menu_open_modal_wins`
  (`crates/snora-core/src/focus.rs`) match this row's claims exactly.
  Demonstrated failing this session by removing `next_zone`'s
  modal-suspend guard: both the suspend test and the modal-wins-over-menu
  test caught it.
- **Tab inside an open modal** — guarded: the Tab order is dialog, sheet,
  toasts (`modal_keyboard.rs`).

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

### Escape dismissal recipe (by hand; see Law 7 for what the engine does)

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

**Nothing to wire for snora's overlays** (Law 7): Escape publishes an overlay's escape
message, or else its click-outside close message.

```rust,ignore
let layout = snora::AppLayout::new(body)
    .on_close_modals(Message::CloseModals)   // a click outside, and Escape
    .on_close_menus(Message::CloseMenus)
    // Optional: Escape alone, for a modal that a stray click must not close.
    .escape(snora::keyboard::Escape { modal: Some(Message::CancelDelete), menu: None });
```

| Open | Escape publishes |
|---|---|
| a dialog or sheet, and a menu beneath | the modal's message (the top layer first) |
| a menu | the menu's message |
| an overlay with neither message | nothing |
| nothing | nothing |

`dismiss_on_escape`, the recipe this section used to show, is **deprecated since 0.57.0
and removed in 0.60.0**. For an overlay of your own outside `AppLayout`, match the Escape key
in your `update`; the [Keyboard map](../guides/keyboard.md#escape) shows it.

## What Snora does not do

- **Keys beyond Law 7 and the shortcuts** — Snora answers Escape for its
  overlays (Law 7), Tab through `traverse`, and the shortcuts the
  application opts into (RFC-116 R-2, the [Keyboard map](../guides/keyboard.md)).
  Every other key is the application's, and snora installs no subscription.
- **A strict focus trap** — Tab inside an open modal is bounded to the modal
  and the toasts (Law 8, RFC-119), with one empty press between the last
  control and the first, which is iced's `focus_next`. A wrap with no gap is
  not built. Where focus *starts* when a modal opens stays the application's:
  `widget::Id` and an iced focus operation.
- **Per-overlay close hooks** — There is no `on_close` on `Dialog` or
  `Sheet`. Use `AppLayout::on_close_modals` (Law 4).
- **Collision detection for popovers** — Not yet a Snora concept.
  (RFC-013-A is the design study.)
