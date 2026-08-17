# RFC 058 — `border` contrast is untested, and `light`/`dark` ship it at ~1.3:1

**Status.** Proposed
**Tracks.** Accessibility. Reported by **tekstide** (2026-08-17), verified
independently.
**Touches.** `crates/snora-design/src/tests.rs`,
`crates/snora-design/src/presets/{light,dark}.rs`,
`docs/src/contributing/accessibility-checklist.md`, `CHANGELOG.md`.
**Release target.** 0.34.0 (minor — preset values change, so rendered
appearance changes on the `design` path).

## Summary

snora's contrast suite asserts **twelve** pairs across all four presets. Every
one is text-on-surface, plus `focus/background` and `focus/surface`.

**`border` and `text_muted` appear in none of them** — zero references in
`tests.rs`. And the untested role is failing: `border` measures **1.19–1.43:1**
against surfaces in `light` and `dark`, against WCAG 2.1 SC 1.4.11's 3:1 for
non-text visual boundaries.

## The evidence

Computed from `presets/*.rs` with the WCAG 2.1 relative-luminance formula,
independently of the report, and matching it to two decimals on all eight
figures:

| preset | border/background | border/surface | border/surface_raised |
|---|---|---|---|
| `light` | **1.39:1** | **1.28:1** | **1.39:1** |
| `dark` | **1.43:1** | **1.32:1** | **1.19:1** |
| `high_contrast_light` | 21.00:1 | 21.00:1 | 21.00:1 |
| `high_contrast_dark` | 21.00:1 | 21.00:1 | 19.80:1 |

`text_muted` already passes — **4.83:1** light, **5.44:1** dark — so asserting
it costs nothing and locks in what we have.

The high-contrast presets are correct by construction, exactly as the
checklist requires. The defect is confined to `light` and `dark`.

## Is it a failure, or only an untested role? Both, and the distinction matters

tekstide drew a line we should keep:

> SC 1.4.11 applies to boundaries *required to identify the component*. A
> border that is purely decorative — where the control is identifiable by its
> fill, label, or spacing — is exempt. **What is unconditional is the test
> gap.**

They could not resolve the first half; we can. **For at least one surface the
border is the identifying boundary**: RFC-039 chose the dialog card as
**border-defined rather than shadow-defined**, deliberately, because shadows
carry almost no information in high-contrast presets. A 1.39:1 border on
`light` is therefore the sole visual boundary of a modal card.

`card_surface`, `card_raised` and `card_selected` all set a border too.

So this is not merely a missing assertion. **It is a shipped 1.4.11 concern on
the `design` path**, and the missing assertion is why nothing said so.

## The root cause is a rule stated once, narrowly

`accessibility-checklist.md`'s **Contrast** section mandates only `>= 4.5:1 for
body text`. The 3:1 non-text rule appears **only** under *Focus visibility*,
attached to one role.

A general obligation documented against a single usage is how it went missing
from the suite. That is the same shape as RFC-057's line-height item, one
release earlier — a rule scoped so narrowly that following the document
produces the gap.

## Scope

1. **Assert `border` at 3.0** against `background`, `surface` and
   `surface_raised`, for all four presets, in `mandatory_pairs`.
2. **Assert `text_muted` at `AA_TEXT`** against `background` and `surface`.
   Passes today; this is a ratchet, not a repair.
3. **Repair `light` and `dark`'s `border`** so the new assertions pass.
4. **Generalise the checklist's 3:1 rule** out of *Focus visibility* into
   *Contrast*, covering non-text boundaries as a class rather than one role.

## The repair is permitted, and RFC-036 prescribes its shape

RFC-036's additive-only covenant forbids changing a preset value — with one
carve-out:

> Changing values *inside* a preset **only** where a contrast test proves the
> change fixes an accessibility defect, recorded as **Fixed** in the CHANGELOG.

This is that case, and **its first exercise.** The order is not optional:

1. Add the assertion. **Watch it fail** on `light` and `dark`.
2. Change the border values until it passes.
3. Record as **Fixed**, citing the measured before/after.

Adding the assertion after the palette edit would satisfy the letter and lose
the proof the carve-out requires.

**No gate reopening is needed.** D-3/D-4 stay closed, because this is the
permitted path rather than a forbidden change.

## This changes appearance, and who sees it

Preset border colours change, so **every `design`-path consumer's borders
change** — cards, the dialog card, chrome. Both known design-path consumers are
affected.

Two things make that acceptable where a restyle would not be:

- It is an **accessibility repair**, which is the only value change the
  covenant permits, precisely because such changes should not be blocked.
- The direction is one-way: borders become *more* visible. Nothing becomes
  harder to see.

It must still be stated as an appearance change in the CHANGELOG and the
migration guide, not filed as a silent fix. One consumer (orbok) has an earlier
appearance change still unadopted; they should be told this one is a defect
repair rather than another styling decision.

## Non-goals

- **No new token roles or scales.** `border` and `text_muted` exist; only their
  values and assertions change.
- **No change to the high-contrast presets.** They pass at 19.8–21:1.
- **No change to `AA_TEXT` or `FOCUS_MIN`.** The constants are right.
- **No new contrast helper.** `contrast_ratio` is sufficient.
- **No pointer-target-size assertion.** Separate question (tekstide Q4).

## Open questions

**Q-1 — what value should `border` become?**
Three pairs must pass simultaneously per preset, and in `light`
`surface_raised == background` (both pure white), so the binding constraint is
the lightest surface. Do not pick a value that passes `background` and fails
`surface`. Compute it; do not eyeball it.

**Q-2 — should `border` be asserted at 3.0 or at `FOCUS_MIN`?**
They are the same number today. Using the named constant couples border
contrast to a constant meaning "focus minimum", which will read oddly. Suggest
a separate `NON_TEXT_MIN = 3.0` so the two obligations can diverge later
without one silently following the other.

**Q-3 — do any other roles carry an untested obligation?**
`border` and `text_muted` are the two tekstide found. A sweep of `Palette`'s 18
roles against the twelve asserted pairs would establish whether there is a
third, and is cheap to do while the file is open.

## Acceptance criteria

1. `mandatory_pairs` asserts `border` against all three surfaces and
   `text_muted` against `background` and `surface`, for all four presets.
2. **Failing-first evidence**: the new assertions fail on `light`/`dark` before
   the palette repair, with the output captured.
3. `light` and `dark` pass after repair; high-contrast presets unchanged.
4. Measured before/after ratios recorded in the CHANGELOG under **Fixed**.
5. The checklist's 3:1 rule covers non-text boundaries as a class.
6. Q-3's sweep answered — a third under-asserted role found, or stated as none.
7. `cargo test -p snora-design` passes; `render_semantics` unmodified.

## Compatibility and security

**Compatibility.** No API change. Preset **values** change, so `design`-path
appearance changes — borders become more visible in `light` and `dark`. Covered
by RFC-036's accessibility carve-out; no gate reopening.

**Security.** None.
