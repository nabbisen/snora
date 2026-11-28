# RFC-143 — A toast's intent, and a notice's tone, are shown by colour alone

**Status.** Proposed, 2026-11-28.
**Raised.** 2026-11-27, the 0.67 audit's request A (row A2-096): the icons guide said a toast shows its intent by its
background colour *and* the text around it. Two tests assert that only colour changes. The owner asked for this RFC
for 0.67 on 2026-11-28.
**Release target.** 0.67.0.
**Priority.** WCAG 1.4.1 (Use of Colour, Level A, the most basic level), on the default path of every snora
application that shows a toast. It also bears on the people the owner's themes work is for.

---

## The finding

- **What a toast changes with its intent** (Info, Success, Warning, Error, Debug): its fill and its text colours.
  Nothing else: no mark, no shape, no border width. `toast/channel_register.rs` and
  `snora-widgets/src/design/channel_register.rs` assert exactly this (`toast_surface_varies_by_colour_only`, and the
  rest). A design notice's `Tone` is the same.
- **Who that fails.**
  - About 1 man in 12 cannot tell red from green.
  - An ageing lens dims blue and violet.
  - Diabetes often brings a blue-yellow loss.
  - In greyscale, or on a low-contrast display, Success and Error can look alike.

  A toast's title says what happened only if the application wrote it that way.
- **RFC-093 (2026-09-12) ruled Q-1 "no cue".**
  - Its reasons: the gap was closed by telling consumers (`accessibility.md` says the prefabs use colour alone); the
    beneficiary was "a future adopter who has not arrived"; and the right design was not obvious.
  - It pre-ruled that any later cue would be opt-in, not on by default.
  - It said Q-1 reopens when "somebody reporting that they expected the prefab to carry the channel" appears. **That
    has happened in the strongest form: snora's own book said the toast carries it.**
- **And the beneficiary has changed.** On 2026-11-27 the owner ruled that snora serves older and visually impaired
  people as a standard feature. The people a colour-only toast fails are now the people snora has said it is for.

## Proposal

**R-1. Each intent carries a mark that differs by shape, not only by colour.** It sits at the toast's start edge, and
a notice's, mirrored under RTL.
- **The marks, for a toast:** Info, Success, Warning, Error and Debug each have a different shape. They must be told
  apart in greyscale, at 1 em of the text they sit with.
- **For a notice,** the four status tones (Info, Success, Warning, Danger) take the same marks as the matching intents
  (Danger takes Error's). `Neutral` and `Accent` say no status, so they take no mark.
- **Drawn without an optional font** (RFC-139): either glyphs every common font covers, checked in the way RFC-139
  Q-3 checked "✓" and "▾", or shapes snora draws itself. Unit 0 measures both, and the review rules.
- **Its colour is the intent's text colour on its fill,** held at 3:1 like every non-text mark (RFC-135), and swept.
- **It takes no Tab stop and no press.** It is information.

**R-2. On by default.** It reverses RFC-093's pre-ruling (opt-in), for the reason above: the beneficiary is now every
snora application's users, and an accessibility default an application must find and turn on reaches almost nobody.
**This is the owner's to rule** (Q-1).

**R-3. One switch, for an application that already supplies its own channel** (orbok has since 2026-06), so that it
does not show two:
- `AppLayout::toast_marks(false)` for toasts;
- `Notice::mark(false)` for a notice.

The documentation says the switch exists for that case only.

**R-4. The book and the register change with it.**
- `accessibility.md`'s colour-alone statement becomes the new truth.
- The channel register asserts that the intent changes colour **and the mark**, so the claim stays checked.
- RFC-141's toned tags (its Q-2, which was ruled to wait for this) can then follow with the same marks.

## Open questions, with suggested rulings

**Q-1. On by default, or opt-in as RFC-093 pre-ruled?** **Suggest: on by default** (R-2). WCAG 1.4.1 is Level A, and
the owner's 2026-11-27 ruling names these users. The cost: one visible change on every default toast, named in the
migration guide, and one switch for an application with its own channel.

**Q-2. Glyphs or drawn shapes?** **Suggest: Unit 0 decides by measurement.**
- **Glyphs** are simple, but font coverage varies (RFC-139 Q-3).
- **Shapes** need no font, but iced 0.14 draws only rounded rectangles without the `canvas` feature, which limits the
  shapes.

The criteria: told apart in greyscale; at least 3:1; no optional font; legible at 1 em.

**Q-3. A textual prefix ("Error:")?** **Suggest: no.** It is not snora's to localise, as RFC-093 found. The book keeps
telling applications to say what happened in the words.

**Q-4. When.** **Suggest: 0.67.0.**

## Review against snora's principles

- **ABDD and accessibility.** The default meets 1.4.1 for the intent itself, for every user, with no work by the
  application.
- **Nobody is confused.** A shape and a colour say the same thing, and the switch says what it is for.
- **Safe.** No optional font. Contrast is held and swept.
- **Clean.** One mark per intent, shared by toasts and notices. One switch each, for one stated case.
- **Correspondence cost.** A visible change on every default toast, so a line in the migration guide for everyone,
  and a letter to orbok, whose own channel would now double up.

## Units

**Unit 0, measured (nothing committed):**
- Both candidates (glyphs, and drawn shapes) for the five intents, on both renderers and in both directions, and in
  greyscale.
- For glyphs, font coverage measured as in RFC-139 Q-3. For shapes, what iced 0.14 can draw without `canvas`.
- Each mark's ink, contrast and size. A proposal for Q-2.

**Unit 1:**
- R-1 to R-3 on both paths. The register updated (R-4).
- The sweeps take the marks in.
- **Tests, each shown failing:**
  - each intent's mark is present, and differs from the others in greyscale;
  - the switch removes it;
  - it mirrors under RTL.
- CHANGELOG: **Changed** (the default look), **Added** (the switches).
- **The book is the architect's:** `accessibility.md`, the icons guide, the toast and notice pages, and the
  migration guide.

## Acceptance criteria

1. Unit 0's measurements, and a proposal for Q-2.
2. Unit 1: each guard shown able to fail, and every `ci.yaml` command run locally.
3. The book and the register say what is now true.
