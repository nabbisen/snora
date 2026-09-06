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

**Checked (RFC-096, 2026-09-06): the ownership claim itself is not a
testable behavior; the recipe it recommends is guarded.** *"Snora does
not capture keyboard events"* is an absence — there is no installed
subscription to assert against, and `render()`'s output (an `Element`)
carries no subscription state at all in iced's model, so nothing in
this workspace's test harness could observe it either way. What *is*
testable and tested: `dismiss_on_escape` itself
(`crates/snora/src/keyboard.rs`), whose 7 unit tests match the state
table below exactly (modal-before-menu priority, no-op with no overlay,
no-op with no sink). Demonstrated failing this session by reversing the
modal-before-menu priority: `both_open_modal_takes_priority` caught it
immediately (`Some("menu")` where `Some("modal")` was expected).

### Law 8 — Modal focus trapping is staged, not shipped

The modal dim/backdrop provides **visual modality** and **pointer blocking**.
It does not promise keyboard focus trapping or screen-reader modal semantics.
These are distinct concerns:

| Concern | Snora provides? |
|---|:--:|
| Visual modality (dim layer) | yes |
| Pointer blocking (backdrop capture) | yes — the backdrop, the dialog, the sheet, and toasts each capture pointer input over their own bounds, so nothing beneath a modal or a toast is reachable through it (RFC-084; negative assertions in `crates/snora/tests/render_semantics.rs`). Covers both clicks and wheel-scroll. Independent of whether a close sink is provided (Law 5). |
| Keyboard dismissal (Escape) | no — application-owned (Law 7) |
| Frame-level zone navigation | **yes (RFC-060)** — `snora_core::focus::next_zone`; **suspended** while a modal is open (it reports this rather than guessing where focus inside the modal should go), unaffected while only a menu is open |
| Focus trapping *inside* an open modal | no — staged behind a separate decision (RFC-060 Q-1); see [design decisions](../contributing/design-decisions.md#why-focus-trapping-is-deferred-v014) for the constraint it inherits |

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
- **Keyboard dismissal (Escape)** — not a testable claim here; see Law
  7's own checked note, which applies identically.
- **Frame-level zone navigation** — guarded.
  `modal_open_suspends_cycling`, `menu_alone_does_not_affect_cycling`,
  and `both_modal_and_menu_open_modal_wins`
  (`crates/snora-core/src/focus.rs`) match this row's claims exactly.
  Demonstrated failing this session by removing `next_zone`'s
  modal-suspend guard: both the suspend test and the modal-wins-over-menu
  test caught it.
- **Focus trapping inside an open modal** — not covered, and
  deliberately: this row states what snora does *not* do. There is
  nothing to assert about behavior that does not exist.

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
