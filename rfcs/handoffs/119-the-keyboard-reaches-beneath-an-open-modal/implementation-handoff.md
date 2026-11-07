# Implementation handoff — RFC-119: the keyboard does what the pointer can

**RFC.** `rfcs/accepted/119-the-keyboard-reaches-beneath-an-open-modal.md`. Read R-1 to
R-5 and the Rulings.
**Release target.** 0.55.0. **The release waits for this** (Q-1).
**Branch.** `rfc119-inert`, off `main` (`e9d7229` or later). Each unit is its own
review. Merge into `main` after Unit 1 is accepted.

## The rule, in one sentence

> **The keyboard does what the pointer can:** it reaches only what the pointer can
> press, and it leaves an overlay the way the pointer can.

| Open | The pointer | The keyboard, after this RFC |
|---|---|---|
| nothing | presses the page | unchanged |
| a menu **with** its backdrop (`on_close_menus`) | cannot press the page; a click outside closes the menu | cannot reach the page; **Escape closes the menu** |
| a menu **without** a backdrop | presses the page | reaches the page (unchanged) |
| a dialog or sheet, **with** `on_close_modals` | cannot press beneath; a click on the dim closes it | cannot reach beneath; **Escape closes it** |
| a dialog or sheet, **without** `on_close_modals` | cannot press beneath; nothing closes it but its own controls | cannot reach beneath; Escape is the application's (`dismiss_on_escape`), as before |
| toasts, in any case | pressable (Law 6) | reachable; Tab goes dialog, sheet, toasts |

"Beneath" means **everything under the topmost backdrop, backdrops included.** A menu
left open under a modal is covered, and so is its backdrop. An Escape that a sinkless
dim does not answer must therefore not fall through to close the menu.

## Unit 0 — reproduce, and settle Q-3's mechanism

1. **A failing test of the finding,** in a new file `crates/snora/tests/modal_keyboard.rs`
   (not `render_semantics.rs`: see "Gate 5" below). With a dialog open over a
   skeleton that has a control:
   - Tab, through the runtime's `UserInterface` as Unit 1 of RFC-116 did, reaches the
     skeleton's control;
   - Enter publishes its message.

   Both must fail once Unit 1 lands. Record the output.
2. **Prototype `Inert`, and answer three questions with measurements:**
   - **Q-3, the opener's focus.** On becoming inert: find the focused control in its
     content, unfocus it, and remember it. On becoming reachable: give it its focus back.
     - **How do you identify it?** If a control's `Id` is regenerated on every `view`,
       an `Id` cannot be remembered across frames. Then use its position in the tree, or
       something else that survives a rebuild. Say which you used, and why it survives.
     - **When does it run?** `diff` has no layout. If it runs on the first `update` after
       the flag changes, is there a frame with the ring drawn under the dim? Measure it
       on both renderers.
   - **R-5, state.** Show that a skeleton `scrollable`'s offset and a `text_input`'s
     contents survive opening and then closing a modal. Show also that they **do not**
     survive when the wrapper is inserted only while the modal is open. That is the
     breakage row, and it proves the test can fail.
   - **R-4, capture.** Show that an event the backdrop captures returns
     `event::Status::Captured` from `UserInterface::update`. That status is how
     `iced::keyboard::listen()` decides (`iced_futures-0.14.0/src/keyboard.rs`: it
     passes only `Status::Ignored`), so it is the testable stand-in for "the application
     never receives it".

**If any of the three cannot be done as R-1 to R-5 and Q-3 say, stop and report.** Q-3
in particular returns to the owner. It is not deferred quietly.

## Unit 1 — `Inert`, the backdrops, and the engine

### `Inert` (in `snora-control`)

- **Always in the tree** around each layer it may cover, switched by a flag (R-5).
  Alternatively it may be tree-transparent, delegating `tag`, `state`, `children` and
  `diff` as `container` does (`iced_widget-0.14.2/src/container.rs:236-250`). But then
  it has no state of its own for Q-3's memory, so state where the memory lives.
- **When inert:**
  - **`operate`:** does not forward `focusable` to its content. `container`, `text`,
    `scrollable` and the rest still pass, so selectors and gate 5 still find content;
  - **`update`:** drops `Event::Keyboard` and `Event::InputMethod`, and passes
    everything else: window events, redraw requests and the mouse. Pointer blocking
    stays the backdrops' job (Law 8), and an animation beneath must keep running;
  - **`draw`:** unchanged;
  - Q-3: unfocus and remember on entry, restore on exit.
