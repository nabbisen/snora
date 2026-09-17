# RFC-099 — The sidebar rail is narrower than its buttons

**Status.** Accepted 2026-09-17. Q-1 = (a), Q-2 = leave it.
**Raised.** 2026-09-17, architect, on a report from **orbok**.
**Release target.** 0.50.0.
**Found by.** orbok's owner, who noticed the sidebar icons looked off-centre;
orbok traced it to source and measured it in a real window.

---

## The finding

Two defects in `app_side_bar`, both verified against source. orbok's
measurements are the rendered confirmation; the mechanism below was read from
the code, not inferred from their numbers.

### 1. The buttons are laid out 32 px wide, not 48

`crates/snora-widgets/src/sidebar.rs`:

```rust
const RAIL_WIDTH: f32 = 64.0;
const BUTTON_SIZE: f32 = 48.0;
// …
button(icon).width(BUTTON_SIZE).height(BUTTON_SIZE)
// …
container(col).width(RAIL_WIDTH).padding(geometry.padding)
```

`padding(f32)` is uniform on all four sides, and `geometry.padding` is **16**.
So the rail's content width is `64 − 2 × 16 = 32`, and a `Length::Fixed(48)`
button inside it is clamped to the space available: **32 × 48**. orbok measured
the active highlight at 39 × 58 px at display scale 1.2 — **32.5 × 48 logical**,
which matches.

What was meant to read as a square reads as a tall, narrow pill.

### 2. The icon sits above centre

`build_side_bar` passes `icon_element(&item.icon)` straight into `button(..)`
with no centring container. iced lays button content from the top-left inside
the button's own padding, so a 14 px glyph in a 48 px-tall button sits in its
upper part — about **13 logical px above centre** by orbok's measurement.

### It is universal, and it is old

- **Both variants.** orbok named the styled variant (`padding:
  tokens.spacing.lg`). The **unstyled** `app_side_bar` has the identical
  literal — `SideBarGeometry::unstyled()` is `padding: 16.0` — so every sidebar
  snora renders is affected, not only the token-derived one.
- **Every preset.** All four named presets are `Density::Comfortable`, whose
  `Spacing::lg` is 16, and compact resolution is deferred. **No configuration
  satisfies the geometry.**
- **At least forty minors.** `rail = 64`, `button = 48`, `padding = 16`, no
  centring — identical at the 0.10.0, 0.20.0, 0.30.0, 0.40.0 and 0.49.0 tags.

## Why nothing caught it — two checks, two different blind spots

### The geometry test asserts wiring, not fit

`side_bar_geometry_matches_mapping_all_presets` (in
`crates/snora-widgets/src/design/widget/tests.rs`):

```rust
assert_eq!(g.padding, t.spacing.lg, "{name}: sidebar padding should map to Spacing::lg");
```

and the function under test:

```rust
fn side_bar_geometry(tokens: &Tokens) -> SideBarGeometry {
    SideBarGeometry { padding: tokens.spacing.lg, /* … */ }
}
```

**The test restates the implementation.** It establishes that the padding is
*wired to* `Spacing::lg`, and it cannot fail for a padding that is wired
correctly and does not fit. The property that matters — that the rail can hold
its buttons — was never asserted anywhere. orbok named it exactly:

> `2 × padding + BUTTON_SIZE ≤ RAIL_WIDTH`, for each preset.

This is the defect class RFC-090 through RFC-098 were raised against, and
RFC-098 was about a gate in the same shape one week ago: a check whose green
was read as coverage of a property it never tested.

### The pointer-target checklist put this in the wrong bucket

`accessibility-checklist.md` § *Pointer target size* says the **width** axis is
*"review-only, not asserted"*, because width is `content_advance + 2 ×
horizontal_padding` and `content_advance` depends on the rendered string, the
font and the shaping engine — which snora cannot compute without a renderer. It
then names *"an icon-only or single-glyph button, in particular"* as the case
needing a hand check.

**That reasoning is correct for text buttons and false for this one.** The
sidebar button's width does not depend on content at all — it is a fixed
`BUTTON_SIZE` constrained by `RAIL_WIDTH − 2 × padding`, three constants and no
rendering. So the one case the checklist singled out for manual review was
also a case where manual review was unnecessary, because the arithmetic could
have been a test. It fell into the review-only bucket by category, and the
review did not happen.

## What this is not

- **Not a covenant change.** RFC-036's frozen surface is `snora-design`'s
  public items and `snora_style`'s public functions. `RAIL_WIDTH`,
  `BUTTON_SIZE`, `SideBarGeometry` and `side_bar_geometry` are private to
  `snora-widgets`. **The fix does not reset D-3 or D-4 — provided it does not
  touch `Spacing`.** Changing `Spacing::comfortable().lg` to make this fit would
  be a forbidden change and would reset both gates re-earned five days ago; it
  would also move every other component that reads `lg`. The fix belongs in the
  sidebar.
- **Not a WCAG 2.5.8 regression, and not a withdrawn claim.** The rendered
  target is 32 × 48 logical, above the 24 × 24 floor. orbok's conformance record
  cites us for 2.5.8, and **its status does not move** — stated here so they do
  not have to establish that themselves.

