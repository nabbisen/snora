# Migrating from 0.21 to 0.22

v0.22 is a **code quality and documentation audit release**. No breaking
changes. All existing applications compile unchanged.

## What changed

### Chip code quality

`chip::removable` no longer allocates a redundant `style_fn_rm` variable
(it was a copy of `style_fn`). The chip hover/pressed states now use a
private `darken` helper instead of inline channel arithmetic. No behaviour
change.

### Test coverage expanded

- `snora_widgets::design::chip` — 4 new unit tests covering all `button::Status`
  variants across all four token presets and the `darken` helper.
- `snora_widgets::design::notice` — compile-time tests covering all `Tone`
  variants, all preset tokens, and all builder combinations.
- `snora_widgets::design::progress` — compile-time tests plus a `value_clamps_within_range`
  runtime test.

### New documentation pages

Three new pages in `docs/src/design/`:

- [Notices](../design/notices.md) — usage, tone table, visibility pattern, accessibility.
- [Chips](../design/chips.md) — filter and removable variants, selection state, accessibility.
- [Progress](../design/progress.md) — value, tone, layout variants, indeterminate limitation.

`v021-primitives.md` updated from planning document to implementation reference.

### Stale version references removed

Internal doc comments that said "Cards in v0.20 are non-interactive" updated
to remove the version reference.

## Upgrade steps

1. Change `snora = "0.21"` to `snora = "0.22"` in `Cargo.toml`.
2. Run `cargo check`. No other changes required.

## Versioning questions (per policy)

| Question | Answer |
|---|---|
| Does this break any public API? | No |
| Does any type rename or move? | No |
| Does a default behavior change? | No |
| Does a new public item require downstream action? | No |
