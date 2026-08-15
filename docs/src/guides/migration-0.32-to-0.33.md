# Migrating from 0.32 to 0.33

`snora_widgets::design::style` and `snora_widgets::design::theme` — the
compatibility re-exports RFC-055 kept while the iced style bridge moved
to `snora-style` — are removed. `snora::design::style::*` and
`snora::design::theme` (the documented consumer route, through the
`snora` facade) are unaffected.

## Who is affected

Only applications depending on `snora-widgets` directly and importing
`snora_widgets::design::style::*` or `snora_widgets::design::theme`. No
current documentation directs anyone to depend on `snora-widgets`
directly — `architecture.md` describes it as an optional crate consumed
through `snora`'s `widgets` feature — so the known affected population
is zero. The one historical exception is
`docs/src/guides/migration-0.5-to-0.6.md:142`, which shows
`snora-widgets = "0.6"` as a direct dependency example; anyone who
followed that nine-minor-old instruction and reached the style layer
through it is who this guide is for.

If you use `snora::design::*` (through the `snora` crate, the
documented path), **you are not affected** — nothing changes for you.

## What changed

| Removed path | Replacement |
|---|---|
| `snora_widgets::design::style::{color, button, container, text, progress}` | `snora_style::{color, button, container, text, progress}` |
| `snora_widgets::design::theme` | `snora_style::theme` |

Both are now direct paths into the `snora-style` crate — the same crate
`snora_widgets::design::style`/`::theme` already re-exported, so the
underlying functions are identical; only the import path changes.

## Why it changed

RFC-055 (0.32.0) relocated the iced style bridge from
`snora-widgets/src/design/style/` (and `theme.rs`) into a new peer
crate, `snora-style`, and kept the old paths as compatibility
re-exports so nothing broke mid-move. Its stated precondition for
removing them — `snora::design::style` and `::theme` pointing at
`snora-style` directly — was met as of that same release. RFC-056
removes the re-exports rather than deprecating them first: in this
workspace, `#[deprecated]` on a bare `pub use` re-export emits no
warning at all, so a deprecation cycle would have required wrapping the
re-export in a local module purely to carry the attribute — machinery
in service of a warning nobody was going to see, since the audience for
it was already hypothetical. See
[RFC-056](https://github.com/nabbisen/snora/blob/main/rfcs/proposed/056-remove-the-style-shims.md)
for the full reasoning.

## Mechanical migration

```rust,ignore
// Before:
use snora_widgets::design::style;
let card = style::container::card_raised(&tokens);

// After:
use snora_style as style;   // or: use snora_style;
let card = style::container::card_raised(&tokens);
```

```rust,ignore
// Before:
use snora_widgets::design::theme;
let iced_theme = theme::theme(&tokens);

// After:
use snora_style::theme;
let iced_theme = theme::theme(&tokens);
```

If you were depending on `snora-widgets` for the style bridge alone
(not for any prefab widget), add `snora-style` as a direct dependency
and drop `snora-widgets` entirely.

## Behavioral migration

None. The functions themselves did not move again in this release —
they moved once, to `snora-style`, in 0.32.0. This release only removes
the path that pointed at them from inside `snora-widgets`.

## Deprecated aliases and removal schedule

None. Per RFC-056, removal was chosen instead of a deprecation cycle —
see "Why it changed" above.

## Examples before/after

No repository example imported `snora_widgets::design::style` or
`snora_widgets::design::theme` directly; none required updating.
