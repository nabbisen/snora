# Developer Handoff — RFC-052 profile-scoped clean

**Governing RFC.** [RFC-052](../../done/052-clean-never-invalidates-release-artifacts.md)
**Status.** Inherited from RFC-052 — Implemented (v0.31.0).
**Release target.** 0.31.0.
**Implementation units.** One. **Must land before RFC-050**, which is blocked
on it.

---

## 1. Task title

Make `measure-compile-time.sh`'s per-measurement clean reach the profiles the
measurements actually build, and record the resulting discontinuity.

## 2. Purpose, and what is already established

You found this. Nothing in §"Motivation" needs re-deriving — the mechanism is
confirmed, independently reproduced by the reviewer, and written up in RFC-052
with the evidence.

**Verify it still holds before changing anything** (one command, §5 step 0),
then implement. If the behaviour has changed under a newer cargo, stop and
report rather than "fixing" something that no longer exists.

Release shape is undecided: 0.30.0 may ship RFC-051 alone with this following
later. **That does not affect this task.** Do not add version numbers or
release-note wording that assumes a particular release.

## 3. Background — read first

- `rfcs/done/052-clean-never-invalidates-release-artifacts.md` in full.
- `docs/src/reference/build-cost-budget.md`'s three existing data-integrity
  notes (RFC-041, RFC-043, RFC-044). **Yours is the fourth.** Match their
  structure: what was wrong, the evidence, what closes it.
- `scripts/measure-compile-time.sh` — 60 lines, and the only place the row is
  produced.

Conventions: English only. `cargo fmt --all --check` is CI-enforced as of
0.28.1.

## 4. The fix

`measure_ms` currently cleans one profile for six measurements spanning three:

```bash
cargo clean -p snora-core -p snora-design -p snora-widgets -p snora 2>/dev/null || true
```

Clean **all three** profiles, unconditionally, before every measurement:

```bash
cargo clean -p <snora crates>                              # dev
cargo clean -p <snora crates> --release                    # release
cargo clean -p <snora crates> --profile release-baseline   # release-baseline
```

Cleaning all three rather than passing the target profile per call site is the
RFC's explicit choice: it decouples a measurement from its profile, so a
future measurement cannot be added with the wrong one. Keep that property —
do not "optimise" it back into per-call-site profiles.

Apply the same to the separate workbench clean, which has the identical
defect:

```bash
cargo clean -p snora-example-design-workbench 2>/dev/null || true
```

Keep the `2>/dev/null || true` guards. A clean that fails must not abort a
release measurement.

**Update the script's design note.** It currently claims per-measurement cold
builds, which was false for four of six measurements. Say what the clean now
does and why it stays scoped to snora's crates rather than the full workspace.

## 5. Required implementation

### Step 0 — Confirm the defect still reproduces

```bash
cargo build -p snora-core --release
ls target/release/libsnora_core.rlib                       # present
cargo clean -p snora-core -p snora-design -p snora-widgets -p snora
ls target/release/libsnora_core.rlib                       # STILL present → confirmed
```

### Step 1 — The fix

Per §4.

### Step 2 — Demonstrate the repair, with `Compiling` lines

The proof is not a timing. It is that compilation now happens:

```bash
# after the fix, the two repaired measurements must show snora's crates compiling
cargo build -p snora-widgets --features design --release 2>&1 | grep Compiling
cargo build -p snora --no-default-features --release 2>&1 | grep Compiling
```

Capture both, before and after. A before/after pair where the "before" shows
`Finished in 0.07s` with no `Compiling` line is the whole finding in two
lines.

### Step 3 — Answer Q-2: which columns actually moved

Run the **full script** — not individual commands — once with the old clean
and once with the new, and report both rows.

**Read §6 before doing this.** Measuring it badly is easy and I did it myself.

Report per column: moved materially, moved slightly, or unchanged. That is
what lets the discontinuity note name the affected columns instead of assuming
all four moved.

### Step 4 — The fourth data-integrity note

`build-cost-budget.md`, matching the existing three. It must carry:

- the mechanism (`cargo clean -p` reaches dev only; `-r`/`--profile` exist
  because of it);
- the before/after `Compiling` evidence from step 2;
- **which columns are discontinuous**, from step 3 — not a blanket claim;
- that rows before and after the fix are not comparable for those columns;
- explicitly, that **no historical row was edited** (RFC-041 N-1).

### Step 5 — Gate 9b

Record in `api-freeze-review.md` that gate 9b's clock **reset again**, this
being the third methodology discontinuity, and why. Do not soften it: the gate
moved further away.

