# RFC 041 — Measurement integrity and gate 9 re-assessment

**Status.** Proposed
**Tracks.** Measurement and release automation integrity. Re-assesses 1.0
gate 9. Independent of the v0.26 appearance milestone.
**Touches.** `.github/workflows/binary-size.yaml`,
`.github/workflows/build-cost.yaml`, `scripts/append-binary-size-row.sh`,
`docs/src/reference/binary-size-budget.md`,
`docs/src/reference/build-cost-budget.md`,
`docs/src/contributing/api-freeze-review.md` (gate 9 row), possibly
`Cargo.toml` (`rust-version`).

## Summary

1.0 gate 9 — "Binary-size and compile-time trends monitored (≥2 data
points)" — is marked ✅. The underlying data does not support it.

The binary-size CSV contains three rows in which **every measurement
column is `N/A`**. No release since v0.19.1 has appended a row to any
budget CSV, despite six subsequent tags. The root cause is a tag-pattern
mismatch: both measurement workflows trigger and gate on `refs/tags/v*`,
while the project's 38 tags carry **no `v` prefix**.

The measurement automation has therefore never executed on a release tag
in the project's history.

This RFC establishes what the data actually supports, fixes the
automation, and recommends **reopening gate 9** until real data points
exist.

## Motivation

Gate 9 is one of eight gates counted as satisfied toward 1.0. It is also
the gate that governs the feature-gating indicators in
`feature-gating-criteria.md` (binary-size drift > 150 KB, build cost >
30 000 ms), which in turn govern whether `design` may ever become
default-on. Decisions are being made against this data.

Evidence-based quality management requires that a gate marked satisfied be
backed by evidence that exists. This one is not.

## Evidence

### E-1 — The binary-size series has no measurements

`docs/src/reference/binary-size-budget/binary-size.csv`:

```csv
version,engine_bytes,widgets_bytes,widgets_diff_bytes,design_bytes,design_diff_bytes,rustc,runner_os,date
0.17.0,N/A,N/A,N/A,N/A,N/A,...,unknown,2026-06-10T12:41:24Z
0.19.0,N/A,N/A,N/A,N/A,N/A,...,ubuntu-latest,2026-06-20
0.19.1,N/A,N/A,N/A,N/A,N/A,...,ubuntu-latest,2026-06-20
```

**Every measurement column in every row is `N/A`.** The v0.25.0 schema
change to the three-probe methodology (RFC-035 F-8 / v0.25.0 CHANGELOG)
converted all historical rows to `N/A` by design — "pre-v0.25 rows carry
`N/A`" — on the assumption that new rows would arrive from v0.25.0
onward. None did.

Usable binary-size data points: **zero.**

### E-2 — No release since v0.19.1 has appended a row

Tags `0.20.0` through `0.25.2` produced no CSV row in any of the three
series. `render-cost.csv` has a single row (`0.17.0`, `runner_os =
unknown`).

### E-3 — Root cause: tag-pattern mismatch

`binary-size.yaml`:

```yaml
on:
  push:
    tags:
      - 'v*.*.*'          # line 8 — workflow does not even trigger
...
if [[ "${GITHUB_REF}" == refs/tags/v* ]]; then     # line 52
if: startsWith(github.ref, 'refs/tags/v')          # line 116 (append step)
if: startsWith(github.ref, 'refs/tags/v')          # line 120 (commit-back)
```

`build-cost.yaml` carries the identical pattern at lines 7, 37, 88, 94.

The project's tags are `0.25.2`, `0.25.1`, … — **no `v` prefix**, per the
Rust crate convention the project follows and which RFC-035 F-3 just
confirmed in `release-process.md`. All 38 tags match this shape; none
matches `v*.*.*`.

Consequence: on a release tag the workflows do not run at all, so the
append and commit-back steps have never fired.

### E-4 — Data quality problems in what little exists

- `compile-time.csv` row `0.17.0` has `runner_os = unknown` (a sandbox
  measurement); `0.19.1` is `ubuntu-latest`. Comparing them is comparing
  two different machines, not a trend.
- `compile-time.csv:0.17.0` reports `example_hello_ms = 182000`, and
  `render-cost.csv:0.17.0` reports `hello_ms = 182000` — the same value in
  two series measuring different things. This suggests a script or
  transcription fault at 0.17.0 and should be treated as suspect.

### E-5 — The gate tracker overstates the data

`api-freeze-review.md` states: *"Gate 9 fully satisfied: binary-size has
three CI data points (v0.17.0, v0.19.0, v0.19.1)."* Two problems:
`0.17.0`'s `runner_os` is `unknown`, so it is not a CI data point; and all
three rows are `N/A`, so none is a data point at all.

## Goals

- G-1. Fix the tag-pattern mismatch so measurement runs on real tags.
- G-2. Re-assess gate 9 against evidence and correct its status.
- G-3. Correct the gate tracker's factual claims.
- G-4. Establish whether the series is comparable across the 0.25.2
  resolver change.
- G-5. Decide the MSRV question, which is entangled with G-4.

## Non-goals

- **N-1. No retroactive fabrication.** Historical rows are not
  back-filled with re-measured values presented as if taken at the time.
  If a baseline is re-established, it starts now and says so.
- **N-2. No change to the measurement methodology.** The three-probe
  approach (DEC-14) stands; this RFC fixes whether it *runs*.
- **N-3. Does not block the v0.26 appearance milestone.** Independent.
- **N-4. Does not by itself change any feature-gating decision.**

## Proposed changes

### C-1 — Fix the tag patterns (both workflows)

Change the trigger filter to match the project's actual tags, and relax
the three internal guards accordingly:

```yaml
on:
  push:
    tags:
      - '[0-9]+.[0-9]+.[0-9]+'
```

