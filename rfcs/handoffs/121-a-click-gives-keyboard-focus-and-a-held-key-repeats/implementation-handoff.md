# Implementation handoff — RFC-121: a click presses and does not focus; a held key presses once

**RFC.** `rfcs/done/121-a-click-gives-keyboard-focus-and-a-held-key-repeats.md`.
Read F-1 to F-3 and the Rulings.
**Release target.** 0.56.0, **first in it.** The owner wants 0.56.0 as soon as
possible, so this is **one unit, one review**, with a stop condition for R-2.
**Branch.** `rfc121-click-focus`, off `main`. Merge after review.

## 1. Failing first

Before changing `control`, write the tests that 0.55.0 fails, and record their output
on `main`:

| Test | 0.55.0 does |
|---|---|
| click a control; nothing is focused afterwards (`find_focused`) | the control is focused |
| click a control; its frame has **no ring**, read from pixels, both renderers. Premise: Tab to it, and the ring is there | a ring |
| click a control, then press Space, then Enter: **one** message in all, the click's | three |
| Tab to a control, then send `KeyPressed { repeat: true }` for Enter three times after the first press: **one** message | four |
| the same for Space | four |

Put them in a new `crates/snora/tests/pointer_and_keys.rs`, and add the control as a
subject the way `every_control.rs` does. **They must cover every snora control, not one,**
because F-1 was in `control` and every control is one.

## 2. The fix

- **R-1:** `control.rs:511` no longer focuses on a pointer or touch press. A press
  elsewhere still unfocuses (`:490`). Update the comment, and the module doc.
- **R-3:** `control.rs:537` presses only when `repeat` is `false`.
- **R-2, the starting point. Try it; stop if it is not small.**
  - **The rule:** after a click on a control, Tab moves to the control after it, and
    Shift+Tab to the one before it. After a press anywhere else, Tab starts from the top.
  - **The lead:** a click marks the control as the starting point in its own state, and
    the press elsewhere clears it, as `:490` does for focus. `traverse` then runs a custom
    operation, through `iced_runtime::task::widget(operation)` (public in `iced_runtime`,
    `task.rs:423`). That operation focuses the target after the marked one, or acts as
    `focus_next` when nothing is marked and nothing is focused.
  - `iced_runtime` would move from `snora`'s dev-dependencies to its dependencies. **That
    is an edge, not a package:** it is already in the lockfile, under iced. Confirm it with
    `cargo tree`, and confirm that `scripts/check-opt-out-probe.sh` still passes.
  - **Stop condition:** if it needs iced's `advanced` feature, or more than this one
    operation and a state flag, do not build it. Report why, and ship R-1 and R-3. The
    architect records R-2 in RFC-116's long-term plan, as Q-2 ruled.
  - If built, its tests: click B, then Tab, and C is focused; click B, then Shift+Tab, and
    A; click empty space, then Tab, and the first control.

## 3. Existing tests

Several existing test files press the pointer: `focus_ring.rs`, `every_control.rs`,
`sidebar_keyboard.rs`, `link.rs`, `menu_dropdown.rs`, `tab_bar_edges.rs`,
`side_bar_fit.rs`, and `control`'s own `behaviour_tests.rs` and `sizing_tests.rs`.
**Any test that relied on a click to give focus is reported to the architect, with what
it was asserting. Do not edit it.** Most press to activate, not to focus, and need no
change. `render_semantics.rs` (gate 5) must pass unmodified.

## Breakage rows

Make named `cp` backups, apply the break, run, restore, and finish with `sha256sum -c`.
**Never `git checkout` or `git restore`.** Before relying on a row, check that it *can*
fail.

| Assertion | Breakage |
|---|---|
| no focus after a click | restore the focus at `:511` |
| no ring after a click | the same break; the pixel test fails, and its premise test still passes |
| Space after a click presses nothing | the same break |
| a repeat presses nothing | press on `repeat` again |
| R-2, if built: Tab after a click lands after the clicked control | ignore the starting point |

## Docs

- **Rustdoc:** `control`'s module doc says what a click does and does not do, and
  `traverse` states the starting point, if built.
- **CHANGELOG `[Unreleased]`, Fixed:** a click gave a control keyboard focus (a ring
  after every click, and Space or Enter pressing it again), and a held key repeated the
  press. Both came in with 0.55.0.
- The book and the 0.54 → 0.55 guide's correction are the architect's; the correction
  is already on `main`.

## Acceptance criteria

1. Section 1's tests failing on `main` (output recorded), then passing.
2. Every breakage row shown failing.
3. The section 3 report, even if it reports nothing.
4. All of `ci.yaml`'s Rust quality gate commands, the Docs job's commands, the feature
   matrix, both rendered jobs and parity. Then the branch CI.
