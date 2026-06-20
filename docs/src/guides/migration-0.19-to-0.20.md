# Migrating from 0.19 to 0.20

v0.20 is the **Snora Design activation release**. No breaking changes to any
existing public API. All existing applications compile unchanged.

## What changed

### `snora-design` is now published on crates.io

The `snora-design` crate was present in v0.19 with `publish = false`. It is
published as of v0.20 and can be used as a direct dependency:

```toml
snora-design = "0.20"   # token-only path, no iced dependency
```

### Binary-size and build-cost baselines updated

Both budget CSVs now have real CI data points (Gate 9 ✅ satisfied in
v0.19.1). See `docs/src/reference/binary-size-budget.md` and
`docs/src/reference/build-cost-budget.md`.

## Upgrade steps

1. Change `snora = "0.19"` to `snora = "0.20"` in `Cargo.toml`.
2. Run `cargo check`. No other changes required.

## Versioning questions (per policy)

| Question | Answer |
|---|---|
| Does this break any public API? | No |
| Does any type rename or move? | No |
| Does a default behavior change? | No |
| Does a new public item require downstream action? | No |