### Step 6 — CHANGELOG

`[Unreleased]` → **Fixed**. Name the affected columns so the discontinuity is
discoverable from the changelog, not only from the budget page.

## 6. The trap — measuring the before/after badly

I tried to quantify the delta locally and **got a contaminated result**: 244 ms
versus 260 ms, from a sequence where the earlier variant had already populated
the artifacts the later variant needed. I reported it as unreliable rather than
as data. Do not repeat it.

Rules for step 3:

- **Run the whole script per variant**, never individual `cargo` commands. The
  measurements are order-dependent by design — each builds on the previous
  one's warm artifacts — so a single command in isolation measures a different
  thing than the same command inside the sequence.
- **Reset to the same starting state between variants.** State what that was.
  Whatever you choose, both runs must start from it.
- **Do not interleave.** Complete one variant fully, reset, then the other.
- **Absolute numbers are local and are not the deliverable.** CI is the
  authority; snora's crates are small enough that a warm developer machine and
  an uncached CI runner disagree substantially. Report the *shape* of the
  change — which columns moved and roughly how much — and say plainly that the
  magnitudes are local.

If you cannot get a clean comparison, **say so and report only the `Compiling`
evidence.** That is sufficient for the finding and for the note. An honest
"could not measure this cleanly" beats a number that will be quoted later.

## 7. Explicit non-change scope

Do **not**:

- **Implement RFC-050's ratio columns.** That RFC is blocked on this one and
  its ratio selection is being re-derived. No new CSV columns in this task.
- **Edit or back-fill any historical row.** Append-only (RFC-041 N-1). The
  discontinuity is recorded, not repaired.
- **Re-enable dependency caching** in `build-cost.yaml` (RFC-043).
- **Change which commands are measured**, or add/remove measurements.
- **Pre-warm iced to make `build_widgets_ms` comparable** to the snora-only
  columns. That is a design question for RFC-050's re-derivation, not a bug
  fix, and doing it here would confound the before/after in step 3.
- **Touch `measure-binary-size.sh` or `binary-size.csv`.** Gate 9a is
  satisfied and unaffected.
- Change where the script writes its per-measurement logs. The workflow
  uploads them as artifacts.

## 8. Required tests

```bash
bash -n scripts/measure-compile-time.sh
scripts/measure-compile-time.sh 0.0.0-test      # run it; inspect the row
mdbook build docs && mdbook test docs
cargo fmt --all --check
```

The script has no `cargo test` coverage; **running it and reading the output
is the test.** Include the emitted row.

## 9. Acceptance criteria

RFC-052 §Acceptance criteria 1–6:

1. All three profiles cleaned for snora's crates before every measurement;
   workbench clean likewise.
2. `Compiling` lines demonstrated for both repaired measurements — evidence,
   not assertion.
3. `build-cost-budget.md` carries the fourth data-integrity note.
4. Q-2 answered: which columns moved, with the honesty caveat from §6 if the
   comparison could not be made cleanly.
5. Gate 9b records the clock reset and the reason.
6. No historical row edited.

Plus: the script's design note no longer claims something false (§4).

## 10. Prohibited shortcuts

- Do not report timings as the proof. `Compiling` lines are the proof; timings
  are context.
- Do not assert all four release-profile columns moved without measuring
  (§5 step 3).
- Do not quietly drop a column that now looks odd. If a corrected measurement
  produces something implausible, that is a finding — report it.
- Do not mark gate 9b as progressing. It regressed.

## 11. Compatibility and security

**Compatibility.** No library change, no API, no CSV schema change. Two
columns change meaning in future rows; that is the intent and is recorded as a
discontinuity.

**Security.** No new data flow, dependency, or integration.

## 12. Required evidence

- Full diff of `scripts/measure-compile-time.sh`.
- **Before/after `Compiling` output** for both repaired measurements (§5 step 2).
- Step 0's confirmation that the defect still reproduces.
- The two full script rows from step 3, with the starting state stated, and
  your per-column assessment.
- The emitted row from §8.
- `build-cost-budget.md` diff in full.
- `mdbook build` / `mdbook test` output.

## 13. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/052-clean-never-invalidates-release-artifacts/`.
**State the single entry-point path to hand to the reviewer** in the
completion summary.

**Requested review focus:** step 3's methodology. The fix itself is three
lines and self-evident once the `Compiling` output is in hand; whether the
before/after comparison is sound — or honestly declared unsound — is the part
worth reviewing, because its output becomes a permanent note in the budget
record.
