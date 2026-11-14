# Implementation handoff — RFC-130: a chord is not a press

**RFC.** `rfcs/accepted/130-a-focused-control-takes-ctrl-enter-from-the-application.md`. Read the
finding, R-1 to R-3, and the Rulings.
**Release target.** 0.62.0. **Start after the audit follow-up's Unit 3 (round 2) lands.** Its
`a_focused_snora_control_leaves_every_other_key_alone` is the test this extends.
**Branch.** `rfc130-chords`, off `main`. One unit, with no Unit 0: the finding is measured
(Unit 3's finding 2), and the change is one predicate. Land as `main` requires:
1. rebase;
2. push to a new branch name;
3. CI green, parity included;
4. `git push origin <sha>:main`.

## The rule

> **A focused control presses on Enter or Space when neither Ctrl, Alt nor the logo key is held.**
> Shift does not count. With Ctrl, Alt or the logo key held, the key is not a press: the control
> publishes nothing and **does not capture it**, so it reaches snora's router and the
> application's `keyboard::listen()`. A held key still presses once (RFC-121).

## What exists

- `crates/snora-interaction/src/control.rs:655-677`: the `KeyPressed` arm. `activates` matches
  the key alone, and the arm both publishes and captures.
- `crates/snora-interaction/src/inert.rs:516-526`: `publishes`, a second copy of the same
  `matches!`, which decides whether a key may have removed a control (RFC-120).
- `crates/snora/tests/pointer_and_keys.rs`: Unit 3's every-subject key test. **Start from it.**
- The rustdoc to correct: `crates/snora/src/control.rs:46, :77, :207, :233`, and
  `design::control`'s, wherever they say "Enter and Space press it".

## The work

1. **One predicate,** `pub(crate)` in `snora-interaction`: from the key and the `modifiers` of
   `KeyPressed` to "this is a press". Both `control.rs` and `inert.rs` call it. Neither keeps its
   own `matches!`.
2. **`control.rs`:** a chord neither publishes nor captures. **Check the order:** today the repeat
   branch captures a held plain key. Keep that for plain keys only, so that a held `Ctrl+Enter` is
   not captured either.
3. **The tests, on every subject in `pointer_and_keys.rs`, focused:**
   - Enter and Space, plain and with Shift: one message each;
   - with Ctrl, with Alt and with the logo key (each alone, and Ctrl+Shift): no message, **and the
     key reaches an application listener**. Assert the arrival, not only the silence: a control
     that drops the key without capturing it would pass a silence-only test;
   - a held `Ctrl+Enter` (with `repeat: true`): no message, not captured.
4. **The removal rule:**
   - a plain Enter that removes a control is still observed;
   - a `Ctrl+Enter` publishes nothing and triggers no observation.

   A test in `inert/tests.rs`, or beside the RFC-120 tests.
5. **The rustdoc** at the places listed above, saying the rule in the RFC's words.
6. **CHANGELOG `[Unreleased]`, Changed:** the rule, with the migration line from Q-2: *"if you
   relied on Ctrl+Enter pressing a focused control, bind the action to plain Enter."*

## Breakage rows (each must fail a named test)

1. **Today's predicate:** modifiers ignored. Fails the chord tests on every subject, and the
   arrival assertion.
2. **Shift counted as a command modifier:** fails the Shift rows.
3. **A chord captured without a message:** fails the arrival assertion, though the silence
   assertion passes. This row proves the arrival assertion is needed.
4. **`inert.rs` keeps its own copy, still ignoring modifiers:** fails the removal test's
   `Ctrl+Enter` half. If it cannot fail it, say so, and say why one predicate is still the right
   structure.
5. **A held chord captured:** fails the repeat row.

## Acceptance criteria

1. Every breakage row shown failing, with named `cp` backups and `sha256sum -c`. Do not use
   `git checkout` or `git restore`.
2. `render_semantics.rs` is unmodified.
3. All of `ci.yaml`'s Rust quality commands and the claims step, the Docs job, the feature
   matrix, both rendered jobs, and parity. Then the branch CI.
4. **The book is the architect's:** `guides/keyboard.md`, and the migration guide at the cut.
