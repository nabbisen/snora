# Implementation handoff — RFC-127: a snora control can be given an id

**RFC.** `rfcs/done/127-a-snora-control-cannot-be-given-focus.md`. Read R-1 to R-4 and the
Rulings.
**Release target.** 0.61.0. **Before RFC-126,** which is larger, and whose Unit 0 can then
measure with ids available. Independent of the 0.61 housekeeping.
**Branch.** `rfc127-control-id`, off `main`. Unit 0 is its own review. If Unit 0 is small and
clean, the review may let Unit 1 follow on the same branch. Land as `main` requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## What exists

- `snora_interaction::Control` holds `id: Id`, set to `Id::unique()` when it is built
  (`control.rs:240`). It reports itself with `operation.focusable(Some(&self.id), …)`
  (`control.rs:489`), so iced's `focus(id)` would reach it **if the id could be chosen.**
- `snora::control` (`crates/snora/src/control.rs:149`) and `snora::design::control`
  (`crates/snora/src/design.rs:68`) are its public builders, with iced `button`'s methods. No
  `.id()`.
- iced 0.14's public tasks: `iced::widget::operation::focus(id)`, `is_focused(id)`,
  `focus_next()` and `focus_previous()`.

## Unit 0 — prototype `.id()` and measure

| Scene | Measure |
|---|---|
| a page with two `snora::control`s, the second given an id; `focus(id)` | only the second is focused; its ring is drawn (**pixels**); Enter publishes its message once; the next Tab goes to the control after it |
| `is_focused(id)` before and after | `false`, then `true` |
| **Q-1:** the `update` that opens a dialog also returns `focus(id)` for a control inside it | whether the control is focused when the dialog first draws (pixels, both renderers). If not, what does work, for example a task chained after the first frame. Say exactly what, and why |
| `focus(id)` for a page control **under** an open dialog (R-3) | nothing beneath is focused. Say whether focus stays where it was, or nothing is focused |
| **two controls given the same id**, then `focus(id)` | how many are focused. iced's `text_input` has the same exposure. Report it, and propose what the rustdoc says |
| a control given an id, then rebuilt by the next `view` with the same id | `focus(id)` still finds it; RFC-120's focus-after-removal and RFC-121's click rule behave as before |

## Unit 1

- **`.id(impl Into<widget::Id>)`** on `snora::control` and `snora::design::control`. Its rustdoc
  shows the R-4 pattern, says that the id must be unique in the window (with Unit 0's answer
  for duplicates), and that a covered control cannot be focused.
- **Nothing else public.** The `design::button` variants are unchanged (Q-2), and so are the
  prefab controls (Q-3).
- **Tests,** each with a breakage row checked first that it can fail (named `cp` backups,
  `sha256sum -c`; never `git checkout` or `restore`):

| Assertion | Breakage |
|---|---|
| `focus(id)` focuses the control with that id, and only it | the builder ignores the id |
| the ring is drawn after `focus(id)` (pixels) | — |
| Enter presses a control focused by `focus(id)`, once | — |
| Tab carries on from it | — |
| `focus(id)` beneath a dialog focuses nothing beneath | `Inert` passes `focusable` while covered |
| the dialog-opening pattern R-4 documents works as documented (Q-1) | the pattern without what Unit 0 found it needs |
| `is_focused(id)` reports it | — |

- **`every_control.rs`:** its subjects gain an id case: each subject given an id is focused by
  `focus(id)`, and its ring passes as for Tab.
- **Unmodified:** `render_semantics.rs`, `modal_keyboard.rs`, `focus_follows_removal.rs`,
  `menu_arrows.rs` and `covered_pointer.rs`.
- **CHANGELOG `[Unreleased]`, Added:** `.id()` on `snora::control` and `snora::design::control`
  (RFC-127).
- **Rustdoc, yours:** `.id()` itself, and `traverse`'s "where focus starts … is still yours
  (`widget::Id` and `operate`)", pointed at `.id()`.
- **The book, the architect's:** `overlays.md`'s checklist, and the buttons and keyboard pages.

## Acceptance criteria

1. Unit 0's table, reviewed, including Q-1's answer.
2. Unit 1: every breakage row shown failing.
3. All of `ci.yaml`'s Rust quality gate commands (16), the Docs job's commands, the feature
   matrix, both rendered jobs, and parity. Then the branch CI.
