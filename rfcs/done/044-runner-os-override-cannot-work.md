# RFC 044 — `RUNNER_OS` cannot be overridden; RFC-043 AC-4 is unmet

**Status.** Implemented (v0.27.0)
**Tracks.** Measurement integrity. Follow-on from RFC-043, whose
acceptance criterion 4 did not take effect in CI.
**Touches.** `.github/workflows/binary-size.yaml`,
`.github/workflows/build-cost.yaml`, `scripts/measure-binary-size.sh`,
`scripts/measure-compile-time.sh`, both budget docs.

## Summary

RFC-043 AC-4 required the `runner_os` CSV column to be stabilised to
`ubuntu-latest`. The implementation set `env: RUNNER_OS: ubuntu-latest` on
the measuring step, the workflow file was reviewed and the criterion marked
verified — and the v0.26.0 rows still emit **`Linux`**.

**GitHub Actions does not permit overwriting `RUNNER_*` variables.** The
override is silently ignored. The criterion cannot be met by that
mechanism, and no amount of reviewing the workflow file would have shown
it: only the emitted row does.

## Evidence

The v0.26.0 rows, appended by CI after the tag:

```csv
0.26.0,15687888,15731536,43648,15734224,2688,rustc_1.97.1_…,Linux,2026-08-03
0.26.0,60341,101637,421,154657,529,6133,rustc_1.97.1_…,Linux,2026-08-03T08:43:55Z
```

`runner_os` = `Linux`, exactly as at 0.25.3 — the value RFC-043 set out to
change.

The mechanism is present and correct-looking
(`binary-size.yaml`, `Measure` step):

```yaml
env:
  RUNNER_OS: ubuntu-latest
run: |
  ROW=$(scripts/measure-binary-size.sh "${{ steps.version.outputs.version }}")
```

and the script consumes it with a fallback:

```bash
RUNNER_OS="${RUNNER_OS:-unknown}"
```

Both halves are individually sound. The step-level `env` simply does not
take effect, because GitHub reserves the `RUNNER_*` namespace — its
documented position is that default variables named `GITHUB_*` and
`RUNNER_*` cannot be overwritten.

## Why this was not caught

Two review passes looked at the **workflow file** and confirmed the
override was present and well-commented. Neither looked at what CI
actually emitted, because at review time no post-fix row existed yet — the
first one could only appear after the tag.

This is the same failure mode RFC-041 was raised to fix, one level up:
*verifying the instrument rather than its output*. RFC-041 replaced
"confirm the workflow run is green" with "confirm a row exists"; the
analogous gap here is that a row existing says nothing about whether its
*contents* are what the change intended.

## Goals

- G-1. `runner_os` is stable and matches historical rows.
- G-2. The release checklist verifies emitted row *contents*, not only row
  existence.

## Non-goals

- **N-1. No back-filling or editing of existing rows.** RFC-041 N-1 stands
  absolutely. The 0.25.3 and 0.26.0 rows keep `Linux`; they are annotated,
  never rewritten.
- **N-2. No change to the measurement methodology.** RFC-043's probe fix
  worked — `widgets_diff_bytes` moved 0 → 43,648 and build-cost is cold
  again. Only the `runner_os` column is at issue.
- **N-3. Not a re-opening of RFC-043.** That RFC shipped; this records the
  one criterion that did not land, per RFC-000's "don't reopen `done/`"
  guidance.

## Proposed changes

### C-1 — Use a variable GitHub does not reserve

Rename the override to something outside the reserved namespace — e.g.
`SNORA_RUNNER_OS` — and have both scripts prefer it:

```bash
RUNNER_OS="${SNORA_RUNNER_OS:-${RUNNER_OS:-unknown}}"
```

Passing the value as a positional argument to the script is equally
acceptable and arguably clearer; the implementer should pick one and say
why. What matters is that the value no longer travels through a name
GitHub owns.

### C-2 — Verify contents, not just existence

The release checklist currently verifies that a row for the new version
exists. Extend it to check the row's **fields**: that `runner_os` matches
the historical spelling, and that `widgets_diff_bytes` is non-zero (which
would have caught RFC-043's original defect at release time rather than
one release later).

### C-3 — Annotate

Record in both budget docs that `runner_os` reads `Linux` for 0.25.3 and
0.26.0, and `ubuntu-latest` before and after, and why. A reader grouping
by that column needs to know.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| The replacement variable is also shadowed somewhere | Low | Low | Verify against the emitted row, not the workflow file — that is the whole lesson here |
| Treated as trivial and shipped unverified again | Medium | Medium | C-2 makes the check part of the release gate |
| Someone "fixes" the historical rows | Low | High | N-1 restated; the annotation explains the discontinuity instead |

## Acceptance criteria

1. The next tagged release emits `runner_os = ubuntu-latest` in **both**
   CSVs — verified from the committed row, not from the workflow file.
2. The release checklist verifies row contents, including a non-zero
   `widgets_diff_bytes`.
3. Budget docs annotate the two-row `Linux` discontinuity.
4. No existing CSV row is edited or deleted.

## Release implications

Patch-level; CI, scripts and docs only. No crate content change. Does not
affect gate 9's arithmetic — 0.26.0 remains the first valid data point
under RFC-043's corrected methodology, and one more is needed regardless.
