# Developer Handoff — RFC-041 measurement integrity and gate 9

**Governing RFC.** [RFC-041](../../proposed/041-measurement-integrity-and-gate-9.md)
**Status.** Inherited from RFC-041 (Proposed; all open questions answered by the owner).
**Release target.** 0.25.3 (patch), alongside RFC-035 and RFC-036.
**Implementation units.** One, in four steps. CI, manifest metadata, and docs.

---

> ## ⚠ Round 2 — only Step 3 needs work
>
> Steps 1, 2, and 4 were **reviewed and approved** — see
> `.git-exclude/reviewed/041-measurement-integrity-and-gate-9/review-result.md`.
> **Do not redo them and do not re-evidence them.** Your existing edits to
> the workflows, `release-process.md`, `api-freeze-review.md`, and the two
> budget docs stand as they are.
>
> **Step 3 only**, with the corrected value: the MSRV is **1.88**, not
> 1.85. Your escalation was right; the RFC was wrong.
>
> **Read `rfcs/handoffs/042-commit-cargo-lock/implementation-handoff.md`
> §5 before starting.** RFC-042 (commit `Cargo.lock`) lands in the same
> release and is order-dependent with this step: the lockfile must be
> committed **first**, as currently resolved, and the MSRV then verified
> against it. The two may be done as a single work item; they are separate
> handoffs for reviewability, not because the work must be split.

---

## 1. Task title

Fix the measurement workflows so they run on the project's real tag
format, declare the MSRV, reopen 1.0 gate 9, and annotate the
non-comparable budget history.

## 2. Purpose

Both measurement workflows gate on `refs/tags/v*`. All 38 of the project's
tags carry **no `v` prefix**. The append and commit-back steps have
therefore never fired on a release tag in the project's history. As a
result `binary-size.csv` holds three rows in which every measurement
column is `N/A`, and no budget CSV has gained a row since v0.19.1 across
six tags — while 1.0 gate 9 is marked satisfied on that data.

## 3. Background

Read `rfcs/proposed/041-measurement-integrity-and-gate-9.md` in full,
especially §Evidence (E-1 … E-5). This handoff executes it.

The owner has answered all three open questions; **there are no open
decisions in this work**:

- Declare `rust-version`; **the verified value is 1.88** (an earlier revision said 1.85 — wrong, see Step 3). Inherited rises are patch, chosen rises are minor.
- Reopen gate 9 — the 1.0 count becomes **7 of 10**.
- Retain the pre-0.25.2 CSV rows and annotate them; never delete or
  back-fill measurement history.

