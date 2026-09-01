# Developer Handoff — RFC-044 `runner_os` override and content verification

**Governing RFC.** **RFC-044** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-044 — Implemented (v0.27.0).
**Release target.** Next release — fold in rather than cutting one for it.
**Implementation units.** One. Small: CI, two scripts, docs.

---

## 1. Task title

Emit `runner_os` through a variable GitHub does not reserve, and extend the
release checklist to verify row *contents* rather than only row existence.

## 2. Purpose

RFC-043 AC-4 required `runner_os` to read `ubuntu-latest`. The v0.26.0 rows
still read `Linux`. The override is present and looks correct — GitHub
simply ignores it, because `RUNNER_*` is a reserved namespace.

The second half matters more than the first. A cosmetic column is worth
little; a release check that confirms a row exists **without looking at
what is in it** is how RFC-043's `widgets_diff_bytes = 0` survived a whole
release, and how this did too.

## 3. Background — read first

`rfcs/done/044-runner-os-override-cannot-work.md` in full.

The key fact, and the reason two review passes missed it: the workflow
step and the script are *each* correct in isolation.

```yaml
env:
  RUNNER_OS: ubuntu-latest        # silently ignored — reserved namespace
```
```bash
RUNNER_OS="${RUNNER_OS:-unknown}"  # correct, but never sees the override
```

Only the emitted CSV row reveals it. **Read the row, not the workflow.**

Conventions: English only. Do not run workspace-wide `cargo fmt` (~152
hunks of pre-existing drift).

## 4. Change scope

| File | Step |
|---|---|
| `.github/workflows/binary-size.yaml` | 1 |
| `.github/workflows/build-cost.yaml` | 1 |
| `scripts/measure-binary-size.sh` | 1 |
| `scripts/measure-compile-time.sh` | 1 |
| `docs/src/contributing/release-process.md` | 2 |
| `docs/src/reference/binary-size-budget.md`, `build-cost-budget.md` | 3 |
| `CHANGELOG.md` | `[Unreleased]` |

## 5. Explicit non-change scope

Do **not**:

- **Edit, delete, or back-fill any CSV row.** RFC-041 N-1 is absolute and
  carries forward again. The 0.25.3 and 0.26.0 rows keep `Linux`; they are
  annotated, never corrected. This is the third handoff to say so — the
  temptation grows each time a row looks wrong, and the answer does not
  change.
- Mark gate 9 satisfied. v0.26.0 is data point **one** of two under
  RFC-043's corrected methodology; the next release supplies the second.
- Change the measurement methodology, the probes, or
  `feature-gating-criteria.md` thresholds.
- Touch any crate source.

## 6. Required implementation

### Step 1 — Move the value out of the reserved namespace

Pick one mechanism and apply it consistently to **both** workflows and
**both** scripts:

- **(a)** a non-reserved env var, e.g. `SNORA_RUNNER_OS`, consumed as
  `RUNNER_OS="${SNORA_RUNNER_OS:-${RUNNER_OS:-unknown}}"`; or
- **(b)** pass the value to the script as a positional argument.

Either is acceptable. **Say which you chose and why** in the review
request — (b) is arguably clearer since it removes the ambient-environment
dependency entirely, but it changes the scripts' call signature, which
affects anyone invoking them by hand.

Keep the explanatory comment that is already in `binary-size.yaml` — it is
good — but correct it: the current wording implies the override works.

### Step 2 — Verify contents, not just existence

`release-process.md` already requires confirming a row exists for the new
version. Extend both of those checklist items to check the row's fields:

- `runner_os` matches the historical spelling (`ubuntu-latest`);
- `widgets_diff_bytes` is **non-zero** — this is the check that would have
  caught RFC-043's original defect at release time rather than a release
  later;
- `check_workspace_ms` is plausibly cold (tens of seconds, not
  milliseconds) — the check that would have caught the warm-cache defect.

Write them as falsifiable assertions a human can run, in the same style as
the existing row-existence checks. State what to do when one fails: treat
it as a release blocker and investigate before publishing, exactly as with
a missing row.

### Step 3 — Annotate the discontinuity

In both budget docs, record that `runner_os` reads `Linux` for 0.25.3 and
0.26.0 and `ubuntu-latest` on either side, and why. Anyone grouping by that
column needs to know it is not a machine change.

## 7. Required tests

```bash
mdbook build docs && mdbook test docs
```

Plus YAML validity for both workflows.

**The fix cannot be fully verified until the next tagged release**, since
`runner_os` only appears in a row the tag path writes. Do not claim
otherwise. State in the review request exactly what you verified (the
mechanism, script behaviour with the variable set locally, YAML validity)
and what remains unproven until the next tag.

A useful local check — run the script with the variable set and confirm the
emitted field:

```bash
SNORA_RUNNER_OS=ubuntu-latest bash scripts/measure-binary-size.sh 0.0.0-test | cut -d, -f8
```

Report the output. That proves the script half; only a tag proves the
workflow half.

## 8. Acceptance criteria

RFC-044 §Acceptance criteria 1–4. Note that **criterion 1 cannot be closed
by this change** — it closes when the next release emits
`runner_os = ubuntu-latest` in both CSVs, verified from the committed row.
Say so plainly rather than claiming it met.

## 9. Prohibited shortcuts

- Do not verify by reading the workflow file. That is precisely what failed
  twice.
- Do not edit historical rows to make the column consistent.
- Do not mark RFC-044's criterion 1 satisfied before a tagged release
  proves it.

## 10. Compatibility and security

Neither affected: CI, scripts and docs only; no crate content, public API,
or feature flag changes. State this explicitly.

## 11. Required evidence

- Diffs of both workflows and both scripts.
- The local script run from §7 showing the `runner_os` field.
- `mdbook build docs` / `mdbook test docs` output.
- `git diff --stat -- 'docs/src/reference/**/*.csv'` — **must be empty**.
- `git diff --stat -- crates/` — must be empty.
- An explicit statement of what remains unverifiable until the next tag.

## 12. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/044-runner-os-override-cannot-work/`. **State
the single entry-point path to hand to the reviewer** in the completion
summary.

**Requested review focus:** whether the new checklist assertions are
genuinely falsifiable — i.e. whether a human following them would actually
catch a wrong value, rather than confirming something that is true by
construction.
