# Implementation handoff — RFC-110: the unstyled modal dim is invisible on dark themes

**RFC.** `rfcs/accepted/110-the-unstyled-modal-dim-is-invisible-on-dark-themes.md`
**Release target.** 0.53.0.
**Rulings.**
- Q-1: alpha 0.44.
- Q-2: dimmed vs undimmed page ≥ 3.0:1.
- Q-3: 0.53.0.

**Touch list.**
- `crates/snora/src/render.rs`;
- its unit tests;
- `CHANGELOG.md` (`[Unreleased]`).

**Not** `crates/snora/src/design/render.rs`'s derivation, `snora-design` (frozen,
`DIM_ALPHA` stays put) or `docs/`. **R-0's docs part is done** (overview,
engine-surfaces, the 0.25→0.26 guide); see the RFC's Rulings.

## Already established

- **The literal and where it is drawn.**
  - `ChromeStyle::unstyled()` sets `dim_color: Color::from_rgba(0.0, 0.0, 0.0, 0.4)`
    (`render.rs:83`), with the comment "the gating invariant depends on this value
    never changing". That comment is now false (RFC-110 R-0).
  - `dim_backdrop` paints it inside a style closure that **already receives the
    theme** (`render.rs:~277`, `move |_theme| …`). So the theme-derived dim needs
    **no new parameter**: derive it in that closure when the chrome style is
    unstyled.
- **The design path** passes a fixed colour from `snora_design::surfaces::modal_dim`,
  with the opposite pole by OKLCH lightness at `DIM_ALPHA = 0.44`. Leave it exactly
  as is.
- **No test pins the unstyled 0.4** (checked: the dim tests are all in
  `design/render/tests.rs`).
- **Measured** (scratch probe, `Theme::ALL`, page vs dimmed page):

  | Themes | Black at 0.40 (today) | Pole at 0.44 |
  |---|---|---|
  | 14 dark | 1.02–1.33 | 3.74–4.36 |
  | 8 light | 2.70–2.85 | 3.05–3.24 (minimum: Tokyo Night Light) |

- **iced's `extended_palette().is_dark`** is the classification
  `snora_design::surfaces::is_dark` reimplements, so the two paths agree on every
  background.

## Unit 1 — failing-first

In `render.rs`'s tests, add these two:

1. **For every `Theme::ALL`, the unstyled dim composited over
   `background.base.color` is ≥ 3.0:1 against it.** Use the crate's existing contrast
   helper, or the toast tests' `contrast_ratio`, which is checked against WCAG's
   reference. **Today it fails**, on all 22 themes. Record the list.
2. **The pole test:** white on `is_dark` themes, black otherwise, at the named alpha,
   mirroring `design/render/tests.rs::dim_color_picks_the_opposite_pole`.

## Unit 2 — the fix

- Add `const UNSTYLED_DIM_ALPHA: f32 = 0.44;`. Its comment names
  `snora_design::surfaces::DIM_ALPHA` as the value it matches, and gives the reason
  it is not shared: `snora-design` is an optional dependency here, and frozen. That is
  RFC-106's `CHIP_DISABLED_ALPHA` pattern.
- The unstyled chrome style **derives** the dim from the theme, as a variant or an
  `Option`, whichever reads best. The design path keeps passing its fixed token
  colour. **Do not change the design path's value or rule.**
- **Rewrite the false comment** to say what is true:
  - the unstyled dim follows the theme by the same rule as the design path (RFC-110);
  - the invariant that still binds is composition, asserted by
    `render_semantics.rs`.

## Check each assertion can fail

Back up to named files with `cp`, break, run, restore, then `sha256sum -c`. Never
`git checkout`/`restore`.

| Assertion | Breakage |
|---|---|
| ≥ 3.0 over `Theme::ALL` | restore the literal (Unit 1 already shows this) |
| ≥ 3.0 over `Theme::ALL` | alpha 0.40, which must fail on the 8 light themes |
| pole | force the pole to black |

## Acceptance criteria

1. Units 1–2, with the table's breakages shown failing.
2. `render_semantics.rs` passes **unmodified**, and the design-path dim tests pass
   unmodified.
3. `snora-design` and `snora-style` unchanged: quote the diff.
4. CHANGELOG under Fixed: the dim on dark themes, with the before and after ranges.
   The guarantee's restatement is the architect's line, in the migration guide.
5. **Optional, if RFC-108's R-1 lands first:** read the drawn dim once with
   `frame_pixels`, on Dark, as a check that the derivation reaches the screen.
