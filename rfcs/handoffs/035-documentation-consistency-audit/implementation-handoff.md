# Developer Handoff — RFC-035 documentation consistency audit

**Governing RFC.** [RFC-035](../../proposed/035-documentation-consistency-audit.md)
**Status.** Inherited from RFC-035 (Proposed).
**Implementation units.** One. This is a single reviewable change.

---

## 1. Task title

Correct in-tree documentation to describe the shipped four-crate
architecture, the actual contributor procedures, and the actual release
gate suite.

## 2. Purpose

Make the documents that the project's own source-of-truth hierarchy calls
authoritative actually authoritative. Today `contributing/architecture.md`
would send a contributor to the wrong crate, and neither architecture page
mentions `snora-design` at all.

## 3. Background

`snora-design` shipped in v0.19 and was published in v0.20. Five minors
have followed. Code, feature graph, CI (`design-isolation` job), and the
release process all reflect four crates. The architecture prose never
caught up. RFC-035 §Findings enumerates every affected location with line
references; this handoff is the execution plan for them.

Read before starting:

- `rfcs/proposed/035-documentation-consistency-audit.md` (the governing RFC)
- `docs/src/contributing/documentation-test-policy.md` (fence rules)
- `.git-exclude/rules/project-instructions-rust-gui.md` (English-only,
  file-splitting, `cargo fmt` ordering)

## 4. Applicable requirements

- **M-3** CHANGELOG and ROADMAP kept current (F-8, blocked)
- **NF-1** `snora-core` / `snora-design` iced-free — must be *stated
  correctly*, not merely true in the manifests
- **NF-7** all doc fences classified; `mdbook test` in CI
- **DEC-01** four-crate split
- Requirements §1.7 "Preserve the *why*"

## 5. Change scope

Exactly these files:

| File | Findings |
|---|---|
| `README.md` | F-1 |
| `docs/src/reference/architecture.md` | F-1 |
| `docs/src/contributing/architecture.md` | F-1, F-2, F-6 |
| `docs/src/contributing/alternate-engine-boundary.md` | F-1 (cross-ref text only) |
| `docs/src/contributing/design-decisions.md` | F-4 |
| `docs/src/contributing/release-process.md` | F-3 |
| `docs/src/contributing/api-freeze-review.md` | F-5 |
| `crates/snora/src/lib.rs` | F-1 — **module doc comment only** |
| `rfcs/README.md` | F-7 |
| `.gitignore` | F-6 — unblocked; owner confirmed the lockfile stays committed |
| `CHANGELOG.md` | F-8 — unblocked; owner confirmed 0.25.2 was published |
| `docs/src/reference/binary-size-budget.md` | F-8 follow-on note (Step 9) |

## 6. Explicit non-change scope

Do **not** touch:

- Any `pub` item, signature, feature flag, or `Cargo.toml` anywhere.
- Any code outside the `//!` module doc comment at the top of
  `crates/snora/src/lib.rs`.
- Any decision's **Status** label or **reconsideration trigger** in
  `design-decisions.md`.
- Any 1.0 gate or D-gate **status value** in `api-freeze-review.md`.
- Any file under `rfcs/done/`. RFCs are historical records.
- `docs/src/guides/migration-*.md` — these correctly describe history.
  A migration guide saying "workspace split into three crates" in the
  v0.6 context is accurate and stays.
- `CHANGELOG.md` entries for already-released versions.
- Anything under `.git-exclude/` or `target/`.

## 7. Required implementation

Work the findings in this order. Each is independent; do not batch edits
across findings in a way that makes review harder.

### Step 1 — F-1, four-crate architecture (5 locations)

The canonical statement to use, adapted per page's register:

> Snora is four crates with a strict dependency direction:
> `snora-core ← snora-design ← snora-widgets ← snora`.
> `snora-core` (vocabulary) and `snora-design` (design tokens) have no
> iced dependency. `snora-widgets` (prefab visuals and the design style
> bridge) and `snora` (engine and facade) depend on iced. Applications
> depend only on `snora`. `snora-design` is reached through the opt-in
> `design` feature.

- `docs/src/reference/architecture.md` — update the opening sentence, the
  ASCII diagram, and add a `## snora-design — design tokens` section
  matching the register of the existing per-crate sections. Keep it
  consumer-facing: what it is, that it is iced-free, that it is opt-in,
  that applications reach it via `snora::design`.
- `docs/src/contributing/architecture.md` — add `crates/snora-design/` to
  the source-layout tree (mirror the real file list: `color.rs`,
  `contrast.rs`, `focus.rs`, `palette.rs`, `presets.rs` + `presets/`,
  `radius.rs`, `spacing.rs`, `tokens.rs`, `typography.rs`, `variants.rs`),
  and add `design.rs` + `design/` under `snora-widgets/src/` and
  `design.rs` + `design/` under `snora/src/`. Extend the "Crate boundaries
  — what goes where" section with the token rule: *if it is a design value
  with no iced type in its signature, it belongs in `snora-design`.*
- `README.md:98` — retitle the bullet to four crates and name
  `snora-design` in one clause. Keep the bullet the same length; the
  README is deliberately concise.
