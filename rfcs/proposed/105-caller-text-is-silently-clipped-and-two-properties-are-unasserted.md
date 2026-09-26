# RFC-105 — Caller text is silently clipped, and two properties are unasserted

**Status.** Proposed.
**Raised.** 2026-09-26, architect. Sources: REQ-003 and REQ-006. The clipping
was found by measurement while assessing them.
**Release target.** 0.52.0. **Priority.** Third: small, and it fixes a real
defect.

---

## The finding

**An unbroken string is silently cut off.** A 400-character unbroken string in
a toast renders as **one line clipped at the 288 px message column**. About 48
characters show, nothing marks the cut, and the close button is not overlapped.
iced's default `Wrapping::Word` cannot break inside a word. File names, paths,
URLs and hashes are the usual cases, so this affects current consumers (arama is
an image browser).

**The obvious detector does not see it.** `visible_bounds()` and `bounds()` are
both 288.0 × 18.2, because the glyphs overflow inside the text widget's own
box. **Natural size against rendered size does detect it**; that is the
technique the RFC-099/100 tests already use.

**Two properties are true but unasserted:**

- **Verbatim:** snora transforms no caller string (checked by inspection), and
  nothing would fail if a component began to.
- **Bounded work:** linear, measured at layout **0.6 / 5.9 / 62.9 / 656 ms** for
  **100 / 1k / 10k / 100k** toasts (release, tiny-skia, reference machine).
  Nothing would fail on a superlinear regression.

## Proposal

**R-1 — caller text is shown whole.** `Wrapping::WordOrGlyph`, which exists in
iced 0.14, on caller text in **fixed-width, multi-line surfaces**, so text
breaks inside a word only when it must. Scope: Q-1.

**R-2 — assert it.** For each surface in scope: render the unbroken fixture and
assert that its rendered box holds its **natural** size (no clipped overflow).
Show it failing against today's `Word`.

**R-3 — assert verbatim.** tekstide's criterion is *"a bidi override, a newline
and a control character, passed in and read back from what the component
draws"*. Render that fixture in each caller-text surface and find it by
**exact** content, which proves snora passes the string through unmodified.
Combine that with R-2's natural-size check.

**What this cannot assert, stated rather than implied:** the drawn *glyphs*.
The harness exposes frame hashes, not pixel reads, in its public API. How iced
and cosmic-text render a bidi override is iced's behaviour, not snora's.

**R-4 — assert bounded work, machine-independently.** A ratio assertion (10×
input costs well under 100× layout time) at sizes large enough to be stable. The
absolute number goes into `performance-envelope.md` as a recorded measurement
with its machine, not as a threshold. Mechanism: Q-2.

## Open questions

**Q-1 — which surfaces get `WordOrGlyph`?** Suggest the **fixed-width,
multi-line** ones: toast title and message, notice title and body, and the
sidebar and close-control tooltips. **Not** single-line chrome labels (tab,
crumb, menu, header): they shrink to their text, so they do not clip at a
width. Wrapping them would change their height instead. The handoff confirms each
by the R-2 measurement rather than by this list.

**Q-2 — how is bounded work asserted without a flaky timing test?**

| Option | Shape | Risk |
|---|---|---|
| **(a) Ratio of layout times** | e.g. `t(10k) / t(1k) < 30`, run in the release profile or `#[ignore]`d and run by a CI step | Timing noise, which the ratio absorbs; debug-build overhead |
| (b) Count-based | Assert that layout visits O(n) nodes (widget-count proxy) | Proves structure, not time |
| (c) Measurement only | Recorded in the envelope doc, no assertion | tekstide: *"unasserted is unverified"* |

**Suggest (a)**, as an `#[ignore]`d test run by one CI step in release, with (b)
as a cheap companion if the handoff finds a stable proxy.

## Acceptance criteria

1. `WordOrGlyph` on the surfaces ruled in Q-1. The unbroken-fixture test is
   **shown failing** on `Word` and passes after.
2. The verbatim fixture (bidi override, newline, control character) is found by
   exact content in every caller-text surface.
3. The bounded-work assertion as ruled in Q-2 runs in CI, is **shown failing**
   against a scratch superlinear edit (e.g. a quadratic clone in toast
   rendering), and the measured number is recorded in `performance-envelope.md`
   with its machine and build profile.
4. The migration guide states the wrapping change: long unbroken text now wraps
   instead of being cut off, and the heights of affected surfaces can grow.
