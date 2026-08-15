# Migrating from 0.28 to 0.29

`snora-dialog-card` was attached to the wrong element since v0.27.0 — the
dialog's full-window centring container, not the actual card. v0.29
fixes this: the centring container is now `snora-dialog`, and
`snora-dialog-card` is re-pointed at the actual styled card.

## Who is affected

Only applications that assert on `snora-dialog-card` in tests — for
example via `iced_test::Simulator::find(Id::new("snora-dialog-card"))`,
or an equivalent selector against the identifier string directly.
Applications that do not query snora's rendered-surface identifiers are
unaffected: nothing about what renders, or how, changed.

## What changed

Before v0.29, `snora-dialog-card` was attached to the dialog's centring
`center(...)` wrapper — a full-window container present on **both**
`snora::render` and `snora::design::render` — and the actual styled card
(present only on the `design` path) carried no identifier at all.
Resolving `snora-dialog-card` always returned window-sized bounds, never
the card's.

As of v0.29:

| Identifier | Surface | Present on |
|---|---|---|
| `snora-dialog` | The centring container (what `snora-dialog-card` used to name) | Both paths, always |
| `snora-dialog-card` | The actual styled card — fill, border, radius | `design` path only |

See [reference/rendered-surface-identifiers.md](../reference/rendered-surface-identifiers.md#static-identifiers)
for the full table, and the [versioning policy](../contributing/versioning-policy.md#rendered-surface-identifiers)
worked example for the rationale.

## Why it changed

RFC-049. The identifier was silently pointing at the wrong element from
the moment RFC-039 introduced the card (v0.27.0): "present" every
render, on both paths, but never distinguishing the card from its
full-window wrapper — the exact failure mode a stable-identifier system
exists to prevent.

## Mechanical migration

| Was asserting | Now |
|---|---|
| Dialog presence or position (the centring container) | `snora-dialog` |
| The card's appearance or bounds (fill, border, padding, or that it is smaller than the window) | `snora-dialog-card` — **`design` path only**. On v0.28.0 this returned window bounds, which was wrong; it now returns the card's actual, smaller bounds. |

```rust,ignore
// Before (0.28): asserting dialog presence/position.
sim.find(Id::new("snora-dialog-card"))?;

// After (0.29): the same assertion now uses the renamed identifier.
sim.find(Id::new("snora-dialog"))?;
```

```rust,ignore
// Before (0.28): no identifier existed for the card itself — this
// assertion was not expressible.

// After (0.29), design path only: the card is now independently
// identifiable, and its bounds are genuinely the card's, not the window's.
sim.find(Id::new("snora-dialog-card"))?;
```

## Behavioral migration

No rendering or interaction change on either path. This is a
**re-pointing of an identifier's referent**, not a visual change: the
same elements render, in the same positions, with the same styling as
before. Only which element a given identifier string resolves to has
changed.

**The one risk this migration cannot catch mechanically:** a test
written against 0.28.0 that asserts on `snora-dialog-card` does not fail
on upgrade — it silently starts resolving the styled card instead of the
window-sized wrapper. If your test relied on the old (incorrect)
window-sized bounds, it may now fail for a different reason, or pass
with different values, without an upgrade-time signal pointing at this
change. Search your test suite for `snora-dialog-card` and re-read each
assertion against the table above before trusting it unchanged.

## Deprecated aliases and removal schedule

None. Rendered-surface identifiers are plain strings, not Rust symbols
— there is no `#[deprecated]` mechanism to attach to a string constant,
so the usual two-minor deprecation bridge does not apply here. The name
was re-pointed rather than retired, accepted for this one release only
because no known consumer had adopted 0.28.0 identifiers yet. See the
[versioning policy](../contributing/versioning-policy.md#rendered-surface-identifiers)
worked example.

## Examples before/after

No repository examples asserted on `snora-dialog-card`; none required
updating.

## Minimum supported Rust version

Unchanged: **1.88**, inherited from `iced` and `wgpu`.
