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

This change propagates to every member crate's own version via
`version.workspace = true`. No per-crate *package*-version edit is
needed for that part — internal dependencies between workspace crates
declare `{ workspace = true }` too (e.g. `crates/snora/Cargo.toml`'s
`snora-core = { workspace = true }`) rather than a hand-pinned path and
version.

That covers a **patch** bump completely: the one-edit
`[workspace.package].version` change is the whole release. A **minor**
bump is not one edit, though — `[workspace.dependencies]` at the
workspace root carries its own five `version = "0.38"` pins (one per
internal crate), centralised rather than absent, and a minor must move
all of them, plus three example manifests that can't use workspace
inheritance at all. See the release checklist below for the specific
files and the exact failure mode of missing one.

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
[ ] If minor: bump the hand-pinned `snora` version in
    examples/size_probe_engine/Cargo.toml AND
    examples/responsive_body/Cargo.toml AND
    examples/size_probe_design_engine/Cargo.toml. All three use an
    explicit `default-features = false` path dependency instead of
    `snora = { workspace = true }` (workspace inheritance cannot
    override `default-features`), so none follows the workspace
    version table and each must be hand-edited on every minor bump —
    missing any fails EVERY `cargo` command in the workspace with
    "failed to select a version for the requirement `snora = "^0.NN"`"
    (this has already happened once, on the 0.26.0 bump). Grep for the
    old minor across all `Cargo.toml` files before assuming the
    workspace table alone is sufficient.
[ ] Move the [Unreleased] section in CHANGELOG.md to the new version,
    and reset [Unreleased] to "Nothing yet."
[ ] Update docs/guides/migration-X.Y-to-X.Z.md (minor only)
[ ] For any capability that arrived, left, or any governance/policy
    decision that answers a question a consumer would ask: apply
    feature-gating-criteria.md § "Documentation scope when a capability
    arrives, leaves, a standing answer is invisible, or a claim is
    withdrawn" — grep the default-path docs for the claim it
    contradicts, or add a consumer-facing statement. Five misses reached
    this checklist before the rule did; this line is why it won't be a
    sixth.
[ ] Did this release withdraw, narrow, or correct anything we
    previously told consumers (RFC-067)? There is no grep for this —
    answer it yourself, since you are the person writing the note. If
    yes, the note names **what to re-check**, not only what changed —
    the same distinction a rendered-appearance change's "re-check any
    screenshot tests…" line already makes, now required of a withdrawn
    or narrowed *claim* too. A correction that does not say what to do
    about it reaches nobody who already acted on the old claim — five
    instances across four consumers (`feature-gating-criteria.md`'s
    documentation-scope table) is why this line exists.
[ ] Re-evaluate feature-gating-criteria.md's "Current status" table
    (RFC-062) — the table itself says to do this and nothing pointed at
    it for ten minors, which is why it went stale in the first place.
    Re-derive indicator 2 (`widgets_diff_bytes`) against the latest
    binary-size.csv row; re-check indicators 3 and 4 against the current
    manifests rather than inheriting the prior row's answer; note any
    new field requests for indicator 5. Also re-run the accessibility-
    tree trigger's check —
    `cargo tree -p snora --all-features | grep -i accesskit` — and update
    design-decisions.md's register with the result and this release's
    date if it changed.
[ ] Read the new `design_overhead_ratio` row against
    build-cost-budget.md's watch points. Gate 9b closed at 0.37.0 on four
    rows, so this is ongoing monitoring, not a closure check — and it is
    monitoring with a stated floor: the ratio moved -4.44% across a release
    that changed no executable code, so treat a move under ~10% as noise
    and investigate above it. The absolute millisecond columns are raw
    record, never a trend (RFC-050).
[ ] Update ROADMAP.md (move shipped items off; rewrite "Near-term"
    if priorities changed)
[ ] Move v0.NN RFCs from rfcs/accepted/ to rfcs/done/ (five-folder
    variant — an accepted RFC lives in accepted/, not proposed/); update their
    Status fields and the rfcs/README.md index
[ ] Answer the four versioning-policy questions for any public API change
    (see docs/src/contributing/versioning-policy.md)
