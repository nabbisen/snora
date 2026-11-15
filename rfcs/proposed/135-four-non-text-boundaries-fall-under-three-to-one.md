# RFC-135 — Four things snora draws fall under 3:1 against what they sit on, and no test measures a glyph

**Status.** Proposed 2026-11-15, for the owner.
**Raised.** 2026-11-15, architect, from the drawn-contrast Unit 0, the non-text half (ruled in
0.63's scope, owner, 2026-11-14).
**Release target.** 0.63.0 (Q-3).
**Priority.** WCAG 1.4.11, which orbok's conformance record cites. One of the four, a progress
track, is invisible on both high-contrast presets.

---

## The finding

The drawn-contrast Unit 0 measured every non-text element snora draws on the design path, from
frames supersampled at 4x, on tiny-skia and wgpu (no verdict differs), in every preset, on every
neutral surface:
- **Glyphs** (every icon, the `×` of a chip, a notice and a toast, the breadcrumb separator):
  **all at 3.12:1 or more.** None fails.
- **The focus ring:** 6.19:1 at the lowest, over 204 cells.
- **Most boundaries pass** (borders, rules, underlines, the active tab's indicator, the selected
  chip, the cards).

**Four do not:**

| What | Lowest | Where | Why |
|---|---|---|---|
| **The progress track** against what it sits on | **1.00:1** | every preset; **invisible on both high-contrast presets** (all 144 cells under 3) | the track is `surface`, with no border. The filled bar is visible (3.12 or more), the remainder is not |
| **A `Debug` toast's fill** against the page | 1.00 to 1.20:1 | all six presets | the fill is `surface_raised`, with no border. The other four intents are at 5.02 or more |
| **The scrollbar** of a menu taller than the window | 1.02 to 1.35:1 | all six | iced's default scrollable style; snora sets none |
| **`card::raised`'s border** against its own drop shadow | 2.75:1 | `light`, `light_aaa` | the card's fill is the page's colour there, so the border is the only boundary, and the shadow under it darkens the page by 6% |

**And nothing tests any of this.** Today's guards measure the focus ring and `secondary`'s border
on pixels, and the `border` role as a token pair. No glyph and no other boundary is measured as
drawn.

## What is not a gap, and why

These are written down, so the rule is predictable and nobody "fixes" what holds:
- **The dialog's and the sheet's border against the dimmed page** (1.05 to 2.40): their **fill**
  against the dimmed page is 3.23 or more, and the fill is the boundary a reader sees.
- **Hover and press tints:** WCAG 1.4.11 does not ask a hover tint for 3:1. RFC-129's cue carries
  the state: the label's colour, and ΔE76 ≥ 3.
- **The drop shadow:** decorative.
- **A tooltip's border over coloured content:** the tooltip floats over whatever is there, and is
  content, not a control. Its text holds its level (RFC-131).
- **Disabled:** exempt, as everywhere.

## Proposal

**R-1. The rule:** every non-text element a reader needs in order to see a control or its state
holds **3:1** against what is beside it, on the drawn pixels. That means a glyph that is not a
word, a boundary, a track, a mark or a scrollbar.
- **A glyph is measured by M-2:** "a glyph holds 3:1 if the pixel furthest in contrast from the
  background beside it, drawn at twice the pixel density, is at least 3:1."
- **A boundary is the component's visible extent:** its fill or its border, whichever is the
  stronger, against what is beside it.
- Words are text, held to the palette's level (RFC-131). The split is WCAG's own (1.4.3 against
  1.4.11).

**R-2. The four fixed, by the smallest change that reaches 3:1 and moves nothing that passes:**
- **the progress track:** a boundary at 3:1, as a border or a track colour (Q-1);
- **a `Debug` toast:** a border at 3:1, as the dialog card has, with its fill unchanged;
- **snora's own scrollables** (the menu dropdown): a scrollbar style whose thumb holds 3:1 against
  its track and the page;
- **`card::raised`:** its border clears 3:1 against the shadowed page, or the shadow stops under the
  border. Unit 1 measures which moves less.

**R-3. A test,** with the drawn-contrast sweep's machinery:
- every non-text subject of the inventory, both renderers, both directions, at 3:1;
- the four tripwires (a frame run not in the table; a text primitive not announced; a design widget
  not in the sweep; a `fill_quad` with no finder), each with its plant.

## Open questions, with suggested rulings

**Q-1 — the progress track: a border, or a darker track?** **Suggest: Unit 1 measures both and
takes the one that changes the bar less.** A border keeps the track's colour and the bar's
contrast. A darker track may lower the filled bar's own contrast against it.

**Q-2 — scrollbars.** **Suggest: snora's own scrollables only.** An application's scrollable takes
its iced theme's style, as the unstyled path does, and the book says so.

**Q-3 — 0.63.0.** **Suggest: yes.** It is the scope the owner set for 0.63 ("non-text contrast at
3:1, on the pixels drawn"), the machinery exists, and the changes are styles.

## Review against snora's principles

- **Nobody is confused.** One rule for non-text, one for words, each one sentence. The exceptions
  are listed with why, so a reader can predict them.
- **Robust.** The inventory comes from frames, and four tripwires keep it complete.
- **Predictable.** Nothing that passes moves.
- **ABDD.** The sweep runs in both directions.
- **The initial cost is not the deciding factor.** The sweep costs CI time, which Unit 1 states.

## Acceptance criteria

1. **Unit 1:**
   - R-1 to R-3;
   - each of the four fixes with a breakage row;
   - nothing that passes moves (a test);
   - the CHANGELOG's Fixed entry, with "retake the screenshot baselines of" the four.
2. **The book,** by the architect: the accessibility guide's non-text statement, and the
   high-contrast page's progress-track sentence.
