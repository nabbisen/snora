# RFC-122 — An application's own content cannot be a keyboard-reachable control

**Status.** Proposed 2026-11-09. Awaiting the owner's rulings on Q-1 to Q-5.
**Raised.** 2026-11-09, architect. RFC-116's long-term plan item **L-4** has its trigger,
"the first adopter that asks". aaai asked on 2026-11-09
(`.git-exclude/upstream/receive/0.57.0/aaai-to-snora-2026-11-09.md`). The plan says a fired
trigger becomes an RFC, with nobody waiting for a re-read.
**Release target.** 0.58.0 (Q-5).
**Priority.** Keyboard compatibility is required (owner, 2026-11-04), and an adopter is
blocked on this.

---

## The finding

From 0.55, every control **snora** draws is keyboard-reachable: it is `control`, now in
`snora-interaction`. **An application's own pressable content is not.**
- An application can press its own content only through iced 0.14's `button`, which takes
  no keyboard input and registers no focus target.
- `design::button::{primary, secondary, ghost, danger, link}` take **a plain string**.
- `snora-interaction` holds `Control`, which takes any content. Its docs say, rightly,
  that applications must not depend on it.

**aaai's three cases,** which their keyboard RFC cannot reach:
- **a glyph beside a label** in a button;
- **a file-tree row**, pressable as a whole;
- **a suggestion whose text is data.**

Each is pointer-only today. aaai would "record them as a named gap until iced closes it",
and iced has not.

This was foreseen: L-4, "No public control for an application's own content", with
`result-card.md`'s card-as-a-target as the book's own example. **The trigger has fired.**

## Proposal

**R-1. A public control for any content, through `snora`, with iced `button`'s builder
shape.** An application moving off `button` changes one name and keeps its style closures.

```rust,ignore
// Before: iced's button, pointer and touch only
iced::widget::button(row![glyph, text("Open")])
    .on_press(Message::Open)
    .padding([6, 12])
    .style(move |theme, status| my_style(theme, status))

// After: keyboard-reachable, everything else the same
snora::control(row![glyph, text("Open")])
    .on_press(Message::Open)
    .padding([6, 12])
    .style(move |theme, status| my_style(theme, status))
```

- **Builder:**
  - `on_press`, `on_press_maybe` (where `None` is disabled and skipped by Tab) and
    `on_press_with`;
  - `padding`, `width`, `height` and `clip`;
  - `style(fn(&Theme, button::Status) -> button::Style)`, taking iced's own types, so
    every existing closure and `snora::design::style::button::*` works unchanged.
- **What it does, with nothing to set:**
  - it is a focus target, with Tab through `traverse`;
  - Enter and Space press it once, publishing what a click publishes;
  - a click presses it and does not focus it (RFC-121);
  - it draws a focus ring at ≥ 3:1;
  - it takes RFC-115's target floor;
  - it is out of reach beneath an overlay (RFC-119);
  - focus follows R-1 of RFC-120 when it goes away.

**R-2. A design-path entry, with the tokens' ring and targets:**
`snora::design::control(&tokens, content)`. It is the same builder, with:
- the ring from `Tokens::focus`;
- the floor from `Tokens::targets.min`, so aaai's `targets.min = 44.0` reaches its own
  controls too;
- a default style of `ghost`, overridable with `.style(…)`.

**R-3. Nothing internal becomes public.** `snora-interaction`'s `Control` stays internal.
The public type wraps it, and exposes only R-1's builder. Not exposed:
- the tag and span declarations of RFC-116 R-2 and RFC-120: the span declaration stays
  *future wanted*;
- `min_target` and `text_size`, which the entry points set;
- the ring's internals.

So the public API commits only to what an application needs, and snora's internals stay
free to change.

## Open questions, with suggested rulings

**Q-1 — expose it at all?** **Suggest yes.**
- Keyboard compatibility is required.
- An adopter is blocked, and the long-term plan's own trigger has fired.
- Without it, every application keeps a class of controls that the keyboard cannot reach,
  and that snora's book tells them to "give a keyboard shortcut". That is a workaround,
  not a route.

**Q-2 — iced `button`'s shape, or a new one?** **Suggest iced's.**
- A reader moving from `button` changes one name, and nothing else surprises them.
- Style closures, and the style bridge's `button::*` functions, work unchanged.
- A new vocabulary would be one more thing to learn and to misread.

**Q-3 — where?** **Suggest `snora::control` (engine path) and `snora::design::control`
(design path).**
- These sit beside `snora::render` and `snora::design::render`, which already split the
  two paths that way.
- `snora::control` needs no feature, because `snora-interaction` is always present.
- So an application on the engine path alone has it without `widgets`. aaai, on the design
  path, uses `snora::design::control`.

**Q-4 — the design-path default style.** **Suggest `ghost`:** the quietest
`design::button` variant. Arbitrary content, such as a tree row or a suggestion, should
not look like a call to action unless the application says so. A button with a glyph that
should look like `primary` sets `.style(…)` with the bridge's `button::primary`.

**Q-5 — release.** **Suggest 0.58.0**, with Unit 0 first: aaai's three cases built on the
prototype, measured for the keyboard probe, ring contrast and target floor, on both
renderers.

## Review against snora's principles

- **Nobody is confused.** The shape is iced's `button`, which every iced user knows, under
  a name that says what snora adds. The rules are the same as for every snora control:
  the keyboard map applies unchanged.
- **Safe.** Every safety rule already shipped applies with no setting: a click does not
  focus, a held key presses once, nothing is reachable beneath an overlay, and focus
  goes to a neighbour after a removal. An application cannot get the keyboard behaviour
  half on.
- **Clean.**
  - A thin public type over the internal one, exposing only the builder (R-3).
  - The internal crate stays internal, and its docs stay true.
  - **Nothing is superseded:** iced's own `button` is iced's, not snora's to deprecate.
    The book's advice changes from "give it a keyboard shortcut" to "use
    `snora::control`".
- **Robust.** Unit 0 builds aaai's actual cases before the API is final.
- **ABDD.** Layout and direction are the content's. The control adds the floor and the
  ring, as for every snora control.
- **Keyboard compatibility:** this closes L-4.

## What this is not

- **Not a new widget set.** The content is the application's.
- **Not the public structure declarations** (RFC-120's future wanted).
- **Not a change to `design::button`.** The plain-string set stays, for the common case.

## Acceptance criteria

1. **Unit 0:** aaai's three cases on a prototype of R-1 and R-2. Each passes
   `every_control.rs`'s probe, has a ring of at least 3:1 in every preset and stock
   theme, and meets the target floor (44 with `targets.min = 44`), on both renderers.
2. **Unit 1:** the public types, with a breakage row per property, and `every_control.rs`
   taking `snora::control` and `snora::design::control` as subjects.
3. **The book,** by the architect:
   - `buttons.md` and `semantic-accessibility.md`;
   - `result-card.md` (the card becomes a control);
   - the keyboard map;
   - the accessibility guide's "your own buttons" bullet, which stops saying "pointer
     and touch only".
4. **The reply to aaai,** after the ruling.
