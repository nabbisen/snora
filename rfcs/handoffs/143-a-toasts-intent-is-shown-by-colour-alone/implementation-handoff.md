# Implementation handoff — RFC-143: a mark for each intent, by shape, on by default

**RFC.** `rfcs/accepted/143-a-toasts-intent-is-shown-by-colour-alone.md`. Read the finding, R-1 to R-4 and the
Rulings: on by default (reversing RFC-093's opt-in), glyphs or shapes decided by Unit 0, and no textual prefix.
**Release target.** 0.67.0.
**Order.** **Unit 0 may start alongside RFC-141 or RFC-142,** since it is measurement only. Unit 1 starts after Unit
0's review rules Q-2.
**Where.** On `main`, by the one-`main` route, **after the review approves it**. Probes are scratch, described and not
committed.

## Where the change goes

- **Toasts, on both paths:** `crates/snora/src/toast.rs` builds the toast row (the text column and the close mark).
  The mark goes at the start edge, before the text column, in the same logical row, so it mirrors under RTL.
  `snora::design::render` draws the design path's toast through the same code with a style.
- **Notices:** `crates/snora-widgets/src/design/notice.rs`. The four status tones take a mark (`Danger` takes
  `Error`'s); `Neutral` and `Accent` take none.
- **The switches:**
  - `AppLayout::toast_marks(bool)` in `crates/snora-core/src/layout.rs`, default `true`, beside `toasts` and
    `on_toast_attention`;
  - `Notice::mark(bool)`, default `true`.

  Each doc comment says the switch is for an application that already supplies its own non-colour channel, and only
  for that.
- **The registers:** `crates/snora/src/toast/channel_register.rs` and
  `crates/snora-widgets/src/design/channel_register.rs` assert that the intent changes colour **and** the mark, so the
  claim stays checked in both directions.

## Unit 0 — measured (nothing committed)

**Both candidates, for the five intents:**
- **(a) glyphs** that every common font covers (for example "i", "✓", "!", "×", and one more for Debug); measure font
  coverage as RFC-139 Q-3 did;
- **(b) shapes snora draws itself** with what iced 0.14 can draw without the `canvas` feature (rounded rectangles: a
  circle, a square, a bar, a ring…). Say what is achievable.

**For each candidate, on both renderers, both directions and both paths:**
- each mark's ink box and its contrast against the fill;
- **a greyscale rendering of the five toasts side by side,** to judge whether the marks tell them apart without
  colour. Include the image;
- what is drawn when the font lacks a glyph (candidate a), using the missing-glyph box defined by drawing it, as RFC-139
  did.

**The room it costs.** The toast is a fixed width, so a mark at the start edge takes width from the message column.
`toast_close_button.rs`'s `message_column_keeps_its_width` asserts the column keeps 0.50.0's room, and it will fail.
**Measure the cost, and propose either a wider toast or a narrower column.** The review rules; do not change the test
on your own.

**A proposal for Q-2,** with the reasons.

## Unit 1

- R-1 to R-4, with the ruled candidate, on both paths, toasts and notices.
- **The drawn sweeps take the marks in** (`nontext_sweep.rs`: the mark against its fill, every intent and tone, every
  preset). Both renderers.
- **Tests, each with a breakage row shown failing:**
  - each intent's mark is present (breakage: one intent without its mark);
  - the five differ from one another in greyscale, compared by drawn masks (breakage: two intents with the same mark);
  - the switch removes the mark, and only the mark (breakage: the switch ignored);
  - it mirrors under RTL (breakage: drawn at the left under RTL);
  - **the registers fail if the mark stops depending on the intent** (breakage: one mark for all).
- CHANGELOG `[Unreleased]`: **Changed** (every default toast and status notice now carries a mark: a visible change),
  **Added** (the two switches).

## Acceptance criteria

1. Unit 0's measurements and greyscale images, and the room cost with a proposal.
2. Unit 1: each guard shown able to fail, and every `ci.yaml` command run locally.
3. **Not yours:**
   - the book: `accessibility.md`, the icons guide's toast paragraph, the toast and notice pages, and the migration
     guide;
   - the letter to orbok, whose own channel will now double up unless they switch the mark off.

   Both are the architect's.
