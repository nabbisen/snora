# RFC 077 — The border is not what outlines the dialog card, and our own rationale says it is

**Status.** Done — shipped in v0.39.0 (2026-08-20).
[Handoff](../handoffs/077-the-border-is-not-what-outlines-the-card/implementation-handoff.md).
**Tracks.** Accessibility rationale / design record.
**Found by** **arama**, 2026-08-20, measuring a 27-photo gallery at 0.38.0.
Arithmetic re-derived by the architect before acceptance.
**Touches.** `docs/src/design/engine-surfaces.md`, `rfcs/done/058`'s rationale as
restated in `CHANGELOG.md` and `docs/src/contributing/accessibility-checklist.md`.
**Release target.** 0.39.0 — documentation and rationale only. **No values.**
**Figures still current.** arama measured at 0.38.0. `git diff 0.38.0 0.38.3` on
`presets/` and `surfaces.rs` is **empty** — no palette value and no `DIM_ALPHA`
changed in the three intervening releases, so their numbers hold as measured and
do not need re-taking.

## Summary

arama measured the dialog card over real photographic content:

| preset | border ǀ card fill | **border ǀ dim** | dim ǀ card fill |
|---|---|---|---|
| `light` | 3.38:1 | **1.02:1** | **3.46:1** |
| `dark` | 3.17:1 | **1.23:1** | **3.89:1** |

**The 1px border is invisible against the dim.** What separates the card from
the dimmed page is the **dim-to-fill step**, not the border.

**Re-derived independently before accepting it.** In `light`, the dim over light
content composites to `0.56 × content`; against near-white content that is a
grey of roughly `0.56`, and `border` is `0.537`. Computing from pure-white
content: **border ǀ dim = 1.04:1**, dim ǀ fill = 3.24:1 — against their 1.02 and
3.46 from real sampled pixels. Their method is sound and their conclusion holds:
**our border colour and the dim over light content land at nearly the same
luminance, by arithmetic, not by accident of one photo.**

Their `border ǀ card fill` column matches our published figures exactly once you
know "surface" means the card's own fill (`surface_raised`, RFC-039) — 3.3808
and 3.1653. That is their stated method check, and it passes.

## Why this matters beyond a curiosity

**RFC-058 repaired the border citing SC 1.4.11 and "a border is that boundary
for the RFC-039 dialog card, which was deliberately chosen as border-defined
rather than shadow-defined."** Against the dim, it is not that boundary. The
repair is still correct and still needed — it does its work at the card's
**inner** edge, border against fill, at 3.38 and 3.17 — but *where* it does that
work is not where our own rationale says.

**Our test was right for a reason we did not state.** RFC-066's sweep asserts
`max(border ǀ dim, fill ǀ dim)` — identifiable if *either* clears the bar. arama's
data shows it is **always the fill route that carries it** in practice and the
border route never does. The assertion is correct; the prose around it implies a
belt-and-braces where there is one working mechanism and one that does not
apply.

## Scope

State, where the border repair and the dim are explained:

1. The border's contrast job is against the **card's own fill**, at the inner
   edge — that is what 3.38/3.17 measure and it is a real requirement.
2. The card's separation from the dimmed page is carried by the **dim-to-fill
   step**, and in `light` the border is very nearly invisible against the dim by
   arithmetic.
3. RFC-066's `max(…)` assertion is therefore load-bearing on one branch, and
   the prose should not imply both are available.

## Non-goals

- **No value changes.** Not the border, not `DIM_ALPHA`. Every measured pair
  clears its threshold; nothing is failing.
- **No new assertion.** RFC-066's sweep already tests the correct quantity.
- **Do not weaken the border requirement.** It is doing necessary work at the
  inner edge; that it does not also outline the card is not an argument for
  relaxing it.

## Open questions

**Q-1 — answered, and it refuted the guess below.** Swept over greyscale content
at `DIM_ALPHA = 0.44`, **every preset has a content luminance at which
`border ǀ dim` reaches 1.00:1** — near-white in `light`, black in `dark` and
`high_contrast_light`, white in `high_contrast_dark`. There is no preset in
which the border reliably outlines the card, and `dim ǀ fill` never drops below
**3.16** in any of them. The original speculation, kept below so the correction
is visible:

**~~Q-1 — does this reach `high_contrast_*`?~~** arama measured `light` and `dark`
only. In the high-contrast presets the border is pure black or white and the dim
composites from the opposite pole, so the border is very likely **highly**
visible against the dim there — the opposite of `light`. **Derive all four
before writing anything**; a claim stated for two presets and quietly implied
for four is the defect this project keeps finding.

**Q-2 — does the same reasoning apply to the sheet panel?** It has a border and
sits over the same dim. Unmeasured. **Suggest measuring, and saying so either
way** rather than generalising from the dialog.

## Acceptance criteria

1. All four presets derived, not two (Q-1), with the method stated.
2. `engine-surfaces.md` states which mechanism separates the card from the dim,
   and which one does not.
3. The RFC-058 rationale is corrected where restated, without weakening the
   border requirement.
4. No palette value, no `DIM_ALPHA`, no assertion changed.
5. arama credited.

## Compatibility and security

**Compatibility.** Documentation only. **Security.** None.
