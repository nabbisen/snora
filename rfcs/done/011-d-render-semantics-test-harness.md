# RFC-011-D — Render-Semantics Test Harness

**Status.** Implemented (v0.11.0)
Initial implementation in v0.11.0; full matrix in v0.12.
**Tracks.** Testing / semantic hardening. Protects the engine's runtime
contract (z-stack, dismissal, toast order, RTL) that `snora-core` doctests
cannot reach.
**Touches.** `crates/snora/Cargo.toml` (dev-dependency),
`crates/snora/tests/render_semantics.rs` (new),
`crates/snora/src/toast.rs` (unit test from RFC-011-B),
`docs/src/guides/testing.md`, CI (`rust-quality` runs `cargo test -p snora`,
per RFC-011-A).

> Project-adopted version of planning-pack RFC-011-D. The planning draft
> said "evaluate `iced_test` / iced headless testing." That evaluation is
> **done and recorded here**: `iced_test` 0.14 provides a headless
> `Simulator`, was verified to run in a display-less environment, and is
> the chosen harness. This version specifies the concrete test design.

## 1. Summary

Add internal tests for the engine's render semantics — z-stack ordering,
backdrop dismissal, modal pointer-blocking, toast visibility/order, and
RTL-sensitive placement — without creating a public `snora-test` crate.

## 2. [Decision] Test technology: `iced_test` 0.14

iced 0.14 ships first-class headless testing via the `iced_test` crate
(`Simulator`). Verified facts that drive this design:

- `iced_test = "0.14"` is compatible with the pinned `iced` 0.14 and builds
  with a CPU (tiny-skia) headless renderer — **no GPU or display required**,
  confirmed by a spike in a display-less sandbox.
- `iced_test::simulator(element)` builds a `Simulator` from any
  `iced::Element`. `snora::render(...)` returns exactly that, so the engine
  output is testable directly.
- Relevant `Simulator` API:
  - `click(selector)` where `&str` selects a widget by its text content
    (e.g. a button label, or the toast close glyph `×`);
  - `point_at(Point)` + `simulate(iced_test::simulator::click())` clicks at
    an arbitrary coordinate — used to hit the full-window backdrop at a
    corner where no overlay content sits;
  - `Point` itself implements `Selector` (click a widget at a coordinate);
  - `into_messages()` yields the messages the interaction produced.
- It is added **only as a `[dev-dependencies]` of `snora`**, so it does not
  touch the public API, the feature matrix, or binary size.

### 2.1 Spike findings that shape the tests

- A text-selector click on a button reliably produces that button's
  message. (Verified.)
- A corner click hits whatever is topmost at that pixel. Because
  `render_dialog` uses `center(content)` (a full-window centering container
  that does **not** capture press events), a corner click falls through to
  the modal backdrop below it in the stack → emits `on_close_modals`.
  `Sheet` wraps its body in `opaque(...)`, so clicks on the sheet body do
  **not** fall through. These match the documented dismissal contract and
  are what the tests assert.

### 2.2 [Observation, out of scope] Dialog vs. opaque

`render_dialog` does not wrap its content in `opaque(...)`, so a click on a
non-interactive region *of the dialog card itself* also falls through to the
dismiss backdrop, unlike `Sheet`. This is a latent asymmetry, not a v0.11
RFC item; it is recorded here and flagged for a future overlay-semantics
RFC. v0.11 does **not** change dialog behavior. Tests therefore click the
dialog's interactive control (a button, by text) to assert "interactive
content is reachable and consumes the click," and click a corner to assert
"outside-click dismisses."

## 3. Goals

- Test behavior and messages, not pixels.
- Cover the highest-value z-stack and dismissal invariants.
- Keep tests robust across iced point releases (assert on produced
  messages, not widget-tree internals).
- Expose no public test API.

## 4. Non-Goals

- No screenshot/pixel tests in the first iteration.
- No public `snora-test` crate.
- No testing of application domain state or of iced itself.

## 5. Invariants to cover

