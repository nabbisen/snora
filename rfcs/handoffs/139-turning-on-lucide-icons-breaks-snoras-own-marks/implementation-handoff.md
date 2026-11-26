# Implementation handoff — RFC-139: snora's own marks never take the Lucide font

**RFC.** `rfcs/accepted/139-turning-on-lucide-icons-breaks-snoras-own-marks.md`. Read the finding, R-1 to R-3 and the
Rulings (every question as suggested, and the principle they set).
**Release target.** 0.66.0, as a fix.
**Starts** now (owner, 2026-11-27). **Order:** RFC-138's Unit 1 first if it is under way, then this. Both come before
the 0.67 audit's request B.
**Where.** On `main`, by the one-`main` route, **after the review approves it**. Probes are scratch, described and not
committed.
**Order of units.** Send Unit 0 for review first. Unit 1 starts after that review.
**The defect's evidence:** `.git-exclude/review-request/audit-0-67-a/evidence/s5-nofont.png`, `s5-font.png` and
`walk-step5-lucide-main.rs`.

## Where the change goes

The five marks come from **three sites**. Each has a `#[cfg(feature = "lucide-icons")]` branch that goes:

| Site | Marks | Today under the feature |
|---|---|---|
| `crates/snora/src/toast.rs`, the close-glyph block (about lines 300 to 365) | the toast's close mark | `char::from(lucide_icons::Icon::X)` in `Font::with_name("lucide")`, on both the design (`text`) and the engine (`Glyph`) branch |
| `crates/snora-widgets/src/design/chip.rs`, `close_glyph` | the chip's remove mark, and the notice's dismiss mark (`notice.rs` calls it) | `icon_element_sized(&Icon::Lucide(Icon::X), ..)` |
| `crates/snora-widgets/src/form.rs`, `Mark::icon` | the checkbox's tick, the select's chevron | `Icon::Lucide(Check)` and `Icon::Lucide(ChevronDown)` |

- **After the change each site has one path:** the one the feature-off build has today ("×", "✓", "▾" in the text's
  font). Keep its sizing exactly (RFC-115's 1.125 em for "×", the form marks' em), so the feature-off build draws the
  same pixels before and after. **That is a check, not a hope:** compare the feature-off frames before and after.
- **The long comments that pair the toast's site with the widgets' site** (RFC-101 Q-1 (a), the private-use-area
  remark) go. Leave one sentence per site citing RFC-139: a feature never changes how a snora mark is drawn.
- **`snora_widgets::icon`'s handling of `Icon::Lucide` is unchanged.** Applications' own Lucide icons still draw in
  the Lucide font. That is what the feature is for.

## The tests

The files that mention Lucide: `crates/snora/tests/common/mod.rs`, `common/sweep.rs`, `dismiss_remove_controls.rs`,
`every_control.rs` (line ~1617 loads the font), `nontext_sweep.rs`, `render_semantics.rs` and `toast_close_button.rs`.
For each one, decide and report:
- **It measured a snora mark as a Lucide glyph** (`LUCIDE_X`, the feature-conditional click target RFC-101 Q-3 gave
  `render_semantics.rs`, `engine_and_widgets_draw_the_same_lucide_glyph`). After R-1 it measures the text glyph in both
  builds. A test that only pinned the two sites to one codepoint goes, and you name it in the request.
- **It loads the font for an application icon** (`Icon::Lucide` drawn by `icon_element`). Keep it.
- **`common/mod.rs`'s `simulator` comment** ("Every simulator in a test that measures a lucide glyph must come from
  here") is corrected to say what still needs the font: an application's Lucide icon, never a snora mark.

**R-3's test, new:** in the all-features suite, a simulator **with no Lucide font loaded** (`iced::Settings::default()`,
not `common::simulator`). It renders each of the five marks, and compares each mark's ink with the same glyph drawn by
`text` in the text's font at the same size. Put the toast's mark and the four widget marks in one file of their own.
- **It must fail on today's code**, where each mark is a Lucide codepoint with no font (a box or a fragment). Show
  that in Unit 0, against `origin/main`, before any product change.
- **Do not compare against a frame recorded from the feature-off build.** One test binary cannot see both builds, and
  a recorded frame cannot fail for the right reason.

## Unit 0 — measured (no product code committed)

- **The three states, on both renderers:** the feature on with no font; the feature on with the font; the feature off.
  For each of the five marks, record what is drawn (ink bounding box, and a crop at 2×) and the control's target size.
- **R-3's prototype test, failing on `origin/main`,** with its output.
- **Q-3, font coverage:** which installed fonts cover "×" (U+00D7), "✓" (U+2713) and "▾" (U+25BE) on this machine and
  on CI's runner (`fc-list ':charset=2713'` and the like), and which font cosmic-text actually picks for each, if you
  can read it. Report it; do not act on it.

## Unit 1 (0.66.0)

- R-1 at the three sites. R-3's test. The test files above, adjusted as you reported.
- **Breakage rows,** each shown failing:
  - restore the toast's Lucide branch (R-3's test fails on the toast's mark);
  - restore `close_glyph`'s (it fails on the chip and the notice);
  - restore `Mark::icon`'s (it fails on the tick and the chevron);
  - load the font in R-3's simulator (the test must still pass after R-1, and fail on today's code: show both).
- **The feature-off frames are unchanged,** shown by comparing the five marks before and after at 2×.
- **The `no-lucide` jobs:** report their run times, and whether they still cover anything the all-features jobs do not.
  I rule on them in the review.
- **CHANGELOG `[Unreleased]`:** **Fixed** (snora's marks under `lucide-icons` without the font); **Changed** (the five
  marks' shape for applications that load the font).

## Acceptance criteria

1. Each test is shown able to fail. Use named `cp` backups and `sha256sum -c`; never `git checkout` or `git restore`.
   Run the breakage script again on the final tree.
2. The feature-off build draws the five marks pixel for pixel as before.
3. Every `ci.yaml` command, run locally, before pushing to `main`, and only once the review has approved it.
4. **The book is the architect's** (R-2): `icons.md` with a compiled snippet that loads the font, `01-install.md`'s
   feature table, the `snora::lucide` rustdoc and `Icon::Lucide`'s (the rustdoc lines are mine too, so leave them), and
   the 0.65 → 0.66 migration guide.
