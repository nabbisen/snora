# RFC 065 — The modal dim is a surface nothing measures against, and `light` fails on it

**Status.** Proposed
**Tracks.** Accessibility. Reported by **tekstide** (2026-08-18) as a finding in
*their* codebase; transposed to snora, where it fails.
**Touches.** `crates/snora/src/design/render.rs`,
`crates/snora-design/src/tests.rs`,
`docs/src/contributing/accessibility-checklist.md`,
`docs/src/guides/migration-0.36-to-0.37.md` (new), `CHANGELOG.md`.
**Release target.** 0.37.0 (minor — `design`-path appearance change).

## Summary

RFC-063 closed the *role* axis: no `Palette` role can be added without declaring
where it renders. It did not close the *surface* axis. `Palette::usages` can only
name the three neutral surfaces — and snora composites a fourth at render time
that appears in no assertion anywhere: **the modal dim.**

Measured against it, the dialog card in the `light` preset is distinguishable
from its own backdrop at **2.85:1**, below SC 1.4.11's 3:1, by either available
signal.

## Where the finding came from

tekstide reported it as a defect in their own code, having gone looking for our
`accent`/`danger` case in their palette:

> **there is a third surface we never enumerate at all.** A modal's real backdrop
> is neither `background` nor `surface_elevated`: it sits over a scrim, which is
> itself composited over whatever was behind it… **We pass by luck rather than by
> declaration** — not an unmeasured role, an unmeasured *surface*.

They pass. We do not.

## The evidence

`design::render`'s `dim_color` derives the dim from the token bundle: black in
light presets, white in dark, at `DIM_ALPHA = 0.4`, composited over whatever is
behind. The dialog card is `card_raised` — a `surface_raised` fill with a 1 px
`border` stroke.

Under SC 1.4.11 the card is identifiable if **either** its border or its fill
clears 3:1 against the adjacent backdrop. Worst case across all three surfaces
the dim can sit over:

| preset | worst backdrop | border ǀ dim | fill ǀ dim | verdict |
|---|---|---|---|---|
| `dark` | `background` | 1.00 | **3.18** | pass, on fill alone |
| `high_contrast_dark` | `surface_raised` | **5.25** | 3.77 | pass |
| `high_contrast_light` | `background` | **7.37** | 2.85 | pass, on border alone |
| **`light`** | `background` | 1.19 | 2.85 | **FAIL** |

In `light`, `background` is pure white, so the dim over it is the *lightest* the
dim can be and 2.85 is a true floor — any real content behind it improves the
figure, never worsens it.

Two details worth recording:

- **`dark`'s border measures 1.00:1 against the dim** — the border and the dim
  land on the same luminance. RFC-039 chose the dialog card as border-defined
  rather than shadow-defined, and against its own backdrop in `dark` the border
  contributes *nothing*; the card is carried entirely by its fill.
- **`high_contrast_light` passes on its border alone**, its fill also being
  2.85. The high-contrast presets are not uniformly safe here — they are safe
  for a different reason in each direction.

## Why RFC-063's mechanism did not catch it

`Palette::usages` declares, per role, the **`Palette` surfaces** it renders on.
The dim is not a `Palette` role. It is derived at render time in the `snora`
crate, from a constant that lives beside the renderer, and composited with an
alpha. `snora-design` — where the contrast suite lives, and which is iced-free
by hard constraint — cannot see it.

So this is not a gap someone forgot to fill. **The vocabulary has no way to
express it**, and that is the more interesting half of this RFC. RFC-063 made
adding a *role* impossible without declaring its surfaces; nothing makes adding
a *surface* impossible without declaring what must be measured against it.

## Scope

1. **Assert the dialog card against the dim**, for all four presets, at
   `NON_TEXT_MIN`, with the either-signal rule (border **or** fill) expressed
   explicitly rather than assumed.
2. **Repair `light`** so the assertion passes — see Q-2.
3. **Extend the checklist** to cover composited/derived surfaces as a class, the
   same widening RFC-058 applied to non-text boundaries.
4. **Record the axis** in `api-governance.md` beside RFC-063's role rule: a new
   *surface*, derived or composited, carries the same declaration obligation as
   a new role.

## Non-goals

