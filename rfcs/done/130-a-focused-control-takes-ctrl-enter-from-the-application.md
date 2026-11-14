# RFC-130 — A focused control takes Ctrl+Enter from the application

**Status.** Done — shipped v0.62.0. Accepted 2026-11-14 (owner), every open question ruled as suggested (see Rulings).
**Handoff.** [`handoffs/130-…`](../handoffs/130-a-focused-control-takes-ctrl-enter-from-the-application/implementation-handoff.md).
**Raised.** 2026-11-14, architect, from the 0.62 audit follow-up's Unit 3. While asserting K-033
("a control uses Enter and Space, and nothing else"), the dev team found that `Ctrl+Space` presses
a focused control on every subject.
**Release target.** 0.62.0 (Q-2).
**Priority.** A keyboard shortcut that silently stops working whenever a snora control has focus.

---

## The finding

A focused snora control presses on Enter or Space **whatever modifiers are held**, and it
**captures** the key (`crates/snora-interaction/src/control.rs:655-677`). The match is on the key
alone:

```rust
let activates = matches!(key.as_ref(), Key::Named(Named::Enter | Named::Space));
```

**So with a control focused, `Ctrl+Enter`, `Alt+Enter`, `Ctrl+Space` and the logo-key chords:**
- **press the control:** its message is published, as if plain Enter had been pressed;
- **are captured:** snora's router acts only on what nothing captured (`router.rs:309`), and so does
  an application's `keyboard::listen()` (`router.rs:24-25`). The application never sees the chord.

**Where it matters:**
- `Ctrl+Enter` is a common "send" or "submit" shortcut.
- An application that binds it works while a text field has focus. **It stops working** after the
  user tabs to any snora button, and the button presses instead: two surprises from one key.
- Nothing tells the user or the developer why.

**It also contradicts what snora already promises for fields.** The audit's R-049 test (Unit 3)
asserts that "a shortcut works in a field that does not use its key". A control that takes every
chord on Enter and Space is the one place where that does not hold.

## Proposal

**R-1. A focused control presses on Enter or Space with no command modifier.**
- **A chord with Ctrl, Alt or the logo key is not a press.** The control neither publishes nor
  captures it, so it reaches snora's router and the application's `keyboard::listen()`, as it does
  when nothing is focused.
- **Shift does not count as a command modifier** (Q-1). `Shift+Enter` and `Shift+Space` still
  press.

**R-2. One predicate, used by both places that read the key.**
- `control.rs` decides whether to press;
- `inert.rs:516-526` decides whether a key may have published, so that it can observe a removal.

Today these are two copies of the same `matches!`. If they drift apart, a chord that presses
would go unobserved by RFC-120's removal rule, or the reverse. They become one function.

**R-3. The rustdoc and the book say it**, where they now say "Enter and Space press it":
- `snora::control` (`control.rs:46, :77, :207, :233`);
- `design::control`;
- the keyboard guide (`guides/keyboard.md`).

## Open questions, with suggested rulings

**Q-1 — does Shift count?** **Suggest: no; Shift+Enter and Shift+Space still press.**
- Shift is not a command modifier. None of snora's routes pairs it with Enter or Space.
- A Shift latched by Sticky Keys should not silently disable every control: that would be the
  same surprise as today's, aimed at a different user.
- The sentence stays one a reader can predict: "Ctrl, Alt or the logo key, and it is not a press".

**Q-2 — 0.62.0, or later?** **Suggest: 0.62.0.**
- The change is small: one predicate and its tests.
- It closes the gap the audit found, in the release the audit belongs to.
- The behaviour change is only that a chord stops pressing. The migration guide says so in one
  line: *"if you relied on Ctrl+Enter pressing a focused control, bind the action to plain Enter."*

## Rulings, 2026-11-14 (owner)

The RFC is accepted as proposed:
- **Q-1:** Shift does not count. `Shift+Enter` and `Shift+Space` still press. A chord with Ctrl,
  Alt or the logo key is not a press, and is not captured.
- **Q-2:** 0.62.0.

## Review against snora's principles

- **Nobody is confused.** A key the application bound keeps doing what it was bound to, whatever
  has focus.
- **Predictable.** One sentence covers it: Enter or Space, without Ctrl, Alt or the logo key.
- **Robust.** One predicate in place of two copies. Every modifier is tested on every subject, with
  each half shown able to fail.
- **Keyboard.** The plain keys are unchanged. Only chords, which a control has no use for, are given
  back.
- **ABDD.** No direction is involved.

## What this is not

- **Not a change to repeat handling** (RFC-121): a held key still presses once.
- **Not a change to text fields.** They take what they use (R-050).
- **Not a new API:** no option to choose the modifiers.

## Acceptance criteria

1. **Every subject in `pointer_and_keys.rs`, focused:**
   - Enter and Space, plain and with Shift, press once;
   - with Ctrl, with Alt and with the logo key, they publish nothing and reach an application
     listener.

   Each assertion is shown able to fail.
2. **The removal rule (RFC-120) still observes a plain Enter press.** A Ctrl+Enter that presses
   nothing triggers no observation.
3. **The rustdoc, the book's table, and a CHANGELOG `Changed` entry** with the migration line.
