# Handoff — 0.63 audit: is every visual guarantee in the book guarded by a test?

**Ruled.** The owner, 2026-11-14 ("Both your recommendations are accepted"), on the architect's
proposal while scoping 0.63.
**What it produces.** A table and findings, in two review requests. **No code and no book change in
this work.** The architect corrects the book. Tests for unguarded claims are a follow-up the
review rules on, and what it rules becomes 0.63.
**Method.** 0.62's audit (`rfcs/handoffs/audit-0-62-keyboard-and-overlay-claims/implementation-handoff.md`).
Read its verdict table and "What counts as a claim"; they apply unchanged. This handoff says only
what differs.
**Where.** On `main` (the owner's one-`main` ruling, 2026-11-14). A probe that has to run code goes
in a scratch test that is **described in the request and not committed**. A remote branch only if
the probe needs the CI runner. Say so, and give the owner the delete command.

## Why

**The visual guarantees are what adopters cite:**
- orbok's WCAG record cites snora for text contrast (1.4.3), non-text contrast (1.4.11), visible
  focus (2.4.7) and target size (2.5.8);
- arama cites the AA contrast;
- knotra holds the typography covenant (RFC-072) as a binding rule.

**No audit has read these pages.** 0.62's covered keyboard and overlays: 8 pages, 508 claims, 15
false, two of them defects. **And 0.62 found a visual claim broader than its evidence by
accident:** "text at AA" held for the declared pairs at rest, and not for a notice's action label
or for hovered and pressed states (RFC-128, RFC-129). This audit looks for the next one
deliberately.

## Scope

**Pages (1,404 lines, plus the visual half of `buttons.md`):**

| Request | Page | Lines |
|---|---|---|
| A | `docs/src/design/theme.md` | 264 |
| A | `docs/src/design/engine-surfaces.md` | 283 |
| A | `docs/src/design/high-contrast.md` | 63 |
| A | `docs/src/design/aaa-presets.md` | 81 |
| A | `docs/src/design/tokens.md` | 78 |
| A | `docs/src/design/chips.md` | 55 |
| A | `docs/src/design/buttons.md`, **except** its keyboard notes (0.62 audited them) | ~100 |
| B | `docs/src/design/typography.md` | 125 |
| B | `docs/src/guides/readability.md` | 116 |
| B | `docs/src/guides/direction.md` | 106 |
| B | `docs/src/design/chrome-geometry.md` | 128 |
| B | `docs/src/design/stability.md` | 105 |

**Rustdoc (request A):** every sentence of `snora-design` and `snora-style` that states a contrast,
a size, a target, a ring, or what a style function draws:
- `contrast.rs`, `palette.rs`, `focus.rs`, `targets.rs`, `tokens.rs` and the presets;
- `snora-style`'s `button`, `chip`, `container`, `text`, `progress` and `theme`.

**Not in scope:** as in 0.62 (migration guides, released changelog entries, history and
*Corrected* lines, rationale, future wanted, RFCs). Also out:
- **0.62's backlog**, the guides' rows guarded in part (owner, 2026-11-14). Do not re-audit them.
- **Keyboard claims** on these pages. 0.62 audited the keyboard map; cite its row instead.

## What differs from 0.62

**1. A claim about what is drawn is guarded only by a test that reads what is drawn.** RFC-128's
lesson: the palette's suite passed, and the label on screen was not a palette pair.
- A test of a token or a style function guards a claim about the token or the function.
- "The ring clears 3:1", "the label reads 7:1" and "the border is visible" are claims about
  pixels. Guarded by a token test alone, they are **Guarded in part**. Say which half is missing.

**2. A guard must not read what it guards.** RFC-129 Q-4's lesson: a suite that takes its floor
from the token it checks lowers its bar when the token is flipped. Where a test takes a threshold
or an expected value from the code under test, the verdict is **Guarded in part**. Name the value
it reads.

**3. Name the renderer.** A pixel claim measured on tiny-skia only is guarded on tiny-skia. Say
whether wgpu (the default renderer) is covered, through the parity job or a rendered test.
**Do not compare across renderers as before and after:** a change in a claim is shown on one
renderer, before against after.

**4. Direction claims (ABDD) need both directions.**
- A layout claim checked in left-to-right only is **Guarded in part**.
- A claim that something does **not** mirror (logical order, F6's order, an icon) needs a test
  that would fail if it did.

**5. The stability page is a covenant about the API, not about pixels.**
- Its claims ("additive-only", "frozen", "`#[non_exhaustive]`") are guarded by a source or
  compile-time check, or they are not.
- Say which: a `#[non_exhaustive]` attribute can be read from the source, while "additive-only
  since 0.x" is guarded only if something compares surfaces across releases.
- If nothing does, the verdict is **Unguarded, true** or **Not testable here**, with the
  reason. Do not propose a cross-release tool in this work; name the gap.

**6. Inventory every non-text element snora draws, as you go.** List every glyph, border,
indicator, divider, track and mark, with where it is drawn and whether any test measures its
contrast. Put the list in request A's evidence as `non-text-inventory.md`. It is the input to the
non-text contrast work (`rfcs/handoffs/non-text-contrast-0-63/`), and that unit starts from it.

## Two review requests, in this order

1. **`.git-exclude/review-request/audit-0-63-a/`:** the contrast and surface pages, the rustdoc,
   and the non-text inventory.
2. **`.git-exclude/review-request/audit-0-63-b/`:** typography, readability, direction,
   chrome geometry, and stability.

Each has a `claims.md` with 0.62's columns and verdict words. The README carries:
- the counts;
- **every false claim first;**
- the proposed tests, grouped by the suite they would join.

**Stop at a false claim and report it in the first lines,** as in 0.62. The architect decides
whether it is a defect to fix or a sentence to correct.

## Acceptance criteria

1. Every normative sentence of the scoped pages and rustdoc is in a `claims.md` row, or listed as
   not a claim with a one-word reason.
2. Every "Guarded" names its test. One in five is shown failing by a breakage. Use named `cp`
   backups and `sha256sum -c`; never `git checkout` or `git restore`.
3. Every "Unguarded" has a probe result. Every pixel claim names its renderer.
4. Request A carries `non-text-inventory.md`.
5. No product code or book page changed, and nothing committed.