- `crates/snora/src/lib.rs` — update the `# Layering` diagram and the
  bullet list beneath it. The diagram must show `snora-design` as an
  iced-free dependency of `snora-widgets`. **Doc comment only.**
- `docs/src/contributing/alternate-engine-boundary.md:63` — the
  cross-reference text must match `design-decisions.md`'s new heading
  after Step 4.

### Step 2 — F-2, contributor procedures

In `docs/src/contributing/architecture.md`, replace "Adding a new prefab
widget" with the correct procedure:

1. Add the function in `crates/snora-widgets/src/<name>.rs`.
2. Declare the module and re-export it in `crates/snora-widgets/src/lib.rs`.
3. Re-export from `crates/snora/src/widget.rs` so it reaches
   `snora::widget::*`.
4. Document it in `docs/src/reference/widgets.md`.

Then add a sibling procedure, "Adding a new design primitive":

1. Add the module under `crates/snora-widgets/src/design/<name>.rs`, with
   tests in `crates/snora-widgets/src/design/<name>/tests.rs`.
2. Declare it in `crates/snora-widgets/src/design.rs`.
3. Re-export it from `crates/snora/src/design.rs` under a named submodule.
4. Document it under `docs/src/design/`.
5. Confirm it against `docs/src/contributing/api-governance.md` before
   promoting it out of experimental status.

While in this file, fix the stale relative paths in "Adding a new
vocabulary type" step 5 (`docs/reference/…` → `docs/src/reference/…`).

### Step 3 — F-3, release process

In `docs/src/contributing/release-process.md`:

- `:91` — `git tag X.Y.Z` (no `v`). Add one clause: *tags carry no `v`
  prefix, matching Rust crate convention.*
- `:48-55` — the workflow table gains a **Build cost** row
  (`build-cost.yaml`, push to `main` + tags, measure compile time and
  append a CSV row on release tags). Change "Three workflows" to "Four
  workflows".
- `:92` — "all four jobs", and name them: `rust-quality`,
  `feature-matrix`, `design-isolation`, `docs`.
- `:80-83` — the gate list becomes the full suite, matching `ci.yaml` and
  the v0.25.1 evidence set:
  ```text
  [ ] cargo fmt --check
  [ ] cargo clippy --workspace --all-targets --all-features -- -D warnings
  [ ] cargo test -p snora-core
  [ ] cargo test -p snora-design
  [ ] cargo test -p snora-widgets --features design
  [ ] cargo test -p snora --lib --all-features
  [ ] cargo test -p snora --test render_semantics   # CI hardware; may OOM locally
  [ ] cargo check -p snora --no-default-features
  [ ] mdbook build docs
  [ ] mdbook test docs
  ```
  Carry over the existing note that `render_semantics` links the full iced
  binary and is expected to run on CI rather than in constrained
  environments.

### Step 4 — F-4, decision register

In `docs/src/contributing/design-decisions.md`:

- Insert `## Why no \`PageContract\` trait` immediately before the
  orphaned "Early drafts (≤ 0.3) defined a trait…" paragraph, and remove
  the stray blank lines between the index table and that paragraph.
- Move the `[TabBar]:` / `[Crumb]:` link definitions (currently at
  `:244-245`, inside a section body) to the foot of the file.
- Index row `:22` — "Three crates, not two" becomes "Four crates
  (`-core`, `-design`, `-widgets`, engine)". Keep Status **Accepted** and
  the existing reconsideration trigger unchanged.
- Section heading `:159` "Why three crates instead of two" becomes "Why
  four crates". Extend the existing body with a short paragraph on why
  `snora-design` is separate from `snora-widgets` — the iced-free
  guarantee (NF-1) is the reason, and it is the same argument that
  separated `snora-core` from `snora`. Do not invent new rationale beyond
  what RFC-020/021 already record; cite them.
- **Warning-palette claim (`:317-321`) — finding already established.**
  iced 0.14 **does** provide a warning pair: see
  `iced_core-0.14.0/src/theme/palette.rs:18` (base `Palette::warning`) and
  `:297` (`Extended::warning`). Confirm this against the pinned source
  yourself, then correct the paragraph to say so. Add one line noting that
  `WARNING_COLOR` in `crates/snora/src/toast.rs:46` is therefore a removal
  candidate whose disposition is deferred to RFC-038 Q-2 — because toasts
  render on the design-inactive path, so removing it would change
  appearance for existing applications. **Do not remove or alter the
  constant.** Report your confirmation in the review request.

### Step 5 — F-5, gate tracker

In `docs/src/contributing/api-freeze-review.md`: update the status header
to the current released version and adjust the D-3/D-4 parenthetical spans
to the current minor range. **Every ✅/⬜ value stays exactly as it is.**

### Step 6 — F-7, RFC index

In `rfcs/README.md`: remove the blank lines inside the Done table so it
renders as one table; add a Proposed row for RFC-035; annotate the
`archive/` bullet as "created on first use".

### Step 7 — F-6, `Cargo.lock` policy (**unblocked**)

Owner decision: the lockfile is intentionally committed, revisitable with
good reason.