and replace `refs/tags/v*` guards with a prefix-agnostic form, deriving
`VERSION` by stripping `refs/tags/` only.

**Recommendation:** accept both shapes (`0.25.2` and `v0.25.2`) in the
guards so a future convention change does not silently disable
measurement again. The trigger filter can list both patterns. Silent
failure is the failure mode that produced this RFC.

### C-2 — Add a visible failure signal

The deeper defect is not the pattern; it is that **six releases passed
without anyone noticing no data arrived**. Whatever the fix, add a check
that fails loudly — for example, a release-checklist step that verifies a
row for the new version exists after the tag push (the checklist already
has steps to "confirm the workflow succeeded"; they were evidently not
actionable because the workflow never ran and so reported nothing).

### C-3 — Re-assess gate 9

On the evidence, gate 9 is **not satisfied**. Recommended disposition:

- Reopen gate 9 (⬜) in `api-freeze-review.md`.
- Correct the "three CI data points" claim.
- Re-satisfy it once ≥2 real post-fix data points exist on the same
  runner and same methodology.

This moves the 1.0 gate count from 8/10 to **7/10**. That is a correction
of the record, not a regression in the product — the gate was never
genuinely satisfied.

### C-4 — Resolve comparability and MSRV together

With no committed `Cargo.lock` (confirmed in RFC-035 F-6), CI resolves
dependencies fresh on every run. Combined with the `resolver = "2"` →
`"3"` switch at 0.25.2, measurements taken across that boundary may differ
for reasons unrelated to snora.

Because the fix in C-1 means the *first* real data points will all be
post-0.25.2, this is largely moot going forward — but it must be recorded,
because the pre-0.25.2 rows cannot be compared against post-fix rows.

The MSRV question is entangled and should be settled here: **no
`rust-version` is declared anywhere in the workspace**, so resolver 3's
MSRV-aware behavior has nothing to key on and the documented "stable ≥
1.85" policy is unenforced. Options:

| Option | Effect |
|---|---|
| **(a) Declare `rust-version = "1.85"`** in `[workspace.package]` ⭐ | Makes resolver 3 meaningful; publishes the MSRV to crates.io/docs.rs; makes the documented policy machine-checked. Needs a stated bump policy (recommend: raising MSRV is a **minor**, per pre-1.0 SemVer). |
| (b) Leave undeclared | Resolver 3 stays inert; MSRV remains documentation-only; nothing prevents a dependency bump from silently raising it. |
| (c) Declare and pin the lockfile | Maximum reproducibility; reverses `b7af344`, a deliberate prior decision. Only worth it if measurement stability proves insufficient after C-1. |

Recommended: **(a)**, with (c) held in reserve pending post-fix data.

## Compatibility and security

**Compatibility.** C-1/C-2 are CI-only. C-4(a) declares an MSRV that the
project already claims to honour; if any dependency currently requires
>1.85 the declaration will surface it, which is the point. No public API
changes.

**Security.** No new data flow, dependency, or integration. One note: the
workflows use `git-auto-commit-action` to push to `main` on tags. Since
that path has never executed, it is **unverified in practice** — the first
real run should be watched, and the `[skip ci]` recursion guard confirmed
to work rather than assumed.

## Testing and verification

| Check | Method |
|---|---|
| Trigger fires on a real tag | Dry-run via `workflow_dispatch`, then verify on the next release tag |
| Row is appended with correct field count | `append-binary-size-row.sh` validates 9 fields; confirm against the current schema |
| Commit-back does not recurse | Confirm `[skip ci]` suppresses a follow-on run |
| MSRV holds | `cargo +1.85 check --workspace --all-features` if C-4(a) is adopted |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| First real commit-back misbehaves on `main` | Medium | Medium | Path never exercised; watch the first run, and consider a dry-run tag first |
| Reopening gate 9 reads as a setback | Certain | Low | It is a correction of the record; the honest count was always 7/10 |
| Declaring MSRV surfaces a dependency needing >1.85 | Low | Medium | Better surfaced than latent; would be a real finding |
| Re-measured baseline is mistaken for historical data | Medium | Medium | N-1 forbids back-filling; new rows must be visibly a new baseline |

## Open questions

All three are **answered**. Recorded here so the Developer Handoff carries
no open decisions.

- **Q-1 — ANSWERED.** Adopt C-4(a): declare `rust-version = "1.85"` in
  `[workspace.package]`. **Raising the MSRV in future is a minor**, per
  pre-1.0 SemVer; declaring it now is additive and ships as part of the
  0.25.3 patch, since it documents support the project already claims
  rather than changing it.
- **Q-2 — ANSWERED.** Reopen gate 9 per C-3. The 1.0 gate count becomes
  **7 of 10**. Re-satisfy only once ≥2 real post-fix data points exist on
  the same runner and methodology.
- **Q-3 — ANSWERED.** Retain the pre-0.25.2 rows and annotate them as
  non-comparable. Measurement history is never deleted or back-filled
  (N-1).

**Release target: 0.25.3** (patch), alongside RFC-035 and RFC-036.

## Acceptance criteria

1. Both workflows trigger and append on the project's real tag format,
   demonstrated on an actual tag.
2. A failure signal exists so a silent no-op cannot recur unnoticed.
3. `api-freeze-review.md` gate 9 status and its supporting claims match
   the evidence.
4. Budget docs state the comparability boundary at 0.25.2.
5. The MSRV question is resolved per Q-1 and, if adopted, verified.
6. No measurement history is deleted or back-filled.

## Release implications

CI and documentation only; no crate content changes. Corrects the 1.0 gate
count from 8/10 to 7/10 if C-3 is accepted. Restores the evidence base
that `feature-gating-criteria.md` depends on — which matters before any
future decision about making `design` default-on.
