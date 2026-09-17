# Implementation handoff — RFC-099: the sidebar rail is narrower than its buttons

**RFC.** `rfcs/accepted/099-the-sidebar-rail-is-narrower-than-its-buttons.md`
— read its *Rulings* section; it sharpens acceptance criterion 1.
**Release target.** 0.50.0.
**Rulings.** Q-1 = **(a)** derive horizontal padding. Q-2 = leave the
styled/unstyled split alone.
**Touch list.** `crates/snora-widgets/src/sidebar.rs`,
`crates/snora-widgets/src/design/widget.rs` and its `tests.rs`, one new test
under `crates/snora/`, `CHANGELOG.md`. **Not** `crates/snora-design`,
**not** `crates/snora-style`, **not** `docs/` — the checklist narrowing (R-4)
and the migration guide are the architect's.

---

## Already established, so you do not redo it

- Both defects were verified in source: `container(col).width(RAIL_WIDTH)
  .padding(geometry.padding)` pads all four sides by 16, leaving 32 px for a
  `Fixed(48)` button; `button(icon)` has no centring container.
- **Both variants, every preset.** `SideBarGeometry::unstyled()` is
  `padding: 16.0` too, and all four presets are `Density::Comfortable`
  (`Spacing::lg = 16`). Nothing currently fits.
- **Unchanged at 0.10.0, 0.20.0, 0.30.0, 0.40.0 and 0.49.0.**
- The fix is **outside RFC-036's frozen surface** as long as `Spacing` is not
  touched. See criterion 6.
- `iced_core` 0.14: `Padding::from([a, b])` is **`[vertical, horizontal]`**
  (`top`/`bottom` = `a`, `left`/`right` = `b`). `Padding` also has
  `.vertical(..)` / `.horizontal(..)` builders. Verified in the registry source.
- `iced_test` 0.14's `Simulator::find` returns a target with
  `visible_bounds()`, and `point_at(Point)` + `simulate(click())` fires at an
  arbitrary coordinate — `render_semantics.rs` already does this. Verified in
  the registry source.

## The trap in Q-1 (a) — read before writing any test

Deriving horizontal padding as `(RAIL_WIDTH − BUTTON_SIZE) / 2` makes
`2 × padding + BUTTON_SIZE ≤ RAIL_WIDTH` **true by arithmetic for any constants
at all**. A test that computes that inequality from the same derivation passes
before any regression and after every regression. It would be a copy of
`side_bar_geometry_matches_mapping_all_presets`'s defect — asserting the
implementation instead of the property — inside the RFC written about that
defect.

**So the tests below measure the layout iced produces, not the numbers we feed
it.** A regression that reverts horizontal padding to 16 must fail them.

## Unit 1 — a failing-first test of the rendered fit, before any fix

Write it first, run it against the current code, and **record it failing**.

**Recommended shape — a coordinate click, which needs no identifiers and no
bounds API.** Render a sidebar alone in `iced_test::Simulator`. Under the
current geometry the first button spans roughly `x ∈ [16, 48]`; under the fixed
geometry it spans `x ∈ [8, 56]`. Click at a point inside the intended 48 px
button but outside the current 32 px one — near `x = 12` at the button's
vertical centre — and assert that button's `on_press` message is produced.

- **Today:** the click lands in rail padding → no message → **fails**.
- **After the fix:** hits the button → passes.
- **If horizontal padding is ever reverted:** fails again. That is the property.

Derive the coordinates from `RAIL_WIDTH` / `BUTTON_SIZE` and the vertical
padding in the test, with a comment, rather than hardcoding `12` — and check
both edges (a point near `x = 12` and one near `x = 52`), so a button that
widened toward only one side does not pass.

**Cover both variants and all four presets:** `snora_widgets::app_side_bar`
(unstyled) and the `design`-gated styled `app_side_bar` for `light`, `dark`,
`high_contrast_light`, `high_contrast_dark`.

**Where it lives.** `iced_test` is a dev-dependency of `crates/snora` only.
Prefer a test there, reaching the sidebar through `snora`'s re-exports, over
adding a new dev-dependency edge to `snora-widgets`. **Gate it on the features
it needs** (`widgets`, plus `design` for the styled cases): the CI feature
matrix runs `cargo test` under `--no-default-features` and eight other
combinations, and an ungated test that names a `design` item will break those
jobs rather than skip. If placing it in `snora-widgets` turns out materially
simpler, say so and why in your report — do not do it silently.