1. Remove line 5 (`Cargo.lock`) from `.gitignore`. Do not remove the file
   from the index — it stays tracked.
2. In `docs/src/contributing/architecture.md`, retitle the section from
   "Why no `Cargo.lock` in version control" to "Why `Cargo.lock` **is** in
   version control" and replace its body with the rationale quoted in
   RFC-035 F-6 (measurement attributability for the binary-size and
   build-cost budgets, given 17 example/probe binaries in the workspace).
   Note that the decision is revisitable.

### Step 8 — F-8, retroactive `[0.25.2]` CHANGELOG entry (**unblocked**)

Owner decision: **0.25.2 was published to crates.io.**

Add a `[0.25.2] — 2026-06-21` section immediately above `[0.25.1]`, under
a **Changed** heading. It must state:

- the workspace feature resolver moved from `2` to `3`;
- `members` changed from an explicit 21-entry list to the globs
  `crates/*` and `examples/*`, so new directories now join the workspace
  automatically;
- user-facing version snippets were updated in `README.md`,
  `docs/src/design/feature-flags.md`, and `docs/src/design/overview.md`;
- **no file under `crates/*/src/` changed**, so the published crates are
  functionally identical to 0.25.1 for downstream consumers, and the
  `resolver` key affects only this workspace's own builds.

Leave `[Unreleased]` reading "Nothing yet" until Step 9 adds this
change's own entry.

Do **not** re-release, re-tag, or yank. crates.io versions are immutable;
accurate retroactive documentation is the whole remedy.

Verify the facts yourself before writing the entry — `git diff 0.25.1
0.25.2` and `git diff 0.25.1 0.25.2 -- crates/` — and report both in the
review request. Do not copy the summary above on trust.

### Step 9 — budget-series note

In `docs/src/reference/binary-size-budget.md`, add a short note that the
trend series spans the 0.25.2 resolver change, that committed-lockfile
resolution means existing data points remain comparable, and that the next
lockfile regeneration is where comparability could break. One short
paragraph; do not restructure the page.

## 8. Required tests

No new tests. Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check -p snora --all-features
cargo test -p snora --lib --all-features
mdbook build docs
mdbook test docs
```

Plus the claim re-verification sweep:

```bash
grep -rn "three crates\|3 crates" --include='*.md' --include='*.rs' . \
  | grep -v target | grep -v .git-exclude
```

Every surviving hit must be a historical statement (CHANGELOG, migration
guide) — never a present-tense description of the current architecture.

## 9. Required documentation updates

The change *is* the documentation update. Additionally add an
`[Unreleased]` CHANGELOG entry under **Fixed** describing the
documentation correction — this is separate from F-8 and is not blocked.

## 10. Acceptance criteria

RFC-035 §Acceptance criteria, items 1–9, verbatim. Do not consider the
work complete until each is individually checkable.

## 11. Prohibited shortcuts

- Do not "fix" the architecture pages by deleting the stale sections.
  The pages must describe four crates, not describe nothing.
- Do not copy text from the v0.25.1 handoff bundle into the docs. That
  bundle is a snapshot and loses to in-tree files; re-derive every claim
  from source.
- Do not silently widen scope into the design-system theming question
  (whether snora should emit an `iced::Theme`). That is a live strategic
  question under separate discussion; touching it here pre-empts an owner
  decision.
- Do not disable or `ignore` a doc fence to make `mdbook test` pass.
- Do not reformat unrelated prose, reflow untouched paragraphs, or
  re-order sections not named in this handoff. Keep the diff reviewable.

## 12. Compatibility and security constraints

**Compatibility.** Zero. If the change alters any public API or any
runtime behavior, it has left scope — stop and escalate.

**Security.** No new data flow, dependency, or integration. Per the
project release rule, existing controls are verified as still valid
rather than re-modeled; requirements §1.4 and cross-cutting S-1…S-6 are
unchanged. Confirm this explicitly in the review request.

## 13. Known risks

Carried from RFC-035 §Risks. The one the implementer most directly
controls: **introducing a new factual error while correcting an old one.**
Every claim written must be traceable to a file you actually opened.

## 14. Required evidence

- List of changed files with a one-line reason each.
- Output of the six commands in §8.
- Output of the `grep` sweep in §8, with each surviving hit classified as
  historical.
- For Step 4: which way the iced 0.14 warning-pair check came out, and how
  you verified it.
- For Step 8: the output of `git diff 0.25.1 0.25.2` and
  `git diff 0.25.1 0.25.2 -- crates/`, confirming the changelog entry you
  wrote matches what actually shipped.

## 15. Required review-request format

Per the workflow policy §9.2:

1. Implementation summary
2. Addressed findings (F-1 … F-8, with disposition for each)
3. Changed files
4. Important implementation decisions
5. Differences from RFC-035 (expected: none — report any)
6. Executed commands and results
7. Build and static-analysis results
8. Unresolved issues, including anything blocked on Q-1/Q-2
9. Known limitations
10. Requested review focus — recommend: the four-crate wording in
    `contributing/architecture.md`, and the F-4 warning-palette
    verification
