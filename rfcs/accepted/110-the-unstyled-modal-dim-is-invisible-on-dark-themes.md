# RFC-110 — The unstyled modal dim is invisible on dark themes, and a published guarantee is false

**Status.** Accepted 2026-10-19. Every open question ruled as suggested.
**Raised.** 2026-10-19, architect. Source: the rest of REQ-002 (tekstide: "every
theme colour comes from the host"), whose 0.52.0 status names this dim as the last
literal colour on the default path. Measuring it found a defect, and a published
claim it collides with.
**Release target.** 0.53.0 (Q-3). **Priority.** Small fix, with a correction that
must not be skipped.

---

## The finding

### 1. The dim does nothing on a dark page

The unstyled path dims behind a modal with a literal, `Color::from_rgba(0.0, 0.0,
0.0, 0.4)` (`crates/snora/src/render.rs:83`). The design path already explains why
that is wrong (`design/render.rs`, RFC-039/065): black composited over a dark page
"is close to a no-op". The design path therefore picks the **opposite pole** of the
page, white on dark and black on light, at `DIM_ALPHA = 0.44`.

Measured over every built-in iced theme (`Theme::ALL`, 22 themes), as the contrast
between the page and the dimmed page:

| | Today: black at 0.40 | Pole at 0.40 | **Pole at 0.44** (the design path) |
|---|---|---|---|
| 16 dark themes | **1.02 – 1.33** | 3.36 – 3.83 | **3.74 – 4.36** |
| 6 light themes | 2.70 – 2.85 | 2.70 – 2.85 | **3.05 – 3.24** |

*(Corrected 2026-10-19: the split is **16 dark / 6 light**, not 14 / 8 as first written. The ranges were right; the counts were the architect's miscount. Found by the dev team in RFC-110's review.)*

- **Moonfly is 1.02:1.** On Dark, which is iced's own dark theme, it is 1.25:1.
- **On Dark, no alpha of black can work.** The page's luminance is 0.026, so black
  cannot exceed 1.52:1 even at full opacity.
- **The light themes are short too.** At 0.40 even black over white is 2.85:1, below
  the 3:1 the design path was repaired to clear (RFC-065, measured on the card).

A user on a dark theme sees a modal open over a page that appears unchanged, and
nothing tells them the page behind is inert.

### 2. The guarantee this must change is already false

The unstyled value is kept, by its own comment, because "the gating invariant
depends on this value never changing". The invariant is RFC-037's, and it is
published as a guarantee in three places:

- `docs/src/design/overview.md:29`: "With `design` inactive, snora's rendered
  output is unchanged from v0.25 — this is a guarantee, not an aspiration."
- `docs/src/design/engine-surfaces.md:21`: "byte-for-byte unchanged with `design`
  inactive (RFC-037's gating invariant)".
- `docs/src/guides/migration-0.25-to-0.26.md:9`: the same promise to consumers.

**It has not been true since at least 0.42.0.** That release's guide opens with
"Rendered appearance change on the default path — no feature needed" (the toast
colours). Later releases did the same:
- 0.50 sidebar buttons;
- 0.51 tab bar edges and toast close target;
- 0.52 host-derived label sizes, the theme's Warning fill, and wrapping.

Each shipped with a migration guide naming it. None amended the invariant.
`engine-surfaces.md:270` still says `WARNING_COLOR` "is unchanged … changing them
there would break the gating invariant", yet RFC-104 removed it in 0.52.0.

**What is still true** is narrower, and it is asserted:
- `render_semantics.rs`, which must pass unmodified, pins **composition**: layer
  order, which surfaces materialise, dismissal, and mirroring;
- no public signature has changed for `design`'s sake;
- appearance on the default path changes only through an accepted RFC that names
  the change in a migration guide.

## Proposal

**R-0 — correct the guarantee first, in the same release.**
- Restate it in all three places as what is true and asserted: composition unchanged
  by `design` (`render_semantics`); default-path appearance changes only by an
  accepted RFC, named in that minor's migration guide.
- Add a correction note saying the stronger sentence was published, and has been
  false since at least 0.42.0.
- Fix `engine-surfaces.md:270` and the `render.rs` comment.
- This is a **withdrawn claim**, so the 0.53.0 cut decides correspondence for it
  explicitly (RFC-080).

**R-1 — the unstyled dim takes the opposite pole, from the theme.** Use
`theme.extended_palette().is_dark` to pick white or black, at **0.44**. iced's own
classification is the algorithm `snora_design::surfaces::is_dark` reimplements, so
the two paths classify a background identically. Name the alpha as a constant that
points to `DIM_ALPHA` and says why it is not shared: `snora-design` is an optional
dependency of the engine, and frozen. RFC-106 did the same for `disabled_alpha`.

**R-2 — assert it.** For every theme in `Theme::ALL`, dimmed against undimmed page
≥ **3.0:1**, plus a pole test (white on dark, black on light) mirroring the design
path's. Once RFC-108's pixel helper exists, read the drawn dim once as well.

## Open questions

**Q-1 — alpha.** **Suggest 0.44**, the design path's value. It is the lowest of the
two measured that clears 3.0 on all 22 themes (minimum 3.05, Tokyo Night Light).
That makes the two paths identical in rule and value, so "derived from the theme"
means the same thing on both.

**Q-2 — the floor.** **Suggest 3.0:1 dimmed vs undimmed**, WCAG 1.4.11's figure for
a visual cue that carries state; here the state is "the page is inert". It is not a
WCAG requirement on dims as such, and the guide will say so rather than imply one.

**Q-3 — release.** **Suggest 0.53.0**, together with R-0. Dark-theme users get a
light veil where there was effectively none, which is a visible appearance change and
a migration-guide item. Light-theme users get a dim slightly darker (0.40 → 0.44).

## What this is not

- **Not a covenant change.** `snora-design` is read, not changed; `DIM_ALPHA`
  stays where it is.
- **Not a restyle of the dialog.** The unstyled path still has no card. Card
  contrast is the design path's and unchanged.

## Acceptance criteria

1. R-2's assertion over `Theme::ALL` is **shown failing** on today's literal (16
   dark themes below 3.0, and 6 light), then passing.
2. The pole test is shown failing with the pole forced to black.
3. `render_semantics.rs` passes **unmodified**. Composition is unchanged, which is
   the part of the invariant that is true.
4. R-0's three restatements, the correction note, and the two stale sentences:
   quote the diff.
5. CHANGELOG under Fixed (the dim) and Changed (the guarantee restated), and the
   migration guide carries both.

---

## Rulings, 2026-10-19

**Q-1:** alpha 0.44. **Q-2:** dimmed vs undimmed page ≥ 3.0:1, with the guide saying
it is snora's floor, not a WCAG requirement on dims. **Q-3:** 0.53.0, together with R-0.

**R-0's docs part, done by the architect, 2026-10-19:**
- `docs/src/design/overview.md` restates the promise as composition (asserted)
  plus RFC-named appearance changes, with a correction note.
- `docs/src/design/engine-surfaces.md` does the same at l.18. Its `WARNING_COLOR`
  paragraph is marked superseded by RFC-104.
- `docs/src/guides/migration-0.25-to-0.26.md`, being historical, keeps its text and
  gains a correction note.

**The `render.rs` comment** ("the gating invariant depends on this value never
changing") is code, and is in the handoff. Correspondence for the withdrawn claim
is decided at the 0.53.0 cut.