[ ] Confirm migration guide exists if any public API broke or renamed
[ ] Run scripts/check-version-snippets.sh and fix every snippet it names
    (RFC-074 — derives the expected minor from Cargo.toml itself, so this
    replaces enumerating files by hand; iced version stays unchanged)
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
[ ] cargo fmt --all --check
    # Note --all: a bare `cargo fmt --check` misses the example crates,
    # where most drift accumulates. CI enforces this on every PR and push
    # as of 0.28.1; before that nothing ran it, and it had silently stopped
    # passing on a clean tree for several releases while still sitting in
    # this checklist. Do not tick a gate you have not seen pass.
[ ] cargo check --workspace --all-features
[ ] cargo clippy --workspace --all-targets --all-features -- -D warnings
[ ] cargo test -p snora-core
[ ] cargo test -p snora-design
[ ] cargo test -p snora-widgets --features design
[ ] cargo test -p snora --lib --all-features
[ ] cargo test -p snora --test render_semantics   # CI hardware; may OOM locally
[ ] cargo test --workspace --all-features | grep -A2 "Doc-tests"
    # Confirm the passed/ignored counts per crate match
    # documentation-test-policy.md's "Current counts" table; update that
    # table if a doctest was added, removed, promoted, or newly ignored.
    # Every fence left at `ignore` must carry a stated reason (RFC-064) —
    # `grep -B1 '```rust,ignore\|```ignore' crates/ -r --include="*.rs"`
    # to spot-check one wasn't added without one.
[ ] cargo check -p snora --no-default-features
[ ] mdbook build docs               # validates the book renders
[ ] mdbook test docs                # validates the doc-fence policy
[ ] All examples in examples/README.md acceptance matrix compile
    (covered by workspace check above; verify no example was removed)
[ ] Workbench manual QA checklist completed (docs/src/getting-started/06-workbench.md)
[ ] cargo package --workspace
    # Inspects all five .crate archives (snora-core, snora-design,
    # snora-style, snora-widgets, snora — snora-style added in RFC-055,
    # 0.32.0). Examples are ALSO packaged (each emits "manifest has no
    # description"); `publish = false` stops them being *published*, not
    # packaged, so no exclusion flag is needed but the output is noisier
    # than "five archives". Corrected at 0.38.0 — the previous wording
    # said they were skipped, and they are not.
    # Do NOT package the five crates individually (see "Publishing").
[ ] Merge to main, then dispatch the `unpinned-build` workflow on main and
    confirm a green run BEFORE tagging.
    # `workflow_dispatch` only works for workflows already on the default
    # branch, so this cannot be verified from a feature branch. A workflow
    # that has never executed is exactly the failure RFC-041 was raised to
    # fix — do not skip this.
[ ] git commit, THEN pull --rebase, THEN tag — in that order
    git tag -s X.Y.Z -m "X.Y.Z"
    # -s is REQUIRED: this repo sets `tag.gpgsign true`, so a bare
    # `git tag X.Y.Z` fails with "fatal: no tag message?" — which reads
    # like a message problem and is actually a signing one. Signed tags
    # are annotated, so -m is mandatory too.
    # Tagging before rebasing leaves the tag on a commit the rebase
    # orphans: `git merge-base --is-ancestor X.Y.Z^{commit} main` fails,
    # and the release tag is not in main's history. The measurement bots
    # append to main between your commit and your push, so a rebase is
    # NORMAL on this repo, not an exception.
[ ] Verify the tag is on main BEFORE pushing it:
    git merge-base --is-ancestor X.Y.Z^{commit} main && echo on-main
    # If it is not, delete the local tag and re-tag. If you already pushed
    # it, cancel any tag-triggered measurement runs first — otherwise
    # re-pushing the tag appends a SECOND row for the same version.
[ ] git push origin main && git push origin X.Y.Z
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
[ ] CONTENT-check that same binary-size row (RFC-044 — a row existing says
    nothing about whether its fields are right; this is the check that
    would have caught RFC-043's `widgets_diff_bytes = 0` at release time
    instead of a release later):
    `git show main:docs/src/reference/binary-size-budget/binary-size.csv | tail -1 | cut -d, -f8`
    must print `ubuntu-latest` — field 8, `runner_os`. If it prints
    `Linux`, the `SNORA_RUNNER_OS` override did not reach the script;
    treat it as a release blocker and investigate before publishing, do
    not just note it and move on.
    `... | cut -d, -f4` (`widgets_diff_bytes`) must be **non-zero**. A `0`
    means the probes are byte-identical again and are measuring nothing —
    treat it as a release blocker.
