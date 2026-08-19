# RFC 071 — The visibility floor tests a bar the palette cleared four minors ago, and the page still publishes the old numbers

**Status.** Done — shipped in v0.38.1 (2026-08-20).
[Handoff](../handoffs/071-the-visibility-floor-tests-a-bar-cleared-four-minors-ago/implementation-handoff.md).
**Tracks.** Accessibility assertions / documentation integrity.
**Found by** the architect, while verifying a **knotra** report that turned out
to be incorrect. The report was wrong; checking it found this.
**Touches.** `crates/snora/src/design/render/tests.rs`,
`docs/src/design/engine-surfaces.md`.
**Release target.** 0.38.1 if documentation and test-only; 0.39.0 if a floor
change is judged to alter a published guarantee.

## Summary

Two defects, one cause.

**1. `engine-surfaces.md` publishes pre-repair figures as current.** Its
"Measured border-vs-background contrast, all four presets" table says:

| Preset | published | **actual, current source** |
|---|---|---|
| `light` | 1.39:1 | **3.38:1** |
| `dark` | 1.43:1 | **3.81:1** |
| `high_contrast_light` | 21.0:1 | 21.00:1 ✓ |
| `high_contrast_dark` | 21.0:1 | 21.00:1 ✓ |

1.39 and 1.43 are the **before** values from the 0.34.0 border repair — they
appear in that release's own before/after table. They have been wrong on this
page since **0.34.0, four minors ago**.

**2. The floor those numbers justify now asserts almost nothing.**
`VISIBILITY_FLOOR = 1.3` (`design/render/tests.rs:50`) is justified, in its own
doc comment and on the page, as *"what the `border` role, used directly and
unmodified, actually achieves across all four built-in presets, with a small
margin under the worst case (`light` at 1.39)"*, and as *"deliberately modest,
well below WCAG SC 1.4.11's full `3.0` … matching a real, unmodified token
value rather than inventing a derivation to clear a stricter number."*

**Every clause of that is now false.** The worst case is 3.38, not 1.39. The
margin is 2.6×, not small. And the real, unmodified token value clears 3.0 in
all four presets — so the stated reason for not using 3.0 has evaporated.

**The consequence:** `light`'s border could regress from 3.38 to 1.31 and the
assertion still passes. **The 0.34.0 accessibility repair is unprotected by the
very test that measures it.** That repair was RFC-036's first use of the
accessibility carve-out — a palette value changed because a test proved a
defect — and nothing now stops it silently unwinding.

## Correction (2026-08-19) — **both** tables on that page are stale

This RFC's first draft said the dim's worst case was 2.85 and therefore blocked
raising the shared floor to 3.0. **That was wrong, and wrong in exactly the way
this RFC exists to describe: I read the figures off the page instead of
re-deriving them.**

Re-derived from source — `modal_dim()` composites the opposite pole at
`DIM_ALPHA = 0.44` over `background`:

| Preset | published | **actual** |
|---|---|---|
| `light` | 2.85:1 | **3.2424:1** |
| `dark` | 3.83:1 | **4.3798:1** |
| `high_contrast_light` | 2.85:1 | **3.2424:1** |
| `high_contrast_dark` | 3.66:1 | **4.2529:1** |

The published figures are the **pre-RFC-065** values, stale since **0.37.0**.
`DIM_ALPHA`'s own doc comment carries the correct one — *"chosen to clear it
with real margin (8%, 3.24:1)"* — so the constant and the page have disagreed
since the day RFC-065 shipped.

**So the page's border table is stale from 0.34.0 and its dim table is stale
from 0.37.0.** Neither repair updated the prose that describes it. That is the
defect, twice, on one page.

**And the blocker disappears.** The dim's true worst case is **3.2424** and the
border's is **3.38** — both clear 3.0 with margin. `VISIBILITY_FLOOR` can be
raised to `NON_TEXT_MIN` **without splitting it**, and the shared-constant
argument survives intact.

## Open questions

**Q-1 — split the constant, or keep one floor? → Keep it shared.**
Superseded by the correction above: splitting was proposed only because the dim
appeared unable to reach 3.0, and it can. One constant at `NON_TEXT_MIN`
satisfies both assertions with margin, and the conceptual argument for sharing —
both measure "is this element visually distinct from the page behind it" —
needs no compromise.

**Q-2 — what floor for the border?** `3.0` (`NON_TEXT_MIN`, already a constant
since RFC-058) is achievable by all four presets today with margin. Note what
asserting it commits us to: under RFC-036 the border values become effectively
frozen at ≥3.0, changeable only through the accessibility carve-out.
**Suggest 3.0 anyway** — the 0.34.0 repair was made specifically to clear 3.0
on each preset's binding pair, so asserting it records an intent we already
acted on rather than adding a new one.

**Q-3 — is the dim's table current? → No. Answered by re-deriving it, above.**
It was stale from 0.37.0. The instruction that produced the answer stands as the
rule: **re-derive; never assume newer means current.** I assumed it once in this
RFC's own first draft and was wrong within the hour.

**Q-4 — is this the systemic instance?** This is the third figure-in-prose that
drifted from its source: RFC-062's feature-gating status table (stale ten
minors), RFC-063's hand-maintained contrast pair list, and now this. Every one
was a number a human wrote next to code that could have produced it. Whether
that warrants a general mechanism — a test that emits the figures a page
claims — is worth asking, but **not in this RFC.** Fix these two first.

## Acceptance criteria

1. **Both** tables on `engine-surfaces.md` — border and modal dim — carry
   current measured figures, re-derived from source rather than copied from
   anywhere, **including this RFC.**
2. The prose justifying the floor no longer cites 1.39 as the worst case, and
   no longer says the real token value sits below 3.0.
3. Q-1 and Q-2 answered; whatever floor the border gets is derived from
   measurement and its doc comment states the measured worst case **and the
   release it was measured at**, so the next drift is visible.
4. The dim's figures re-derived (Q-3), and its floor stated on the same terms.
5. **A perturbation demo:** move `light`'s border toward `background` until it
   crosses the new floor, capture the failure, restore. A floor that has never
   fired is not known to fire.
6. No palette value changes. This RFC repairs an assertion and a page, not a
   colour.

## Compatibility and security

**Compatibility.** No public API, no palette value, no rendered output. Raising
an internal test floor changes what CI catches, not what ships.

**Security.** None.
