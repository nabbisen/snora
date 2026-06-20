# Migrating from 0.23 to 0.24

v0.24 is an **architect-review cleanup release** addressing findings from the
v0.23 architect review. One breaking change to an internal-only method.

## What changed

### `Palette::roles()` removed from public API

`Palette::roles() -> [Color; 18]` was previously `pub`. It is now
`#[cfg(test)] pub(crate)` — test-only and crate-internal. This method was
only intended for internal validation; the fixed return type `[Color; 18]`
would lock the role count against future `#[non_exhaustive]` extensions.

**If your code called `palette.roles()`**: access fields directly.
`palette.text_primary`, `palette.accent`, `palette.success`, etc.

### Chip selected state: solid accent background (accessibility fix)

Prior to v0.24, `chip::filter` and `chip::removable` in the selected state
used a semi-transparent accent tint (alpha 0.15–0.30). After compositing
over the surface, hovered and pressed states fell below WCAG AA (4.5:1) in
light and dark presets.

The selected style now uses a solid `accent` background + `accent_text`
foreground (≥6.7:1 across all presets). New contrast tests verify this.

**Visual change**: selected chips now appear with a fully saturated accent
background rather than a light tint. Unselected chips are unchanged.

### Binary-size and build-cost measurement extended to `design` feature

Scripts and CSVs now measure the marginal cost of `features = ["widgets", "design"]`:

- `binary-size.csv` gains `design_on_bytes` and `design_diff_bytes`.
- `compile-time.csv` gains `build_widgets_design_ms` and `example_workbench_ms`.

Rows before v0.24 carry `N/A` in these columns. First real measurements
appear after the next CI tag.

### Documentation corrections

- RFC README `## Proposed` section corrected (was listing RFC-031 under
  Proposed despite it being in `done/`).
- Version snippets updated: `README.md` (`0.10` → `0.24`), `snora/src/lib.rs`
  (`0.18` → `0.24`), design docs (`0.19` → `0.24`).
- "35 design-track RFCs" corrected to "15" throughout.
- `api-freeze-review.md` header updated to v0.24.0; D-8 marked ✅ (v0.20).
- `release-process.md` stale `publish = false` note corrected.
- Script comments updated to reflect 9-field binary-size schema.
- SUMMARY.md: recipes moved under Snora Design section; API governance
  stays under Contributing.
- Relative links fixed in `pull_request_template.md` and migration guides.
- `composite_over` gains `debug_assert!(bg.is_opaque())`.
- `notice.rs` and `chip.rs` module comments document the `×` label
  accessible-label limitation explicitly.

## Upgrade steps

1. Change `snora = "0.23"` to `snora = "0.24"` in `Cargo.toml`.
2. If your code called `palette.roles()`, use direct field access instead.
3. Run `cargo check`. No other changes required.

## Versioning questions (per policy)

| Question | Answer |
|---|---|
| Does this break any public API? | Yes: `Palette::roles()` moved from `pub` to `#[cfg(test)] pub(crate)` |
| Does any type rename or move? | No |
| Does a default behavior change? | Yes: chip selected color (accessibility fix — visual appearance only) |
| Does a new public item require downstream action? | No |