Conventions (the owner's rules document is not in the repository):

- **English only** for all prose and comments.
- Do **not** run workspace-wide `cargo fmt`: ~152 hunks of pre-existing
  drift exist, unrelated to this work.

## 4. Applicable requirements

- **1.0 gate 9** — binary-size and compile-time trends, ≥2 data points
- **DEC-14** three-probe binary-size methodology (unchanged by this work)
- **I-6 / DEC-09** a CI-script fix is a patch, not a minor
- **NF-5** binary-size and build-cost budgets tracked in CI

## 5. Change scope

| File | Step |
|---|---|
| `.github/workflows/binary-size.yaml` | 1 |
| `.github/workflows/build-cost.yaml` | 1 |
| `docs/src/contributing/release-process.md` | 2 (failure signal) |
| `Cargo.toml` (`[workspace.package]`) | 3 |
| `docs/src/contributing/api-freeze-review.md` | 4 (gate 9 row only) |
| `docs/src/reference/binary-size-budget.md` | 4 (comparability note) |
| `docs/src/reference/build-cost-budget.md` | 4 (comparability note) |
| `CHANGELOG.md` | `[Unreleased]` entry |

## 6. Explicit non-change scope

Do **not**:

- Delete, edit, or back-fill any existing CSV row. **N-1 is absolute**:
  historical rows are annotated, never rewritten. A re-measured value
  presented as historical data is a fabrication.
- Change the measurement methodology or the probe crates (DEC-14 stands).
- Touch `scripts/measure-*.sh` logic beyond what the tag fix requires.
- Touch any D-gate row (D-1 … D-8) — RFC-036 owns those, and both changes
  target `api-freeze-review.md`. **Only the gate 9 row is yours.**
- Touch any crate source, public API, or feature flag.

## 7. Required implementation

### Step 1 — Fix the tag patterns in both workflows

In `.github/workflows/binary-size.yaml` and
`.github/workflows/build-cost.yaml`:

1. **Trigger filter.** Replace `- 'v*.*.*'` so the project's real tags
   match. Accept **both** shapes so a future convention change cannot
   silently disable measurement again:

   ```yaml
   tags:
     - '[0-9]+.[0-9]+.[0-9]+'
     - 'v[0-9]+.[0-9]+.[0-9]+'
   ```

2. **Version extraction.** The `refs/tags/v*` test currently strips a `v`.
   Make it prefix-agnostic: strip `refs/tags/` and then an optional
   leading `v`. `VERSION` must come out as `0.25.3`, never `v0.25.3` —
   the CSV `version` column has no prefix in any existing row.

3. **Step guards.** Replace both
   `if: startsWith(github.ref, 'refs/tags/v')` conditions in each file
   with a check that matches either shape.

There are three guards in `binary-size.yaml` (≈ lines 52, 116, 120) and
three in `build-cost.yaml` (≈ lines 37, 88, 94). **Verify the line numbers
yourself; do not trust these.**

### Step 2 — Add a failure signal

The deeper defect is that six releases passed with nobody noticing no data
arrived. In `docs/src/contributing/release-process.md`, replace the two
existing "confirm the workflow run succeeded" checklist items with items
that are actually falsifiable — verifying that **a row for the new version
exists** in the CSV on `main` after the tag push, and naming what to do if
it does not (the workflow silently not running is the failure mode that
produced this RFC).

If you can additionally make the workflow itself fail loudly when the
append produces no new row, propose it in the review request — but do not
implement speculative CI logic without review.

### Step 3 — Declare the MSRV (**corrected: 1.88, not 1.85**)

An earlier revision of this step specified `1.85`. **That value was wrong**
— you verified it and escalated, and the owner has accepted the corrected
value. The effective floor is **1.88**, forced by `iced 0.14.0` and
`wgpu 27.0.1`; nothing in the resolved graph exceeds it.

1. In the root `Cargo.toml`, add to `[workspace.package]`:

   ```toml
   rust-version = "1.88"
   ```

   and inherit it in all four crate manifests with
   `rust-version.workspace = true`.

2. **Verify on a pinned toolchain. This check is mandatory, not
   optional:**

   ```bash
   cargo +1.88 check --workspace --all-features   # must pass
   cargo +1.87 check --workspace --all-features   # must fail (proves the floor is real)
   ```

   `cargo check` on the ambient toolchain is **not** sufficient and proves
   nothing: a newer active toolchain satisfies the graph regardless of the
   declared value. That is exactly how the wrong value nearly shipped.

   If `+1.88` fails, or `+1.87` unexpectedly passes, **stop and escalate**.
   Do not adjust the declared value to make a check pass.

3. Correct the false claim in `docs/src/getting-started/01-install.md:4`
   — currently "**Rust edition 2024** (rustc ≥ 1.85)". This is the **only**
   in-tree occurrence; the architect swept for it. State 1.88 and note the
   floor comes from `iced`, while snora's own code needs only the
   edition-2024 minimum of 1.85.

4. Record the bump policy in
   `docs/src/contributing/versioning-policy.md`:

   | Case | Level |
   |---|---|
   | **Inherited** rise (a dependency raises its `rust-version`) | **patch** |
   | **Chosen** rise (snora adopts a language/toolchain feature) | **minor** |

   Rationale to include: snora controls neither the timing nor the value
   of an inherited rise, and with `rust-version` declared, cargo's
   MSRV-aware resolver keeps users on older toolchains at the last
   compatible snora rather than breaking them.

5. Add an MSRV verification step to the release checklist in
   `docs/src/contributing/release-process.md`, so the floor cannot drift
   silently again:

   ```text
   [ ] cargo +<declared MSRV> check --workspace --all-features
   [ ] Confirm no resolved dependency declares a higher rust-version:
       cargo metadata --format-version 1 --all-features
   ```

**Version level for this change: patch.** No working configuration breaks
— 1.85 users already could not build snora — and it corrects a claim that
was false.

### Step 4 — Reopen gate 9 and annotate the history

In `docs/src/contributing/api-freeze-review.md`:

- Gate 9 row → `⬜`, with a one-line reason: the measurement workflows
  never fired on a release tag, so `binary-size.csv` contains no
  measurements and the series is not a trend. Reference RFC-041.
- Correct the "Gate 9 fully satisfied: binary-size has three CI data
  points" claim. It is wrong twice: `0.17.0`'s `runner_os` is `unknown`
  (not CI), and all three rows are `N/A`.
- Update the gate count from **8 of 10** to **7 of 10**, including the
  summary line near the top of the page.

In `docs/src/reference/binary-size-budget.md` and
`build-cost-budget.md`, add a short note that:

- rows before 0.25.3 predate the workflow fix and were not produced by the
  tag automation;
- the series spans the 0.25.2 `resolver = "2"` → `"3"` change, and with no
  committed `Cargo.lock` dependency resolution is not pinned between runs,
  so pre- and post-0.25.3 rows are **not comparable**;
- `compile-time.csv` row `0.17.0` was taken on `runner_os = unknown` (a
  sandbox), not CI, and reports `example_hello_ms = 182000` — the same
  value `render-cost.csv` reports for a different metric, so it should be
  treated as suspect.

Rows stay. Annotate; do not edit.

## 8. Required tests

```bash
cargo check --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
mdbook build docs
mdbook test docs
```

Plus YAML validity for both workflows (any linter, or `gh workflow view`
if available).

**The tag path cannot be fully verified before the release tag exists.**
Do not claim it works. State in the review request exactly what you did
verify (pattern matching against the real tag list, YAML validity,
version-extraction logic) and what remains unproven until 0.25.3 is
tagged.

Useful local check that the new pattern matches reality:

```bash
git tag | head -40      # confirm no tag carries a 'v' prefix
```

## 9. Acceptance criteria

RFC-041 §Acceptance criteria 1–6. In particular:

- Both workflows' trigger filters and all six guards match the project's
  real tag format.
- `VERSION` extraction yields `0.25.3`, not `v0.25.3`.
- `rust-version = "1.88"` is declared and `cargo +1.88 check --workspace --all-features` passes while `+1.87` fails.
- Gate 9 is `⬜`, the count reads 7 of 10, and no D-gate row changed.
- No CSV row is deleted, edited, or back-filled.

## 10. Prohibited shortcuts

- **Do not** back-fill CSV rows for 0.20.0 … 0.25.2 by re-measuring now.
  That would present today's numbers as historical data.
- **Do not** mark gate 9 satisfied because the fix is in. It is satisfied
  when ≥2 real data points exist, which cannot be true before 0.25.3 ships.
- **Do not** adjust the declared MSRV to make a check pass — escalate
  instead. (This already happened once and was handled correctly.)
- **Do not** edit any D-gate row.

## 11. Compatibility and security

**Compatibility.** CI and metadata only; no crate content changes.
`rust-version = "1.88"` declares the floor the dependency graph already
enforces; it does not narrow support that anyone actually had. Users on
1.85 could never build snora.

**Security.** One item to flag rather than assume: the workflows push to
`main` via `git-auto-commit-action` on tags. **That path has never
executed.** Confirm the `[skip ci]` recursion guard and the `concurrency`
group are intact, and say in the review request that the first real run
should be watched. Do not assert it works.

## 12. Known risks

Per RFC-041 §Risks. The two you control: back-filling history (forbidden
above), and claiming the tag path is verified when it cannot be until a
tag exists.

## 13. Required evidence

- Diffs of both workflow files.
- `git tag | head -40` output confirming no tag carries a `v` prefix.
- The version-extraction logic, and how you tested it (a shell trace of
  the extraction against `refs/tags/0.25.3` is sufficient).
- `cargo +1.88 check` (pass) and `cargo +1.87 check` (fail) output.
- `cargo metadata` confirmation that no resolved package declares a
  rust-version above 1.88.
- `mdbook build docs` / `mdbook test docs` output.
- Explicit statement that no CSV row changed (`git diff --stat` over
  `docs/src/reference/**/*.csv` must be empty).
- Explicit statement that no D-gate row changed.

## 14. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, and `evidence/`, under
`.git-exclude/review-request/041-measurement-integrity-and-gate-9/`.
Report paths relative to the project root.

Call out explicitly in "Unresolved issues" what remains unverifiable until
0.25.3 is tagged.
