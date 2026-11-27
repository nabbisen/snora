# RFC-139 — Turning on `lucide-icons` breaks the marks snora draws itself

**Status.** Accepted 2026-11-27 (owner), every open question ruled as suggested (see Rulings).
**Raised.** 2026-11-26, the 0.67 audit's request A (row A2-082, probed in a real window by the dev team), reviewed by
the architect (`.git-exclude/reviewed/audit-0-67-a/review-result.md`). The owner asked for this RFC for 0.66 on
2026-11-27.
**Release target.** 0.66.0, as a fix.
**Priority.** A defect in shipped behaviour. It reaches applications that never asked for the feature.

---

## The finding

**With `lucide-icons` on, five marks that snora draws itself take their glyph from the Lucide font:**

| Mark | Where | Without the font |
|---|---|---|
| The toast's close mark | `crates/snora/src/toast.rs` (RFC-101 Q-1 (a)) | **a missing-glyph box (▯)**, measured in a real window (`s5-nofont.png`) |
| The design-path notice's dismiss mark | `snora-widgets/src/design/chip.rs`, `close_glyph`, used by `notice.rs` | a missing-glyph box (measured in Unit 0) |
| The design-path chip's remove mark | the same `close_glyph` | a missing-glyph box (measured) |
| The checkbox's tick | `snora-widgets/src/form.rs`, `Mark::Tick` | **a "1" in the checked box** (measured): the private-use codepoint falls through to another font's digit |
| The select's chevron | `form.rs`, `Mark::Chevron` | **a "3"** (measured) |

*(Measured in Unit 0, 2026-11-27, on both renderers; this table first said "by the code, the same" for the last four.)*
**The layout moved with the state too:** the notice's dismiss control was 28.0 px wide with the feature off, 28.4 with it
on and no font, and 34.0 with the font loaded. A select differed by up to 8 px.

- **snora never loads the font.** Only the application can, with `iced::application(..).font(LUCIDE_FONT_BYTES)`, and
  **no adopter-facing page or rustdoc says that it must.** The only mention is the 0.40 migration guide's remark that
  the byte slice exists.
- **It reaches applications that never asked.** Cargo features unify across the dependency graph. One crate that
  enables `snora/lucide-icons` for its own icons turns it on for the whole application. Every toast's close mark then
  becomes a box, and every checked checkbox loses its tick, in an application whose authors never chose Lucide.
- **Why no test saw it.** `crates/snora/tests/common/mod.rs`'s `simulator` loads the Lucide font whenever the feature is
  on. Every test built on it therefore ran in the state an adopter reaches only *after* finding the byte slice, and
  **no test asserted what a mark is in the state an adopter starts in**: the feature on, and the font not loaded.
  *(Corrected after Unit 0: this said no test ran in that state at all.)*
- **And the tests that did sit in that state measured the wrong glyph and passed** (found in Unit 0). The drawn sweeps
  use a rig that never loads the font, so in the all-features job they read the boxes, the "1" and the "3" as the
  marks, and passed, because a box has ink and contrast. **A contrast sweep proves that a mark is visible, not that it
  is the right one.** R-3's test compares each mark with the intended glyph.

## Proposal

**R-1. A Cargo feature never changes how a mark snora draws itself is drawn.** The five marks above draw, with
`lucide-icons` on, exactly what they draw with it off: "×", "✓" and "▾" in the text's own font. These are the glyphs
the `no-lucide` CI job already draws, and its sweeps already measure them for contrast and target size.
- **`lucide-icons` becomes what its name says:** it adds the Lucide set for the *application's* icons
  (`Icon::Lucide`, `snora::lucide`), and nothing else.
- This reverses RFC-101 Q-1 (a) and RFC-100's consistency argument for the marks: a close mark that matches the
  application's Lucide icons is not worth a control that can silently break.
- **A visible change for applications that load the font today:** their five marks change from Lucide's shapes to the
  text glyphs. The migration guide says so, with before and after.

**R-2. An application that uses Lucide icons is told to load the font, where it meets them:**
- the `snora::lucide` module's rustdoc, and `Icon::Lucide`'s;
- `guides/icons.md`'s Lucide section, with a compiled snippet that loads it;
- `01-install.md`'s feature table.
Each says what happens without it: the icon draws as a stray fragment or a box.

