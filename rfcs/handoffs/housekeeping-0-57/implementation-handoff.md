# Implementation handoff — 0.57: the crate rename, the two removals, and actionlint's cache

**Ruled by the owner, 2026-11-08:**
- the rename and its final notice release (Q-1);
- the removal rule (Q-2);
- the CI fix (Q-4);
- each as reviewed against the fundamental philosophy, with the refinements recorded
  here.

**Release target.** 0.57.0. **This lands first in 0.57,** so that RFC-116 R-2's work goes
into the renamed crate.
**One branch, three commits (A, B, C), one review.** Land it as `main` requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## A. `snora-control` becomes `snora-interaction`

**Why:**
- beside `snora-widgets`, "control" reads as another word for widget;
- the crate now holds far more than the control: `Inert`, `OnEscape`, `Keyed`, `Group`,
  `sequential_focus`, `Glyph`, `EmBox` and `Underlined`.

What they share is *how the reader operates snora*. The widgets are what is drawn; the
interaction crate is how it is operated.

1. **Move the crate.**
   - `crates/snora-control/` becomes `crates/snora-interaction/`, with package name
     `snora-interaction` and library `snora_interaction`. Every item moves unchanged.
   - **The type `Control` keeps its name:** it is a control.
   - Update every use in `snora` and `snora-widgets`, and the workspace dependency entry.
2. **The final `snora-control` release: a notice, nothing else.** A new, minimal
   `crates/snora-control/`, in the workspace for 0.57.0 only:
   - `Cargo.toml`: version from the workspace, **no dependencies**, a `description` that
     begins "Deprecated: renamed to snora-interaction.", and
     `[badges] maintenance = { status = "deprecated" }`;
   - `src/lib.rs`: **no items**, only a crate doc stating that the crate is renamed to
     `snora-interaction` from 0.57.0, that applications never needed it (they use
     `snora`), and that 0.55.0 and 0.56.0 stay published and are never yanked, so
     existing lockfiles keep working;
   - a short `README.md` saying the same;
   - **nothing depends on it.** `release.yaml` publishes it with the rest: its trusted
     publisher already exists.
   - It **leaves the workspace in 0.58**, and the architect records that.
3. **Everywhere that lists crates:**
   - `Cargo.toml` (members, `[workspace.dependencies]`, `[package.metadata.docs.rs]`);
   - `.github/workflows/ci.yaml` (the feature-matrix rows, and the wgpu job's
     `snora-control` step, which becomes `snora-interaction`);
   - `scripts/check-literal-sizes.sh`, `scripts/measure-compile-time.sh` (its clean lists),
     and any other script `grep -rn snora-control scripts .github` finds.

   **RFC-112's opt-out probe must stay clean.**
4. **Not renamed:**
   - past RFCs, the CHANGELOG's released entries, and the 0.54 → 0.55 guide (history);
   - comments that name RFCs.

   The book is the architect's. Tell the architect which pages still say `snora-control` after your
   grep.
5. **Check:**
   - `cargo tree -p snora | grep snora-` shows `snora-interaction` and no `snora-control`;
   - `cargo package --workspace` packages seven crates, the notice crate included, and the
     notice crate's package contains only `Cargo.toml`, `src/lib.rs` and `README.md`.

## B. Remove the 0.54 deprecations, and name every removal version

**The rule (ruled):**

> **A deprecated item is removed in the third minor after the one that deprecated it.**

Items deprecated in 0.54 go in 0.57; items deprecated in 0.56 go in 0.59.

1. **Remove** `snora::toast::sweep_expired` (superseded by `Holds::sweep`) and
   `ToastLifetime::DEFAULT` (superseded by `ToastLifetime::reading_time`). Their tests go
   with them, and any example or doc that still uses them moves to the replacement.
2. **Name the removal version** in every `#[deprecated]` note still in the tree: the
   three from 0.56 (`on_toast_hover` and its field, `ToastHover`, `Holds::hover`).
   Each note says "removed in 0.59.0". **A rule is only predictable if the reader is told
   the date.**
3. **CHANGELOG `[Unreleased]`:**
   - **Removed:** the two items, each with its one-line replacement;
   - **Changed:** "the deprecation notes of `on_toast_hover`, `ToastHover` and
     `Holds::hover` now name their removal version, 0.59.0".

**Check:** `grep -rn "sweep_expired\|ToastLifetime::DEFAULT" crates examples` finds
nothing outside history. Every `#[deprecated]` note in `crates/` contains "removed in".

## C. Workflow lint survives a download outage, and never runs an unverified binary

Workflow lint went red twice in two days on GitHub's release server answering 504, with
nothing wrong in the workflows.

**The gap a plain cache would open:** `scripts/check-workflows.sh` verifies the SHA-256
of the **archive** it downloads (`:108`). A cached, already-unpacked binary is then run
with no check, so caching that binary in CI would run an unverified program.

1. **Cache the archive, not the binary.** In CI, cache the downloaded
   `actionlint_<version>_<platform>.tar.gz` (`actions/cache`, keyed on the pinned version
   and platform).
2. **Verify the pinned SHA-256 on every run, cached or not, before unpacking.** The
   script keeps the archive and checks it each time. A mismatch deletes it and refuses,
   as now.
3. **Retry the download a few times on a server error** (`curl --retry 4
   --retry-all-errors`, or equivalent) before giving up.
4. **Unchanged:** when the tool truly cannot be obtained, the check still says
   "REFUSED … unvalidated, not clean" and exits 1. Never report a tree clean without
   having checked it.
5. **Breakage rows:**
   - corrupt the cached archive: the run refuses, and does not use it;
   - point the URL at a 5xx: the retries are visible, then it refuses.

   Named `cp` backups, ending with `sha256sum -c`.

**Locally,** an existing `target/actionlint-cache/<version>/actionlint` from earlier runs
is an unverified binary too. After this change the script ignores it, and works from the
verified archive.

## Acceptance criteria

1. A, B and C as above, each with its check.
2. All of `ci.yaml`'s Rust quality gate commands, the Docs job's commands, the feature
   matrix, the opt-out probe, both rendered jobs and parity. Then the branch CI.
3. **Not your part:** the first publish of `snora-interaction`. At the 0.57.0 cut, the
   architect publishes it from the owner's machine before the tag, and the owner adds its
   trusted publisher (`release-process.md`, "A crate's first version").