- **Its public surface is only what the engine needs.** It is not for applications.

### The backdrops answer Escape (R-4)

- `transparent_backdrop` and `dim_backdrop` (`crates/snora/src/render.rs:326,345`) are
  `mouse_area`s, which take no keys. Each needs to publish its `on_press` message on
  Escape and capture the event. How is yours to choose: a small widget, or a field on
  `Inert`. `dim_without_capture` answers no Escape.
- **Order:** `stack` delivers from the top layer down and stops at a capture. So the
  modal's dim answers before a menu's backdrop, which is Law 1's priority and
  `dismiss_on_escape`'s.
- **A focused `text_input` in a dialog** handles Escape itself first
  (`text_input.rs:1235`). Assert that Escape closes the dialog only once nothing in it
  has taken the key, and that it takes no more than two presses.

### The engine (`crates/snora/src/render.rs`, from `:195`)

- The skeleton is wrapped in `Inert` **always**. It is inert while a menu backdrop or a
  modal is up.
- The menu layers, the menu backdrop included, are wrapped likewise, and are inert
  while a modal is up.
- **A menu without a backdrop** leaves the skeleton reachable.
- Toasts are never wrapped.

## Tests — `crates/snora/tests/modal_keyboard.rs`

**Each test gets a breakage row, and each row is checked first that it *can* fail.**
Both of the last two units had a specified breakage that could not.

| Assertion | Breakage |
|---|---|
| dialog open: Tab never finds a skeleton control; `find_focused` stays inside the dialog and toasts | `Inert` forwards `focusable` |
| dialog open: Enter on a skeleton control focused **before** the dialog opened publishes nothing | `Inert` passes keyboard events |
| menu with backdrop: the same two, for the skeleton | the engine skips the wrapper for menus |
| menu **without** backdrop: Tab **does** reach the skeleton | wrap regardless of the backdrop |
| sheet open: the same as for the dialog | — |
| Escape with `on_close_menus`: publishes it **once**, and `update` reports `Captured` | the backdrop ignores Escape, which is the trap; then, captured: not captured |
| Escape with `on_close_modals`: publishes it once; a menu beneath stays open | the modal's dim ignores Escape, and the menu closes instead |
| Escape with **no** `on_close_modals`, and a menu with a backdrop beneath: publishes **nothing**, and the status is `Ignored` | the menu's backdrop is left outside `Inert` |
| a skeleton scroll offset and text input's contents survive a modal opening and closing (R-5) | insert the wrapper only while open |
| Q-3: the control focused before a modal opens is focused again after it closes | skip the restore |
| Q-3: **no ring under the dim** while the modal is open, read from **pixels**, both renderers | skip the unfocus |
| Q-4: Tab order is dialog, sheet, toasts | — |

**A trap to avoid in the Q-3 ring test:** `find_focused` cannot see inside `Inert`,
because it hides focusables. A test that asks it whether the covered control is focused
passes whether or not the control is still focused. That is why the row reads the
ring from pixels.

**`every_control.rs`:** add one subject, a control under an open dialog. It is
expected **not** to be reached. Say how the harness represents that, and do not change
the existing subjects.

## Gate 5

`render_semantics.rs` must pass **unmodified** (`api-freeze-review.md`, gate 5). The
RFC's R-3 said that file "gains" assertions; that was the architect's error, corrected
here. **Write nothing in it.** Its unchanged pass is the evidence that the composition
did not move. That includes layer order, every surface, and pointer blocking.

## Docs

**Rustdoc:**
- `AppLayout::on_close_menus` and `on_close_modals` (`crates/snora-core/src/layout.rs:345,352`):
  "Also published on Escape";
- `snora::keyboard::dismiss_on_escape`: when the engine now answers Escape, and the one
  case left to it;
- `traverse`: what Tab reaches while an overlay is open.

The architect rewrites the book, including Laws 7 and 8.

**CHANGELOG, `[Unreleased]`:**
- **Fixed:** the keyboard reached beneath an open modal;
- **Changed:** an overlay that closes on a click outside also closes on Escape, and
  an application's own `text_input` beneath an open modal no longer takes keys.

## Acceptance criteria

1. Unit 0's failing test and its three answers, reviewed.
2. Unit 1: every row above passing, and every breakage shown failing (named `cp`
   backups, `sha256sum -c`; never `git checkout` or `restore`).
3. `render_semantics.rs` unmodified, and passing.
4. All of `ci.yaml`'s Rust quality gate commands (16), the Docs job's commands, the
   feature matrix, both rendered jobs, and parity. Then the branch CI.
