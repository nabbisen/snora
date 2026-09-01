# Troubleshooting

An index of errors this project has already diagnosed, so you don't have
to re-diagnose them. Each entry points at where the full explanation
already lives — nothing here is restated, only located (F-25, RFC-089).

## `E0027: pattern does not mention field ...`

You matched exhaustively on `Palette` or `Typography` and the compiler
refused because a role or field wasn't named. This is intentional: it's
the mechanism that makes contrast/typography coverage a compile error
instead of a maintained list. See
[Stability § You do not need to re-check our contrast](../design/stability.md#you-do-not-need-to-re-check-our-contrast).

## `E0638: '..' required with struct marked as non-exhaustive`

You tried to copy snora's own exhaustive-match-as-completeness-check
pattern onto `Palette` from outside this crate. `#[non_exhaustive]` only
permits exhaustive destructuring *inside* the defining crate, so the
compiler demands `..` — which silently defeats the mechanism. This is a
language boundary, not a bug on your side; apply the pattern to your own
palette type instead. See
[Stability § You do not need to re-check our contrast](../design/stability.md#you-do-not-need-to-re-check-our-contrast).

## `E0433`, specifically when combining `widgets` and `design` features

Compiling with both `widgets` and `design` enabled fails to resolve
`snora-widgets`'s own `design` submodule. This means a workspace manifest
dropped the `?` from `"snora-widgets?/design"` in a `design` feature
declaration (or removed the entry outright) — the `?` is required so the
feature activates conditionally only when `snora-widgets` is already
present. See `rfcs/done/055-extract-the-style-bridge.md` § "the fix" for
the full explanation (RFC-055) — a source-tree path, not a book link:
this page's build doesn't include `rfcs/`.

## `error: failed to select a version for the requirement 'snora-core = "^0.NN"'`

Seen when packaging or checking the workspace right after a minor version
bump. Not a fault — it means `[workspace.dependencies]`'s internal-crate
pins were bumped but the matching version isn't on crates.io yet, which
is expected mid-release, before `cargo publish --workspace` finishes. See
[Release process § Why not one `cargo publish` per crate](../contributing/release-process.md#why-not-one-cargo-publish-per-crate).

## `fatal: no tag message?`

Reads like a missing `-m`, but the real cause is usually a missing `-s`:
this repository sets `tag.gpgsign true`, so `git tag X.Y.Z` (unsigned)
fails this way. Use `git tag -s X.Y.Z -m "X.Y.Z"`. See the [release
checklist](../contributing/release-process.md#release-checklist).