## Proposal

**R-1 — assert the fit, for every variant and every preset.** A test asserting
`2 × horizontal_padding + BUTTON_SIZE ≤ RAIL_WIDTH` for the unstyled geometry
and for `side_bar_geometry(&t)` across all four named presets. **It fails today
on all five**, which is the failing-first evidence — no perturbation needed,
and that is worth saying because it means the defect is demonstrable rather than
argued. Keep the mapping test; it answers a different, legitimate question.

**R-2 — make the geometry fit.** Shape is Q-1.

**R-3 — centre the icon** on both axes within `BUTTON_SIZE`, in the shared
`build_side_bar`, so both variants inherit it.

**R-4 — narrow the checklist's claim.** The width axis is review-only **for
content-sized controls**. Fixed-size controls have computable width and should
be asserted. One sentence, and it stops the next icon-only primitive landing in
the same bucket.

## The appearance change, and who it reaches

**This changes rendered appearance on the default path** — every sidebar, both
variants. It will invalidate visual baselines for any team doing capture-based
verification, and it must be disclosed in the migration guide as such.

**orbok has explicitly not worked around it** — *"no custom rail, no padding
override, so that your fix, when it lands, is what we see."* That is a
consequence worth honouring: whatever we ship is what their owner sees, so the
fix should be the finished answer rather than a first pass.

**apimokka is unaffected in practice** — they compile without `snora-widgets`.

## Open questions

**Q-1 — which dimension gives?** Three ways to satisfy the invariant:

| Option | Change | Visible consequence |
|---|---|---|
| **(a) Derive horizontal padding** | `(RAIL_WIDTH − BUTTON_SIZE) / 2` = 8; vertical padding unchanged | Rail stays 64 px, **body content does not reflow**; button becomes the intended 48 × 48 |
| (b) Widen the rail | `RAIL_WIDTH` → `BUTTON_SIZE + 2 × padding` = 80 | Button 48 × 48, but **the body shifts 16 px** in every application |
| (c) Shrink the button | `BUTTON_SIZE` → 32 | Rail and body unchanged, but the target shrinks to 32 × 32 and the icon rail gets visibly denser |

**Suggest (a).** It is the only option that produces the geometry the constants
already describe — a 64 px rail of 48 px square buttons — without moving any
surface other than the sidebar. Deriving the value rather than writing `8.0`
makes the invariant hold **by construction**, so R-1 guards against a future
constant change rather than re-checking a literal.

The cost of (a) is that the styled variant's *horizontal* padding stops being a
token. RFC-040 made the styled geometry token-derived, and this makes one
dimension of it derived from the rail instead. **That is correct** — the rail's
horizontal padding is not a spacing choice, it is what is left over — but it is
a real departure from RFC-040's framing and should be decided rather than
slipped in. Vertical padding and the inter-button gap stay token-mapped.

**Q-2 — is the styled/unstyled distinction still carrying its weight here?**
Both variants now share the same failure and the same fix, and after (a) their
only horizontal difference disappears. Suggest: leave it — out of scope, and the
variants still differ on gap and radius — but note it for the next time the
sidebar is touched.

## Acceptance criteria

1. The fit invariant is asserted for the unstyled geometry and all four styled
   presets, and **shown failing against the current code before the fix**.
2. The geometry satisfies it, via Q-1's ruled shape, **without changing
   `Spacing`** — verified by `git diff` on `crates/snora-design`.
3. The icon is centred on both axes in `build_side_bar`, with a test that fails
   if the centring container is removed.
4. `accessibility-checklist.md` narrows "width is review-only" to content-sized
   controls.
5. Migration guide discloses the appearance change and the invalidated
   baselines; CHANGELOG under **Fixed**, crediting orbok.
6. `api-freeze-review.md`'s D-3/D-4 rows unaffected — confirmed, not assumed.

---

## Rulings, 2026-09-17

**Q-1 — (a).** Horizontal padding is derived, `(RAIL_WIDTH − BUTTON_SIZE) / 2`,
in one place. Vertical padding and the inter-button gap keep their current
sources (token-mapped in the styled variant, literals in the unstyled one). The
departure from RFC-040's all-token framing is accepted deliberately: the rail's
horizontal padding is what is left over, not a spacing choice.

**Q-2 — leave it.** The styled/unstyled split stays; out of scope.

### A consequence of (a) the handoff must guard against

**Deriving the padding makes the RFC's own invariant true by arithmetic.**
`2 × ((R − B) / 2) + B = R` for any `R` and `B`, so a test that recomputes
`2 × padding + BUTTON_SIZE ≤ RAIL_WIDTH` from the same derivation would pass
forever — restating the implementation, which is precisely the defect this RFC
found in `side_bar_geometry_matches_mapping_all_presets`. Acceptance criterion 1
is therefore sharpened: **the fit must be asserted against the layout iced
actually produces** (`iced_test::Simulator`, already a dev-dependency of
`crates/snora`), so the test fails if the padding is ever reverted, not only
before the fix.
