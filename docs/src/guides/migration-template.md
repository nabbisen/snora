# Migration x.y → x.z

> Replace this line with a one-sentence summary of the most important
> change.

## Who is affected

Describe which applications, feature combinations, or usage patterns are
affected. Be specific: "users who construct `AppLayout` via struct literal"
is more helpful than "all users."

If a change only affects a non-default feature combination, say so.

## What changed

Describe the surface change precisely. Include before/after type signatures
or field names where relevant.

## Why it changed

Explain the motivation. Link the RFC if one exists.

## Mechanical migration

Step-by-step instructions an automated tool or a developer can follow.
Prefer concrete code examples over prose.

```rust,ignore
// Before:
let layout = AppLayout { body, header: Some(h), .. };

// After:
let layout = AppLayout::new(body).header(h);
```

## Behavioral migration

Describe any behavioral changes that cannot be handled mechanically —
changes in rendering order, timing, or interaction semantics. If none,
write "No behavioral changes."

## Deprecated aliases and removal schedule

List any `#[deprecated]` aliases introduced alongside this change and
the planned minor release when they will be removed.

| Alias | Replaces | Removed in |
|---|---|---|
| `OldTypeName` | `NewTypeName` | v0.NN |

## Examples before/after

If examples in the repository were updated as part of this change, link
the relevant example and summarize what changed.

---

*Copy this template to `docs/src/guides/migration-X.Y-to-X.Z.md` and fill
it in. Then add a link to `docs/src/guides/migrations.md`.*
