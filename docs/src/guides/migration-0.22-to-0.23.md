# Migrating from 0.22 to 0.23

v0.23 is a **recipes and governance release**. No code changes. All
existing applications compile unchanged.

## What changed

### Four initial design recipes published

`docs/src/design/recipes/` now contains four copy-paste patterns:

- [Empty state](../design/recipes/empty-state.md)
- [Background task card](../design/recipes/background-task.md)
- [Friendly error recovery notice](../design/recipes/error-recovery.md)
- [Result card](../design/recipes/result-card.md)

Each recipe follows the nine-section format defined in
[Recipes and dogfood process](../contributing/recipes.md).

### All design-track RFCs complete

RFC-033 (Recipes and Dogfood Process) and RFC-034 (Promotion, Stabilization,
and API Governance) are both closed. All 35 design-track RFCs (RFC-020
through RFC-034) are now in `rfcs/done/`. Future design work is governed
by `docs/src/contributing/api-governance.md`.

## Upgrade steps

1. Change `snora = "0.22"` to `snora = "0.23"` in `Cargo.toml`.
2. Run `cargo check`. No other changes required.

## Versioning questions (per policy)

| Question | Answer |
|---|---|
| Does this break any public API? | No |
| Does any type rename or move? | No |
| Does a default behavior change? | No |
| Does a new public item require downstream action? | No |