| Invariant | v0.11 (initial) | v0.12 (full) |
|---|:--:|:--:|
| Body content reachable (skeleton renders, button clickable) | ✓ | ✓ |
| Outside-click on modal emits `on_close_modals` | ✓ | ✓ |
| Modal present + `on_close_modals = None` → no dismiss message, content still renders | ✓ | ✓ |
| Toast dismiss button emits `DismissToast(id)` even while a modal is present (toasts above modal) | ✓ | ✓ |
| Toast newest closest to anchor (order policy) | ✓ (unit, RFC-011-B) | ✓ |
| Menu present + `on_close_menus` emits it on outside click | — | ✓ |
| Dialog+sheet coexistence; sheet above dialog | — | ✓ |
| RTL mirrors logical Start/End (sidebar / sheet / toast anchor) | ✓ (helper-level) | ✓ |

## 6. Internal design

### 6.1 Dependency

`crates/snora/Cargo.toml`:

```toml
[dev-dependencies]
iced_test = "0.14"
```

### 6.2 File layout

Start with a single file; split per the project's ~300-ELOC rule if it
grows:

```text
crates/snora/tests/render_semantics.rs
```

If it later exceeds the line budget, split into
`tests/render_semantics/{mod.rs,overlay.rs,toast.rs,direction.rs}` with a
`tests/common/` for shared builders (kept out of the public API).

### 6.3 Harness shape

```rust
use iced::{Element, Point, widget::{button, text}};
use iced_test::simulator;
use snora::{AppLayout, Dialog, Toast, ToastIntent, render};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    BodyPressed,
    CloseModals,
    DialogOk,
    DismissToast(u64),
}

fn labeled<'a>(label: &'static str, msg: Msg) -> Element<'a, Msg> {
    button(text(label)).on_press(msg).into()
}
```

### 6.4 Initial v0.11 test cases

1. **Body reachable** — `AppLayout::new(labeled("body", BodyPressed))`,
   render, `click("body")`, assert `[BodyPressed]`.
2. **Outside-click dismisses modal** — dialog present,
   `on_close_modals = CloseModals`; `point_at(Point::new(4.0, 4.0))` +
   `simulate(click())`; assert messages contain `CloseModals`.
3. **Dialog control reachable** — same layout; `click("OK")` (the dialog's
   button); assert `[DialogOk]` and *not* `CloseModals`.
4. **No close sink → no dismissal, content renders** — dialog present,
   `on_close_modals = None`; corner click; assert **no** `CloseModals`;
   `find("OK")` still succeeds (content rendered).
5. **Toast above modal** — dialog present + a toast whose close button is
   the glyph `×` with `on_dismiss = DismissToast(7)`; `click("×")`; assert
   messages contain `DismissToast(7)` even though a modal is present.

Toast **order** policy is covered by the `render_order_for` unit test in
`toast.rs` (RFC-011-B §8); RTL Start/End resolution is covered by the
existing `LayoutDirection`/`horizontal_align`-style helper tests at this
stage. Interaction-level RTL placement assertions are deferred to v0.12.

### 6.5 No public test API

No helper is exported from `snora`. Shared builders, if needed, live under
`tests/`. The only "semantic helper" in production code is
`render_order_for` (RFC-011-B), justified as render policy, not test access.

## 7. Documentation changes

`docs/src/guides/testing.md`: add a section "What Snora tests internally vs.
what applications should test" — Snora covers engine render semantics via
`iced_test` in its own test target; applications still test their own
`update` state transitions at the data layer. Reiterate that Snora ships no
public `snora-test` crate.

## 8. CI

`cargo test -p snora` runs these tests; that step is added to the
`rust-quality` job in RFC-011-A (this is why 011-A's job list deviates from
the planning draft).

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| `iced_test` is young. | Assert on produced messages, not widget internals; pin to iced 0.14. |
| Headless renderer unavailable in some CI. | Verified CPU/tiny-skia headless works without a display; ubuntu-latest is fine. |
| Brittleness across iced versions. | Behavior/message assertions; revisit on the next iced major (a 1.0 gate anyway). |
| Test harness tempts public exposure. | Dev-dependency only; helpers under `tests/`. |

## 10. Acceptance criteria

**Initial (v0.11.0):**

- `crates/snora/tests/render_semantics.rs` exists and runs under
  `cargo test -p snora` in CI.
- Toast order policy covered (unit, RFC-011-B).
- At least one overlay-dismissal behavior covered (cases 2 & 4).
- Toast-above-modal dismissal covered (case 5).
- Testing guide updated.
- No public test crate or test-only public API.

**Full (v0.12):**

- z-stack, menu dismissal, dialog+sheet coexistence, and interaction-level
  RTL placement all have regression coverage.
