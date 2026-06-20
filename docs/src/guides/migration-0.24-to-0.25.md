# Migrating from 0.24 to 0.25

v0.25 is a **measurement methodology and documentation cleanup release**
addressing the second architect review of the design-system track. No
breaking changes to any public API.

## What changed

### Binary-size measurement replaced with size-probe crates

The previous binary-size measurement compared `snora-example-hello`
(widgets ON/OFF) against `snora-example-design-workbench` (design ON).
This was not valid: the diff included different application logic, not
just the feature cost.

v0.25 introduces three identical probe crates in `examples/size_probe_*/`:

| Crate | Features | Measures |
|---|---|---|
| `snora-size-probe-engine` | `--no-default-features` | Engine-only baseline |
| `snora-size-probe-widgets` | default (`widgets`) | Widgets baseline |
| `snora-size-probe-design` | `widgets` + `design` | Design baseline |

All three contain identical application code. The diffs now measure
the marginal cost of each feature in isolation:

- `widgets_diff_bytes = widgets_bytes − engine_bytes`
- `design_diff_bytes = design_bytes − widgets_bytes`

The `binary-size.csv` schema has been updated accordingly. Rows before
v0.25 carry `N/A` in all size columns.

### Build-cost measurement: `snora-design` now cleaned between runs

`measure-compile-time.sh` previously omitted `snora-design` from its
per-measurement clean, so the `build_widgets_design_ms` column could
reuse artifacts from the earlier `check --workspace --all-features` step.
Fixed: `snora-design` is now included in the clean list, and
`snora-example-design-workbench` is also cleaned before its measurement.

### Documentation corrections

- `rfcs/README.md`: RFC-031 row added to the Done table (was missing).
- `ROADMAP.md`: stale design-system section rewritten to accurate history.
- `docs/src/design/chips.md`: "tinted accent background" updated to
  "solid accent background with `accent_text` foreground".
- `examples/design_workbench/src/main.rs`: "v0.20" removed from banner.
- `examples/README.md`: invalid `--features design` flag removed from
  design-workbench run command.
- `crates/snora/Cargo.toml`: `design` feature comment updated (removed
  "roadmap Option B: v0.20" note).
- `CHANGELOG.md`: bottom reference links updated through v0.24.0.
- `docs/src/contributing/feature-gating-criteria.md`: current status
  updated to v0.25.
- `docs/src/contributing/api-freeze-review.md`: D-3/D-4 rationale
  clarified (five clean minors v0.20–v0.24; close after v0.25 freeze review).
- `scripts/README.md`: updated to document all three scripts and the
  new probe-based measurement approach.

## Upgrade steps

1. Change `snora = "0.24"` to `snora = "0.25"` in `Cargo.toml`.
2. Run `cargo check`. No other changes required.

## Versioning questions (per policy)

| Question | Answer |
|---|---|
| Does this break any public API? | No |
| Does any type rename or move? | No |
| Does a default behavior change? | No |
| Does a new public item require downstream action? | No |