[ ] After tag push: confirm a NEW ROW EXISTS for this version in
    docs/src/reference/build-cost-budget/compile-time.csv on main —
    `git show main:docs/src/reference/build-cost-budget/compile-time.csv | tail -1`
    and check its `version` column equals the tag just pushed. Same
    failure-mode note as above: verify the row, not just workflow status.
    If no new row exists, treat it as a release blocker. If
    build_widgets_ms exceeds 30 000, follow up per
    feature-gating-criteria.md indicator 1.
[ ] CONTENT-check that same compile-time row (RFC-044, same rationale as
    the binary-size content-check above):
    `git show main:docs/src/reference/build-cost-budget/compile-time.csv | tail -1 | cut -d, -f9`
    must print `ubuntu-latest` — field 9, `runner_os`. This field number
    does NOT move when a column is added (RFC-050 appends
    `design_overhead_ratio` as the LAST field, field 11, specifically so
    this check and every other positional read of an existing field
    keeps working unchanged — see build-cost-budget.md's append-only
    column note). If it prints `Linux`, treat it as a release blocker,
    same as above.
    `... | cut -d, -f2` (`check_workspace_ms`) must be **at least 10 000**
    (plausibly cold — tens of seconds, not milliseconds). A value in the
    hundreds or low thousands means a dependency cache was restored and
    the "cold" build is warm — the exact defect RFC-043 fixed once
    already; treat it as a release blocker and check that
    `build-cost.yaml` still has no `Swatinem/rust-cache` step.
[ ] cargo publish --workspace   — FROM A CLEAN TREE AT THE TAG
    # cargo packages the WORKING DIRECTORY, not the tagged commit. If other
    # work is in flight, publish from a throwaway worktree:
    #   git worktree add --detach /tmp/pub X.Y.Z && cd /tmp/pub && cargo publish --workspace
    # Cargo refuses a dirty tree by default. NEVER pass --allow-dirty.
    # ONE command — cargo resolves member order itself. Do not publish the
    # five crates individually; an interrupted per-crate sequence leaves a
    # public tag with `snora` itself missing from crates.io, and anyone
    # depending on the new minor gets a resolution failure until it is
    # finished. See "Publishing" below.
[ ] Confirm all five crates report the new version on crates.io
```

### Publishing

```bash
cargo publish --workspace
```

**One command. Do not publish the five crates individually.**

**Publish from a clean tree at the tag** — not from a working directory
with other work in flight. `cargo publish` packages the **working
directory**, not the tagged commit, so uncommitted work from a later
milestone would be uploaded inside the release. Cargo refuses a dirty tree
by default, which is the guard; **never pass `--allow-dirty` to get past
it.**

If other work is in flight, publish from a throwaway worktree at the tag:

```bash
git worktree add --detach /tmp/publish-X.Y.Z X.Y.Z
cd /tmp/publish-X.Y.Z && cargo publish --workspace
```

This is not hypothetical: 0.27.1 was cut while the 0.28.0 work sat
uncommitted, and cargo's dirty-tree refusal is what stopped a docs-only
patch from shipping two unreleased features' source.

Cargo computes the dependency order from the manifests
(`snora-core` and `snora-design` have no internal dependencies;
`snora-style` depends on `snora-design`; `snora-widgets` depends on
`snora-core`, `snora-design` and `snora-style`; `snora` depends on all
four) and waits
for each to become available on crates.io before publishing the next. The
order cannot drift from the manifests, because nothing restates it.

#### Why not one `cargo publish` per crate

That was this project's process until v0.27.0, and it predates
`cargo publish --workspace`. It had two costs:

- **A hand-maintained order.** A restated dependency order is a second
  source of truth that can disagree with the manifests.
- **Packaging appeared broken on every minor bump.** Once
  `[workspace.dependencies]` moves from `0.26` to `0.27`,
  `cargo package -p snora-widgets` fails with

  ```text
  error: failed to select a version for the requirement `snora-core = "^0.27"`
  ```

  because that version is not on crates.io yet. This is not a fault, and
  earlier revisions of this page carried a long explanation of why it was
  expected. `cargo publish --workspace` removes the condition rather than
  explaining it.

A half-published release is the failure this avoids: if the sequence is
interrupted partway, the tag is public while `snora` itself is missing, and
anyone depending on the new minor gets a resolution failure until it is
finished.

#### `--no-verify`

`cargo package --no-verify` skips the *build* verification step; it does
**not** skip dependency resolution. It is occasionally useful for
inspecting a `.crate` archive's contents in isolation, but it is not part
of the normal release path.

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