- **No new `Palette` role.** RFC-036 forbids it without reopening D-3/D-4, and
  the dim is deliberately derived rather than a token.
- **No change to the unstyled path.** It dims with a literal
  `Color::from_rgba(0.0, 0.0, 0.0, 0.4)` and draws no card, so SC 1.4.11's
  component-boundary clause does not attach the same way. Out of scope, and
  worth stating so nobody "fixes" both for symmetry.
- **No shadow on the dialog card.** RFC-039 removed it deliberately; shadows
  carry almost no information in high-contrast presets, which is the reasoning
  two consumers have now independently found useful.
- **No change to `NON_TEXT_MIN`.**
- **No re-litigation of `border`'s value** (RFC-058). Its three asserted pairs
  still pass; this is a fourth surface, not a regression in the first three.

## Open questions

**Q-1 — how should a derived surface be declared, given `snora-design` cannot
see it?** Three shapes, and this is the architectural half of the RFC:

- **(a)** Assert it in `snora`'s own test suite, where `dim_color` lives. Cheap
  and local; splits contrast assertions across two crates, which is how a rule
  gets applied unevenly.
- **(b)** Move the *derivation* into `snora-design` as a pure function over
  `Tokens` — `snora-design` already has `composite_over` — and assert it beside
  the rest. Keeps one contrast suite; adds a token-crate function whose only
  consumer is the engine.
- **(c)** Extend `RoleUsage` to carry computed surfaces, not just `Palette`
  fields. Most general and most invasive.

**(b)** is the leading candidate: `composite_over` exists in `snora-design`
precisely for this and — per tekstide's Q3 — currently has no in-repo caller.
This would be its first, which is a point in its favour and slightly against it.

**Q-2 — what repairs `light`?** The cheapest correct answer appears to be
`DIM_ALPHA`, and the numbers are close enough that the RFC should not pretend
otherwise:

| `DIM_ALPHA` | `light` | `dark` | `hc_light` | `hc_dark` |
|---|---|---|---|---|
| **0.40** (today) | **2.85** | 3.18 | 7.37 | 5.25 |
| **0.42** | **3.04** | 3.40 | 6.91 | 4.89 |
| 0.44 | 3.24 | 3.64 | 6.48 | 4.56 |

**0.42 clears every preset**, and it improves `dark` as well. But 3.04 is a 1.3%
margin, which is thinner than this project has been willing to accept since
RFC-058 recorded `dark`'s 4.526:1 as a trap for the next edit. **0.44 or 0.45
buys real headroom for a barely-perceptible extra dim** and should be weighed
against it.

Note the constant's comment says it *"matches the unstyled path's literal"* —
changing it breaks that symmetry deliberately, which is a decision to record
rather than a side effect.

**Q-3 — is a 5% miss worth an appearance change at all?** 2.85 against 3.0.
RFC-058 set the precedent: measure, and if it fails, repair — that RFC's own
`text_muted` miss was 4.46 against 4.5, a 0.9% shortfall, and was repaired.
Consistency argues yes. State the reasoning either way.

## Acceptance criteria

1. The dialog card is asserted against the dim for all four presets, with the
   either-signal rule explicit.
2. **Failing-first evidence**: the assertion fails on `light` before any repair,
   captured.
3. `light` passes after repair; the other three presets still pass, with
   before/after for all four.
4. Q-1 answered, with the reasoning for the chosen shape recorded.
5. The checklist covers composited/derived surfaces as a class.
6. `api-governance.md` records the surface axis beside RFC-063's role rule.
7. `render_semantics` passes unmodified.

## Compatibility and security

**Compatibility.** If Q-2 changes `DIM_ALPHA`, every modal on the `design` path
renders a slightly stronger dim — a rendered appearance change requiring a
migration-guide note, the treatment RFC-058's border change received. No API
change. The unstyled path is untouched.

**Security.** None. Worth noting the inverse, though: a modal whose boundary is
below 3:1 is harder to distinguish from the page behind it, and two consumers
treat modal containment as a security-adjacent property. This is a legibility
defect, not a containment one, but they sit next to each other.

## Credit

tekstide, who found the axis in their own codebase after our review found the
`accent`/`danger` instance in theirs, and reported it despite passing.
