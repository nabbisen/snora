# Implementation handoff — RFC-105: caller text is silently clipped, and two properties are unasserted

**RFC.** `rfcs/accepted/105-caller-text-is-silently-clipped-and-two-properties-are-unasserted.md`
**Release target.** 0.52.0. **Start after RFC-104 is committed**: both change
`crates/snora/src/toast.rs` and `CHANGELOG.md`.
**Rulings.** Q-1: `WordOrGlyph` on fixed-width, multi-line surfaces, each
confirmed by measurement. Q-2 (a): a timing-ratio assertion, `#[ignore]`d, run in
release by one CI step.
**Touch list.** `crates/snora/src/toast.rs`,
`crates/snora-widgets/src/design/notice.rs`, new tests under `crates/snora/tests/`,
`.github/workflows/ci.yaml` (one step), `CHANGELOG.md`. **Not** `docs/`:
`performance-envelope.md` and the migration guide are the architect's; put the
numbers in your report.

## Already established

- **Measured:** 400 unbroken `a`s as a toast message render as one line,
  **288.0 × 18.2**, clipped at the message column; about 48 characters show.
- **`visible_bounds()` does not see it:** it equals `bounds()`, 288.0 × 18.2,
  because the glyphs overflow inside the text widget's own box. **Do not use it
  as the detector.** Use the string's natural (unconstrained) size against its
  rendered size, the RFC-099/100 technique.
- iced 0.14 has `Wrapping::{None, Word, Glyph, WordOrGlyph}`; the default is
  `Word`. Nothing in snora sets wrapping or shaping; shaping is `Auto`.
- **Scale, measured** (release, tiny-skia, toasts): layout 0.6 / 5.9 / 62.9 /
  656 ms for 100 / 1k / 10k / 100k. That is linear.
- **Tooltip text is invisible to `Simulator::find`** (RFC-100 §4: iced 0.14's
  tooltip overlay does not implement `operate`).

## Unit 1 — which surfaces clip (Q-1, confirmed by measurement)

For each candidate, place 400 unbroken characters in a parent narrower than the
string, and compare natural size with rendered size:

- toast title and message (fixed 340 px);
- notice title and body (width `Fill`; put it in a fixed-width parent);
- sidebar and close-control tooltips: **you cannot `find` their text.** They are
  content-sized, so they should not clip at a width. **Confirm what you can**,
  e.g. by frame-hash comparison or by reasoning from iced's tooltip layout, and
  say which. If you cannot establish it either way, leave tooltips out and record
  that as a limitation. **Do not assert something you cannot measure.**

Report each surface's result. Only surfaces that clip get `WordOrGlyph`.

## Unit 2 — the fix and its assertion (R-1, R-2)

- `.wrapping(Wrapping::WordOrGlyph)` on each surface that clips.
- **Test:** the unbroken fixture's rendered box holds its natural height for the
  wrapped width, or equivalently the rendered width does not exceed the column
  **and** the height has grown to fit. Pick whichever you can make fail cleanly,
  and say why. **Shown failing on `Word`**, passing on `WordOrGlyph`.
- Normal prose must be **unchanged**: measure the existing normal fixture's size
  before and after, and state that it matches.

## Unit 3 — verbatim (R-3)

The fixture is tekstide's criterion: a string containing a **bidi override**
(e.g. U+202E), a **newline** and a **control character** (e.g. U+0007). Pass it
into every caller-text surface in scope and **find it by exact content**, which
shows that snora hands iced the string unmodified. Combine it with Unit 2's
natural-size check, so it is also shown whole.

**Say plainly what this does not assert:** the glyphs drawn for those characters.
That is iced's and cosmic-text's rendering, and the harness exposes frame hashes,
not pixel reads.

**Check it can fail:** a scratch edit that trims or replaces the string inside the
component must break it.

## Unit 4 — bounded work (R-4, Q-2 a)

- An `#[ignore]`d test, `crates/snora/tests/bounded_work.rs`: lay out N and 10N
  toasts (choose N so the smaller run is stable, e.g. N = 1,000). Take the
  **minimum of several runs** for each size, and assert
  `t(10N) / t(N) < 30`; linear is about 10.
- **One CI step** runs it in release: `cargo test --release -p snora --test
  bounded_work -- --ignored`. Run `check-workflows.sh` on the edit.
- **Shown failing** against a scratch quadratic edit, e.g. cloning the toast
  queue once per toast inside `render_toasts`. Report the ratio in both states.
- Report the absolute numbers with machine and profile, for the envelope doc.
- **If the ratio is not stable enough on CI to be trusted, say so** and propose
  the count-based companion (Q-2 (b)) rather than loosening the bound until it
  passes.

## CHANGELOG

- **Fixed:** long unbroken text (file names, paths, URLs, hashes) was silently
  cut off in toasts, and in any other surface Unit 1 confirms. It now wraps.
  Surfaces with such text can grow taller. Name REQ-003 (tekstide), and note that
  it is a defect for current consumers too.
- **Added (tests):** verbatim and bounded-work assertions; REQ-003 / REQ-006.

## Acceptance criteria

1. Each candidate surface measured; `WordOrGlyph` exactly where clipping was
   shown; tooltips handled as Unit 1 says.
2. The unbroken-fixture test is **shown failing** on `Word`, and normal prose is
   unchanged.
3. The verbatim fixture is found by exact content, and **shown failing** under a
   scratch transform.
4. The bounded-work test and CI step exist, and the test is **shown failing**
   under a scratch quadratic edit.
5. Numbers reported; nothing under `docs/`.