**R-3. A test in the adopter's starting state.** It runs in the all-features suite, with the feature on and **no Lucide
font loaded**. It renders each of the five marks and asserts that it draws the intended glyph in the text's font, not a
missing-glyph box or a fragment. It must be shown to fail on today's code.

## Open questions, with suggested rulings

**Q-1. Should an application that loads the font be able to keep Lucide's marks?** **Suggest: no.**
- A Cargo feature cannot be the switch, because features unify. That is this defect again.
- A run-time setting would be safe, but it would be a new public API with a cosmetic gain that no adopter has asked
  for. Each of its two states would also need every sweep.
- If an adopter asks, it is a separate RFC, and it would be a run-time setting, never a feature.

**Q-2. Can snora detect that the font is missing and say so?** **Suggest: no, and the RFC says why.** A widget cannot
see which fonts the application loaded, and iced 0.14 has no query for it. R-2's sentences and R-3's test are the
route.

**Q-3. The text glyphs depend on the system's fonts too.** "×" (U+00D7) is in essentially every Latin font. "✓"
(U+2713) and "▾" (U+25BE) are in common desktop fonts, and cosmic-text falls back across every installed font, but
neither is universal. **Suggest:** Unit 0 measures which fonts cover them on the platforms snora names and on CI's
runner. If a common desktop setup lacks them, drawing the marks as shapes is a separate RFC, measured first, and is not
part of this 0.66 fix. Nobody has reported a box on the feature-off path.

**Q-4. When.** **Suggest: 0.66.0, as a fix,** after RFC-138 Unit 1 or beside it.

## Rulings, 2026-11-27 (owner)

"Authorized." Every question as suggested:
- **Q-1:** no way to keep Lucide's shapes for snora's marks. If an adopter asks, it is a separate RFC, and a run-time
  setting, never a feature.
- **Q-2:** snora does not try to detect a missing font. R-2's sentences and R-3's test are the route.
- **Q-3:** Unit 0 measures the system fonts' coverage of "✓" and "▾". Drawing the marks as shapes, if needed, is a
  separate RFC.
- **Q-4:** 0.66.0, as a fix.

**The principle this sets, for every RFC after it:** a Cargo feature may only make an API available. It never changes
how something snora already draws looks or behaves, because a feature can be turned on by another crate in the graph.

## Review against snora's principles

- **Safe.** A feature turned on by someone else can no longer break a control the user needs, such as a toast's close
  mark or a checkbox's state.
- **Nobody is confused.** `lucide-icons` now means what it says. The one thing an application must do for its own
  Lucide icons is written where it meets them.
- **Clean.** Five `#[cfg]` branches go, and one path is left for each mark. The `no-lucide` job's sweeps already cover
  that path.
- **Robust.** The adopter's starting state gets a test, which closes the gap that hid this.
- **ABDD and keyboard.** Unchanged.
- **Correspondence cost.** One line in 0.66's news, which goes out anyway. A letter of its own is not needed: the fix
  ships in the same release as the news.

## Units

**Unit 0: measured, no product code.**
- The five marks with the feature on and the font not loaded, on both renderers: what each draws, from pixels.
- The same with the font loaded, and with the feature off. Record each mark's size and position in the three states.
- Q-3's font coverage for "×", "✓" and "▾".
- A prototype of R-3's test, shown to fail on today's code.

**Unit 1: the fix.**
- R-1 in the three source files. `engine_and_widgets_draw_the_same_lucide_glyph` and the comments that pair the two
  sites go, and the test-harness comment about loading the font is corrected.
- R-3's test, with breakage rows: restore any one of the five Lucide branches, and the test fails.
- The rendered suites and sweeps pass under `--all-features` and under the `no-lucide` job.
- After R-1 the two combinations draw the same marks. Whether the `no-lucide` jobs stay (they still build without the
  `lucide-icons` dependency) or shrink is ruled at Unit 1's review, with their run times.
- CHANGELOG `[Unreleased]`: **Fixed** (the marks under `lucide-icons` without the font), and **Changed** (the marks'
  shape for applications that load it).

## Acceptance criteria

1. Unit 0's measurements, and R-3's test shown failing on today's code.
2. Unit 1: R-1 and R-3, each guard shown able to fail, and every `ci.yaml` command run locally.
3. **The book, by the architect:** R-2's pages and snippet, and the 0.65 → 0.66 migration guide's section.
