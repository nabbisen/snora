# Developer Handoff — RFC-042 commit `Cargo.lock`

**Governing RFC.** **RFC-042** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-042 — Implemented (v0.25.3).
**Release target.** 0.25.3 (patch), alongside RFC-035, RFC-036, RFC-041.
**Implementation units.** One. Small.

---

## 1. Task title

Track `Cargo.lock`, record why, and add a scheduled unpinned-build job so
early warning of upstream movement is preserved.

## 2. Purpose

Budget measurements and the MSRV floor both depend on dependency
resolution, and resolution is currently unpinned. RFC-041 is rebuilding
1.0 gate 9 from zero data points; those measurements are only attributable
to snora's own changes if resolution is fixed. The MSRV floor already
moved unobserved once (documented 1.85, actual 1.88).

## 3. Background

Read `rfcs/done/042-commit-cargo-lock.md` in full. It reverses
`b7af344` ("remove Cargo.lock from vcs"), which RFC-035 F-6 confirmed was
deliberate — so this is a governed reversal, not a correction of a mistake.

Conventions (the owner's rules document is not in the repository):

- **English only** for all prose and comments.
- Do **not** run workspace-wide `cargo fmt`: ~152 hunks of pre-existing
  drift exist, unrelated to this work.

## 4. Applicable requirements

- **RFC-041** gate 9 rebuild and the MSRV declaration (this work makes both
  meaningful)
- **NF-5** binary-size and build-cost budgets tracked in CI
- **DEC-14** three-probe measurement methodology (unchanged)

## 5. Ordering — read before starting

**This handoff and RFC-041's interact.** The declared MSRV is only
meaningful relative to a specific resolution.

Do it in this order:

1. Commit `Cargo.lock` **as currently resolved** (Step 1). Do not run
   `cargo update` first — the current resolution is the one against which
   1.88 was verified.
2. Then complete or re-verify RFC-041 Step 3's MSRV checks **against that
   lockfile**.

Required final state: `Cargo.lock` is tracked **and**
`cargo +1.88 check --workspace --all-features` passes against it.

If RFC-041 Step 3 has already landed when you start, just re-run its
verification after committing the lockfile and report the result.

## 6. Change scope

| File | Purpose |
|---|---|
| `Cargo.lock` | newly tracked |
| `.gitignore` | remove the `Cargo.lock` line |
| `docs/src/contributing/architecture.md` | replace the rationale section |
| `.github/workflows/unpinned-build.yaml` (new; name at your discretion) | scheduled job |
| `docs/src/contributing/release-process.md` | checklist step |
| `CHANGELOG.md` | `[Unreleased]` entry |

## 7. Explicit non-change scope

Do **not**:

- Run `cargo update` as part of this change. The committed lockfile must
  be the resolution currently in use. If the working tree has no
  `Cargo.lock`, generate it with a plain `cargo check` and say so.
- Change any dependency range in any `Cargo.toml`. **N-2**: ranges are
  untouched; only resolution is recorded.
- Vendor dependencies.
- Let the scheduled job commit anything.
- Touch any crate source, public API, or feature flag.

## 8. Required implementation

### Step 1 — Track the lockfile

```bash
git add -f Cargo.lock          # -f: still ignored at this point
```

Then remove the `Cargo.lock` line from `.gitignore` in the same change.
Confirm with `git ls-files --error-unmatch Cargo.lock` (must exit 0 — note
that plain `git ls-files <path>` exits 0 either way and prints nothing on
no match, which has already caused one false finding in this project).

### Step 2 — Replace the architecture rationale

`docs/src/contributing/architecture.md` has a section "Why no `Cargo.lock`
in version control". It is accurate today and becomes false at Step 1.
Retitle it ("Why `Cargo.lock` **is** in version control") and rewrite the
body to record:

- the workspace is also a measurement harness — 17 example and probe
  binaries feed the binary-size and build-cost budgets, which track drift
  *between releases*, so resolution must be fixed for a delta to be
  attributable to snora;
- the MSRV floor is inherited from `iced`/`wgpu` and a lockfile makes its
  movement a reviewable diff rather than a surprise;
- **the cost accepted**: CI no longer notices that a fresh resolution has
  broken, which is why the scheduled job in Step 3 exists;
- that committing a library's lockfile has **no effect on downstream
  consumers** — they resolve against their own.

Record the cost, not just the conclusion. A rationale that lists only
benefits is not a rationale.

### Step 3 — Scheduled unpinned-build job

New workflow, weekly schedule plus `workflow_dispatch`:

```yaml
on:
  schedule:
    - cron: '<weekly>'
  workflow_dispatch:
```

Steps:

```bash
cargo update
cargo check --workspace --all-features
cargo test -p snora-core -p snora-design
cargo +1.88 check --workspace --all-features    # MSRV still holds?
```

Requirements:

- It must **not** commit or push the updated lockfile.
- It must fail loudly. A failure means "upstream moved — investigate",
  not "auto-merge".
- The MSRV line is deliberate: it makes an **inherited** floor rise visible
  weekly rather than at release time, which is what makes RFC-041's
  inherited-rise-is-a-patch policy actionable.
- Prove it runs via `workflow_dispatch` before submitting, and include the
  run result in the evidence. Do not submit a workflow that has never
  executed — that is precisely the defect RFC-041 exists to fix.

### Step 4 — Release checklist

Add to `docs/src/contributing/release-process.md`: confirm `Cargo.lock` is
current and its diff is intentional before tagging. A lockfile that drifts
unreviewed is worse than none, because it carries the implied assertion
that someone looked.

### Step 5 — CHANGELOG

`[Unreleased]` under **Changed**: `Cargo.lock` is now tracked; rationale
and the accepted cost; note explicitly that downstream consumers are
unaffected.

## 9. Required tests

```bash
cargo check --workspace --all-features
cargo +1.88 check --workspace --all-features
mdbook build docs
mdbook test docs
```

Plus YAML validity for the new workflow, and a successful
`workflow_dispatch` run.

## 10. Acceptance criteria

RFC-042 §Acceptance criteria 1–5, plus the ordering requirement in §5:

1. `Cargo.lock` tracked; `.gitignore` no longer lists it.
2. `architecture.md` records why it is committed **and** the cost accepted.
3. Scheduled workflow exists, does not commit, includes the MSRV check, and
   has been proven to run via `workflow_dispatch`.
4. Release checklist requires confirming the lockfile diff.
5. **No dependency range in any `Cargo.toml` changed** — show
   `git diff -- '**/Cargo.toml'` is limited to what RFC-041 Step 3
   contributes.
6. `cargo +1.88 check --workspace --all-features` passes against the
   committed lockfile.

## 11. Prohibited shortcuts

- Do not `cargo update` before committing the lockfile.
- Do not let the scheduled job auto-commit or auto-PR the lockfile.
- Do not submit the workflow without dispatching it once.
- Do not adjust dependency ranges to make anything resolve more neatly.

## 12. Compatibility and security

**Compatibility.** No crate content changes. A library's committed
lockfile is ignored when the library is consumed as a dependency, so
downstream is unaffected in either direction. State this explicitly.

**Security.** Mildly positive — dependency changes become visible in
review. Do **not** claim it protects against a compromised version already
in the graph; it does not, and `cargo audit` is not in scope.

## 13. Known risks

Per RFC-042 §Risks. The one you control: shipping a scheduled workflow
that has never run. Dispatch it.

## 14. Required evidence

- `git ls-files --error-unmatch Cargo.lock` exiting 0.
- Diff of `.gitignore` and `architecture.md`.
- The new workflow file, and the URL/output of its `workflow_dispatch` run.
- `cargo +1.88 check` output against the committed lockfile.
- `git diff -- '**/Cargo.toml'` showing no dependency-range change.
- `mdbook build docs` / `mdbook test docs` output.

## 15. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, and `evidence/`, under
`.git-exclude/review-request/042-commit-cargo-lock/`. Report paths relative
to the project root.