**A cheap arithmetic guard is fine in addition, not instead.** Asserting
`BUTTON_SIZE <= RAIL_WIDTH` (so the derived padding cannot go negative under a
future constant edit) is a meaningful check, because it can fail. Asserting the
derived inequality cannot.

## Unit 2 — the geometry, per Q-1 (a)

- Split `SideBarGeometry::padding` so the rail's **vertical** padding keeps
  today's source (unstyled: `16.0`; styled: `tokens.spacing.lg`) and the
  **horizontal** padding is `(RAIL_WIDTH − BUTTON_SIZE) / 2`, **computed in one
  place** — a `const` beside `RAIL_WIDTH`/`BUTTON_SIZE` is the obvious home,
  since both inputs are constants.
- Apply it with `Padding::from([vertical, horizontal])` or the builders. Mind
  the order.
- Rename the field for what it now is (e.g. `vertical_padding`) rather than
  leaving `padding` meaning half of what it used to, and **update the mapping
  test to match** — keep it; it answers a real question about token wiring for
  the dimensions that remain token-mapped.
- The `gap` and `button_radius` mappings do not change.

**Do not change `Spacing`.** Not `comfortable().lg`, not anything in
`crates/snora-design`. That would be a forbidden change under RFC-036, would
reset D-3/D-4 re-earned at 0.47.0, and would move every other component
reading `lg`.

## Unit 3 — centre the icon

Wrap the icon in a container centred on both axes inside the button, in the
shared `build_side_bar` so both variants inherit it.

**Check iced's button padding.** An iced `button` has non-zero default padding,
so "centred in the container" and "centred in the 48 × 48 button" can differ if
the container does not fill the padded content area. Read the default in the
registry source rather than assuming, and decide whether the button's padding
should be zeroed for an icon-only fixed-size button. State what you chose.

**Test — measured, and it must fail if the container is removed.** Use
`Simulator::find(..)` → `visible_bounds()` to compare the icon's centre with the
button's centre, within a small tolerance you state and justify. Demonstrate it
failing against the uncentred code.

**Selecting the icon without inventing a production identifier.** If the only
way to `find` the icon is to attach a new `Id` to a production widget, **stop
and escalate** — rendered-surface identifiers are a documented, governed surface
(RFC-047), and adding one to make a test possible is a public-surface decision,
not an implementation detail. Test-only selection (by text content, or a test
fixture that supplies its own identifiable icon element) is preferred.

## Criterion 6 — confirm the covenant, do not assume it

In your report, quote:

```bash
git diff <base> -- crates/snora-design crates/snora-style
```

and show it empty. D-3/D-4 were reset once this year and re-earned five days
ago; "outside the frozen surface" is the RFC's argument, and this command is its
evidence.

## Rendered target size — note it, no action

After the fix the sidebar's pointer target becomes **48 × 48** (from 32 × 48).
Both clear WCAG 2.5.8's 24 × 24 floor; no conformance claim moves. Mention it in
the CHANGELOG because orbok's record cites 2.5.8 and a changed target size is
the kind of thing they will want to see stated rather than infer.

## CHANGELOG

Under **Fixed**, crediting **orbok** — their owner noticed it and they traced it
to source. It must say:

- the buttons rendered 32 × 48 instead of 48 × 48, and the icon sat above
  centre;
- **both** the unstyled and styled variants, every preset, since at least 0.10.0;
- **this is a rendered-appearance change on the default path** and invalidates
  visual baselines that include a sidebar;
- the existing geometry test asserted token wiring, not fit — and the new tests
  measure rendered layout for that reason;
- rail width is unchanged, so body content does not reflow.

## Gate suite

The usual. Plus `scripts/check-wcag-floors.sh` (you are in a crate with contrast
tests), the feature matrix locally for at least `--no-default-features`,
`--features widgets` and `--all-features` given Unit 1's gating, and
`cargo +1.88 check --workspace --all-features --locked`.

## Acceptance criteria

1. The rendered-fit test exists, covers unstyled + four styled presets, is
   **shown failing before the fix**, passes after, and — shown by reverting
   horizontal padding to 16 in a scratch edit — **fails again on regression**.
2. Horizontal padding derived in one place; vertical padding and gap keep their
   sources; field renamed and mapping test updated.
3. Icon centred via a container in `build_side_bar`; centring test **shown
   failing** with the container removed; no new production identifier.
4. `git diff` on `crates/snora-design` and `crates/snora-style` quoted and empty.
5. Feature-gated so every feature-matrix combination still builds and tests.
6. CHANGELOG under Fixed, crediting orbok, stating the appearance change.
7. Nothing under `docs/` touched.
