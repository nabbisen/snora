# Implementation handoff — RFC-116, Unit 0 only: a focusable control, prototyped

**RFC.** `rfcs/accepted/116-no-snora-control-can-be-reached-from-the-keyboard.md`
**Scope.** Unit 0: one focusable `control`, used by the sidebar item, with its costs
measured. **On a branch (`rfc116-unit0`), not merged to `main`.** The owner rules on
Q-1's porting cost from your figures, and only then does R-1 cover every control.
**Order.** After RFC-114 lands, because it rebuilds the sidebar item that this
replaces. Read `.git-exclude/reviewed/iced-buttons-take-no-keyboard-input/` first.

## Already established

- **iced 0.14's `button`** takes no keyboard input and is not focusable. The probe is
  in that finding's `evidence/`.
- **`iced_core::Widget`** (`iced_core/src/lib.rs:84`) and
  **`iced_core::widget::operation::Focusable`** (`…/operation/focusable.rs:7`) are
  public with no feature. iced's `advanced` feature only re-exports them. **Do not
  enable `advanced`.** Depend on `iced_core` directly, at the same `0.14`
  requirement as `iced`, so that Cargo unifies the two.
- **iced binds no Tab.** Focus moves only through
  `iced::widget::operation::focus_next()` / `focus_previous()`
  (`iced_runtime/src/widget/operation.rs:50,55`).
- **`iced::keyboard::listen()` delivers only ignored events.**

## What to build (on the branch)

1. **`snora_widgets::control`, crate-private.** It wraps an `Element`, and:
   - **`operate`:** registers as `Focusable`, with an `Id`;
   - **`update`:**
     - press and release with mouse or touch, as iced's `button`;
     - **Enter or Space while focused emits the same message a click emits**;
     - a click also focuses it;
   - **`draw`:** the content, then a **focus ring** while focused. On the design
     path the ring uses `FocusTokens`; on the unstyled path, a theme-derived colour,
     your proposal.
   - **`mouse_interaction`:** as iced's `button`.
2. **`snora::keyboard::traverse(key, modifiers) -> Option<Task<Message>>`:** Tab →
   `focus_next`, Shift+Tab → `focus_previous`, anything else `None`.
3. **The sidebar item** (`Beneath`, `Beside` and `Tooltip` forms) uses `control` in
   place of `button`. Nothing else changes yet.

## What to measure and report

| Measure | How |
|---|---|
| **Keyboard probe, inverted** | Tab reaches the item; Enter and Space press it; a click still presses it. Each shown failing on `main`'s sidebar first. |
| **Focus ring contrast** | ≥ 3:1 against the rail and against the active highlight, all presets and `Theme::ALL`, drawn pixels (RFC-108's route) |
| **Nothing else moved** | the existing sidebar tests pass unmodified; `render_semantics.rs` passes |
| **Binary size** | every size probe, before and after. Report the graph change as RFC-112 did; byte-identity is not expected |
| **Compile time** | `measure-compile-time.sh`'s columns, before and after |
| **Porting surface** | every `iced_core` item `control` uses, with a note on which are most likely to change in an iced major. This is the figure Q-1 is ruled on |
| **Renderer parity** | the ring on both renderers, through the parity job |

**Also report:** anything iced's `button` does that `control` does not yet do (status
styling, padding, clip). For each, say whether R-1 must add it before replacing
`button` everywhere.

## Check each assertion can fail

Back up to named files with `cp`, break, run, restore, then `sha256sum -c`. **Never
`git checkout`/`restore`.**

| Assertion | Breakage |
|---|---|
| Tab reaches it | drop the `Focusable` registration in `operate` |
| Enter presses it | ignore keyboard events in `update` |
| it emits the same message as a click | emit a different message on Enter |
| ring drawn at ≥ 3:1 | draw the ring in the rail's colour |

## Acceptance criteria

1. Every row above shown failing, then passing, with the CI run on the branch linked.
2. The measurement table complete.
3. A recommendation, yours, on whether R-1 should proceed, and what it would still
   need.
