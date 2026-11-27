# RFC-141 — A read-only tag has no primitive, and the recipe's is drawn disabled

**Status.** Accepted 2026-11-28 (owner), every open question ruled as suggested (see Rulings).
**Raised.** 2026-11-27, the 0.67 audit's request B (row B3-043, measured), reviewed by the architect
(`.git-exclude/reviewed/audit-0-67-b/review-result.md`). The owner asked for this RFC for 0.67 on 2026-11-28.
**Release target.** 0.67.0, as an addition.
**Priority.** Not a defect in a widget. The book's advice produces text under 2:1, and every adopter with a result
list needs what the advice was for.

---

## The finding

- **`recipes/result-card.md` says a read-only tag is `chip::filter(tokens, label, true, None)`.** But `None` disables
  a chip: `filter`'s rustdoc says "Pass `None` to disable". A disabled chip is drawn at reduced contrast, which the
  text floor exempts, because nobody needs to read a disabled control.
- **Measured** (`.git-exclude/review-request/audit-0-67-b/evidence/zz_probe_b3_readonly_chip.rs`, both renderers
  agreeing to 0.03):

  | | Label against its own fill, six presets |
  |---|---|
  | the recipe's "read-only" tag (selected, `None`) | **1.52 to 1.97:1** |
  | the same chip, unselected, `None` | 2.20 to 3.77:1 |
  | the same chip, enabled and selected | 6.70 to 11.75:1 |

- **A tag carries information:** a file's type, a result's status, a category. It is not a disabled control, so the
  exemption must not apply to it.
- **snora has no non-interactive chip.** Every chip is a control, and so a Tab stop unless disabled. An application
  that wants a readable tag that is not a Tab stop must assemble one, and **snora's sweeps never see what an
  application assembles.**

## Proposal

**R-1. `snora::design::chip::tag(tokens, label)`: a chip that is information, not a control.**

```rust
row![
    chip::tag(&tokens, "PDF"),
    chip::tag(&tokens, "Shared"),
]
```

- **It is not a Tab stop, takes no press, and has no hover or pressed state.** There is nothing to do with it, so
  the keyboard does not stop on it (RFC-116).
- **It is drawn at full contrast.** Its label is held at the palette's `text_contrast` (4.5:1 on `light` and `dark`,
  7:1 on the AAA and high-contrast presets) against its own fill. Its boundary is held at 3:1 against every neutral
  surface it can sit on, as a chip's is.
- **It looks like a chip's resting state,** in the same height and padding, so a row that mixes tags and filter
  chips lines up. It never looks like a *selected* chip, because that would claim a state it does not have.
- **It takes no `direction`.** A single label has nothing to order. The direction guard's list of widgets that say
  why they take none gains this one, with that reason.

**R-2. The recipe moves to `chip::tag`,** and stops calling a disabled chip read-only. `filter`'s rustdoc gains one
line: "for a tag that is information rather than a choice, use `tag`".

**R-3. The sweeps take it in:** `text_sweep.rs` (the label, every preset) and `nontext_sweep.rs` (the boundary).
Both renderers.

## Open questions, with suggested rulings

**Q-1. The name.** **Suggest `tag`.**
- "Tag" is what the recipe already calls it, and what a reader expects of a label on a result.
- `badge` usually means a count on an icon, which this is not.
- `label` is a text role in snora's typography, so the word would mean two things.

**Q-2. Tones (a success, warning or danger tag)?** **Suggest: not now.**
- One neutral tag first. The palette's status pairs (`success`/`success_text`, and the rest) are tested, so a toned
  tag is a small later addition, and an additive one (RFC-036).
- A tone on its own would also be meaning carried by colour alone, the same question the 0.67 audit raised for the
  toast (WCAG 1.4.1). It should be ruled together with that question, not before it.

**Q-3. The engine path?** **Suggest: no.** Chips exist only on the design path today. A tag on the engine path would
be the first chip there, which is a separate question.

**Q-4. When.** **Suggest: 0.67.0,** with the audit's follow-up.

## Rulings, 2026-11-28 (owner)

"Authorized." The owner first asked for the purpose and the reasoning, and accepted them. Every question is ruled as
suggested:
- **Q-1:** the name is `tag`.
- **Q-2:** no tones now. Ruled together with the toast's colour-only intent, in 0.67.
- **Q-3:** the design path only.
- **Q-4:** 0.67.0, as an addition.

## Review against snora's principles

- **Nobody is confused.** A tag that cannot be pressed is not a Tab stop and does not look selected. A disabled chip
  is no longer offered as something to read.
- **Safe (ABDD).** The label is held at the palette's level, and the sweeps measure it in every preset. The recipe's
  1.5:1 becomes impossible to reach by following the book.
- **Clean.** One function, no new type, and the chip module's existing styles and tests.
- **Keyboard.** It is not a stop, which is correct for something that does nothing. The rule (RFC-116) is that every
  *control* is reachable, and this is not a control.
- **Correspondence cost.** None. One line in 0.67's migration guide.

## Units

**Unit 0, measured (nothing committed):**
- A prototype, in all six presets on both renderers: the label's contrast against the fill, the boundary against
  `background`, `surface` and `surface_raised`, and its height beside a filter chip.
- That it registers no focus target and publishes nothing on a click.

**Unit 1:**
- R-1, with a doc-test. R-3. The direction guard's entry.
- **Tests, each shown failing:**
  - the label at the preset's level;
  - not a Tab stop;
  - no message on a click;
  - the same height as a filter chip.
- CHANGELOG `[Unreleased]`, Added.
- **The book is the architect's** (R-2): the recipe, `filter`'s rustdoc line, and the design pages that list the
  chips.

## Acceptance criteria

1. Unit 0's measurements in every preset, on both renderers.
2. Unit 1: R-1 and R-3, each guard shown able to fail, and every `ci.yaml` command run locally.
3. The recipe and the rustdoc corrected.
