# Implementation handoff — RFC-116 R-1, Unit 1: `control`'s prerequisites

**RFC.** `rfcs/accepted/116-no-snora-control-can-be-reached-from-the-keyboard.md`.
Read its "Ruling, 2026-11-07".
**Ruled.** R-1 proceeds (owner, 2026-11-07).
**Scope.** Unit 1 only: what `control` must do before it replaces `button` anywhere
else.
- Units 2 and 3, the rollout to every control and RFC-115's sizing rules, follow in
  their own handoff, after the owner rules where `control` lives (see the ruling's
  placement question).
- **Continue on `rfc116-unit0`.** It is still not merged to `main`.

**Order.** Before anything else on RFC-116. The gate fixes are already on `main`
(`6501c9d`) and merged into the branch.

## 1. Redraw on status change — first

`button` caches its status and calls `shell.request_redraw()` when it changes.
`control` recomputes status in `draw` and never asks. So a real application may not
repaint on hover or focus. **A headless snapshot cannot see this,** so test it at the
widget level:
- drive `control::update` with a cursor-enters event, and then a focus change;
- **assert that the shell recorded a redraw request each time** (`Shell::is_redraw_requested`
  or the 0.14 equivalent; name what you used);
- **breakage:** remove the request, and the assertion must fail.

## 2. `clip`, `on_press_maybe`, `on_press_with`

Each has the same meaning as on iced's `button`.
- **`on_press_maybe(None)`** is how a control becomes `Status::Disabled`. Assert that
  a disabled control:
  - ignores click, Enter and Space;
  - **is not a focus target.** Reaching a control that does nothing is the confusion
    the owner's rule forbids.
- **`on_press_with`** builds the message lazily. Assert that it is built only on
  press.
- **`clip`:** a test with overflowing content.

Give each a breakage row.

## 3. "Tab moves focus", end to end

Unit 0 named this as untestable without a runtime. **Try closing it:**
- **`iced_runtime::UserInterface`** is public, with `build` and `operate`
  (`iced_runtime/src/user_interface.rs:93,539`).
- **`iced_core::widget::operation::focusable::{focus_next, find_focused}`** are plain
  `Operation`s (`focusable.rs:165,201`).

**Steps:**
1. build the three-item rail's `UserInterface` with a headless renderer;
2. operate `focus_next`, then `find_focused`: the id is the first item's;
3. again: the second item's;
4. Shift+Tab's `focus_previous`: back to the first.

**Breakage:** drop the `Focusable` registration, and nothing is found.

If a headless renderer cannot be built that way, say exactly why, and the gap stays
named, as it is now. Do not write a test that only looks as if it covers it.

## Wgpu

Any ring or redraw sweep on wgpu runs with `--test-threads=1`, which the wgpu job
already does. Say so in each new test file's header, since Unit 0 found parallel wgpu
contexts segfault.

## Acceptance criteria

1. Each item above passing, and each breakage shown failing.
2. All twelve of the Rust quality gate's commands run locally before the push, plus
   the branch CI.
3. Nothing outside `control`, `traverse` and their tests changes.
