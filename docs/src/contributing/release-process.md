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

## GitHub Actions workflows

Four workflows run automatically; they have distinct responsibilities:

| Workflow | File | Trigger | Responsibility |
|---|---|---|---|
| **CI** | `ci.yaml` | PR, push to `main` | Rust quality gate: `rust-quality`, `feature-matrix`, `design-isolation`, `docs` jobs. **No release merges while this is red.** |
| **Docs** | `docs.yaml` | Push to `main` | Build and deploy mdBook to GitHub Pages. |
| **Binary size** | `binary-size.yaml` | PR, push, tags | Measure stripped binary size; append a row to the CSV on release tags. |
| **Build cost** | `build-cost.yaml` | Push to `main`, tags | Measure compile time; append a row to the CSV on release tags. |

Do not confuse the **CI docs job** (PR gate) with the **Docs workflow**
(deployment). They run mdBook with the same `^0.5` locked version; keeping
them in sync is a release-process invariant.

## Release checklist

```text
[ ] Bump [workspace.package].version
[ ] If minor: bump snora-core / snora-widgets dep versions across crates
[ ] Move the [Unreleased] section in CHANGELOG.md to the new version,
    and reset [Unreleased] to "Nothing yet."
[ ] Update docs/guides/migration-X.Y-to-X.Z.md (minor only)
[ ] Update ROADMAP.md (move shipped items off; rewrite "Near-term"
    if priorities changed)
[ ] Move v0.NN RFCs from rfcs/proposed/ to rfcs/done/; update their
    Status fields and the rfcs/README.md index
[ ] Answer the four versioning-policy questions for any public API change
    (see docs/src/contributing/versioning-policy.md)
[ ] Confirm migration guide exists if any public API broke or renamed
[ ] Update user-facing version snippets in install.md and icons.md to the
    new version (snora = "X.Y" — iced version stays unchanged)
[ ] Re-run cargo metadata; confirm every crate reports new version
[ ] Confirm Cargo.lock is current and its diff (if any) is intentional —
    `git status --porcelain Cargo.lock`; if it shows a diff, review what
    moved and why before committing. A lockfile that drifts unreviewed is
    worse than none: it carries the implied assertion that someone looked.
[ ] cargo +<declared MSRV> check --workspace --all-features
    # must pass against the committed Cargo.lock; an MSRV not re-checked
    # at release time is a claim, not a fact
[ ] Confirm no resolved dependency declares a higher rust-version than the
    declared MSRV: cargo metadata --format-version 1 --all-features
[ ] cargo fmt --check
[ ] cargo check --workspace --all-features
[ ] cargo clippy --workspace --all-targets --all-features -- -D warnings
[ ] cargo test -p snora-core
[ ] cargo test -p snora-design
[ ] cargo test -p snora-widgets --features design
[ ] cargo test -p snora --lib --all-features
[ ] cargo test -p snora --test render_semantics   # CI hardware; may OOM locally
[ ] cargo check -p snora --no-default-features
[ ] mdbook build docs               # validates the book renders
[ ] mdbook test docs                # validates the doc-fence policy
[ ] All examples in examples/README.md acceptance matrix compile
    (covered by workspace check above; verify no example was removed)
[ ] Workbench manual QA checklist completed (docs/src/getting-started/06-workbench.md)
[ ] cargo package -p snora-core    --no-verify    # check .crate contents
[ ] cargo package -p snora-design  --no-verify    # check .crate contents
[ ] cargo package -p snora-widgets --no-verify    # check .crate contents
[ ] cargo package -p snora         --no-verify    # check .crate contents
[ ] Merge to main, then dispatch the `unpinned-build` workflow on main and
    confirm a green run BEFORE tagging.
    # `workflow_dispatch` only works for workflows already on the default
    # branch, so this cannot be verified from a feature branch. A workflow
    # that has never executed is exactly the failure RFC-041 was raised to
    # fix — do not skip this.
[ ] git commit, git tag X.Y.Z, git push --tags
    # tags carry no `v` prefix, matching Rust crate convention
[ ] Confirm CI workflow green on the tag commit (all four jobs:
    rust-quality, feature-matrix, design-isolation, docs)
[ ] After tag push: confirm a NEW ROW EXISTS for this version in
    docs/src/reference/binary-size-budget/binary-size.csv on main —
    `git show main:docs/src/reference/binary-size-budget/binary-size.csv | tail -1`
    and check its `version` column equals the tag just pushed. This is the
    falsifiable check; "the workflow run is green" is not, because a
    workflow that never triggers reports nothing rather than failing (the
    exact failure mode RFC-041 exists to fix — the tag-pattern mismatch
    that caused it went unnoticed for six releases specifically because
    only the run's status, not the row's existence, was ever checked). If
    no new row exists, treat it as a release blocker: check the Actions
    tab for whether the `binary-size` workflow triggered at all on the
    tag, not just whether a triggered run went green. If the diff column
    exceeds 150 KB, follow up per feature-gating-criteria.md indicator 2.
[ ] After tag push: confirm a NEW ROW EXISTS for this version in
    docs/src/reference/build-cost-budget/compile-time.csv on main —
    `git show main:docs/src/reference/build-cost-budget/compile-time.csv | tail -1`
    and check its `version` column equals the tag just pushed. Same
    failure-mode note as above: verify the row, not just workflow status.
    If no new row exists, treat it as a release blocker. If
    build_widgets_ms exceeds 30 000, follow up per
    feature-gating-criteria.md indicator 1.
```

### Why `--no-verify`

`cargo package --no-verify` skips the dependency-resolution check
that would otherwise demand the sibling crate be on crates.io
*already*. We use it to inspect the `.crate` archive locally before
the actual `cargo publish` (which has its own verification step
that is order-aware).

### Publish order

Strictly bottom-up along the dependency graph:

1. `snora-core` (no internal deps).
2. `snora-design` (no internal deps; no iced dependency; published from v0.20 onward).
3. `snora-widgets` (depends on `snora-core`; optionally on `snora-design`).
4. `snora` (depends on `snora-core`; optionally on `snora-widgets` and
   `snora-design`).

Each crate's `Cargo.toml` uses both `path = "..."` and
`version = "..."` for inter-crate references, so cargo's local
build does not require crates.io, and crates.io's verification
finds the just-published sibling at the matching version.

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
