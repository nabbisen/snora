# Release process

The workspace uses inheritance for the version number, so a release
is fundamentally one edit. The supporting steps make sure the
release is consistent across crates, examples, and the published
artifact.

## Versioning policy

Snora is pre-1.0 and follows the conventions of pre-1.0 SemVer:

- **Patch (`0.x.y` → `0.x.(y+1)`)** — bug fixes only. No API change,
  no behavior change visible to a typical app.
- **Minor (`0.x` → `0.(x+1)`)** — feature additions, API additions,
  and *small* breaking changes when justified. The `0.4 → 0.5`
  toast-default change is an example.
- **Major (`0.x` → `0.(x+1)`)** does not exist; a true major bump
  will be `1.0` with a stability pledge.

Inside a workspace cycle, all member crates share the same version.
This is enforced by `[workspace.package].version` inheritance.

## One-edit release

```toml
# Cargo.toml at workspace root
[workspace.package]
version = "0.5.1"        # bump
```

This change propagates to every member crate via
`version.workspace = true`. No per-crate edit is needed.

If `snora-core`'s on-disk version changes minor digits, also bump
`snora`'s declared dep:

```toml
# crates/snora/Cargo.toml
[dependencies]
snora-core = { path = "../snora-core", version = "0.5" }
```

The trailing `"0.5"` is a caret range (`^0.5`), so all `0.5.*`
patch releases are accepted. Bump it only on a minor.

## Release checklist

```text
[ ] Bump [workspace.package].version
[ ] If minor: bump snora-core dep version in crates/snora/Cargo.toml
[ ] Update docs/guides/migration-X.Y-to-X.Z.md (minor only)
[ ] Re-run cargo metadata; confirm every crate reports new version
[ ] cargo check --workspace --all-features
[ ] cargo clippy --workspace --all-targets --all-features -- -D warnings
[ ] cargo test --workspace --all-features
[ ] cargo package -p snora-core --no-verify     # check .crate contents
[ ] cargo package -p snora      --no-verify     # check .crate contents
[ ] git commit, git tag vX.Y.Z, git push --tags
[ ] cargo publish -p snora-core
[ ] cargo publish -p snora
```

### Why `--no-verify`

`cargo package --no-verify` skips the dependency-resolution check
that would otherwise demand the sibling crate be on crates.io
*already*. We use it to inspect the `.crate` archive locally before
the actual `cargo publish` (which has its own verification step
that is order-aware).

### Publish order

`snora-core` first, then `snora`. The `snora` crate's `Cargo.toml`
has both `path = "../snora-core"` and `version = "..."` on its
dependency, so cargo accepts the build locally and on crates.io
finds the just-published `snora-core` of the matching version.

## Tarball releases (if used)

For local release artifacts shipped outside crates.io, name them
with a version suffix:

```text
snora-X.Y.Z.tar.gz
```

This was the convention adopted from 0.4.2 onward.

## Examples are not published

The `examples/*` crates set `publish = false` in their
`Cargo.toml`. They are part of the workspace for `cargo check` and
`cargo run -p` convenience but never go to crates.io.
