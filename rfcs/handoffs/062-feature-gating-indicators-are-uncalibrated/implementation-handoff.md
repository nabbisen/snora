# Developer Handoff — RFC-062 feature-gating indicators

**Governing RFC.** **RFC-062** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-062 — Accepted (owner, 2026-08-18).
**Release target.** 0.36.0, alongside RFC-061. **Documentation only** — no code.
**Implementation units.** One.

---

## 1. Task title

Resolve the feature-gating status table's contradiction, retire the invalid
compile-time proxy, correct indicator 2's stated method, record every
indicator's measured value, and attach a check to the accessibility-tree
trigger.

## 2. Purpose

`feature-gating-criteria.md` decides when snora splits `widgets` into per-widget
gates. Its status table records indicator 1 as **"Within budget"** while citing
**≈96 s against a 30 s threshold** — 3.2× over, in the document that decides.
Every row since 0.26.0 reads 2.2×–3.5× over.

The cause is that RFC-043 changed the measurement and nobody recalibrated the
indicators consuming it.

## 3. Q-1 is decided: option (b). Do not implement (a) or (c)

RFC-062 left three shapes open. **Checking them closed two.**

- **(c) measure marginal cost** — the most coherent with what the indicator
  decides, and **blocked**. No existing column expresses "what `widgets` adds
  over an engine-only build" in compile time: `build_widgets_ms` builds
  snora-widgets including iced, `build_engine_only_ms` builds the engine against
  a warm graph, and neither is the marginal quantity. It would need a new
  column, and RFC-050's non-goals state: *"No new measurement columns. Adding
  one would reset the comparability clock again."*
- **(a) restate the threshold in CI terms** — inherits the defect RFC-050 just
  documented. The absolute columns carry **36–60% spread**, so any absolute
  threshold decides a crate split on runner luck whenever a reading lands near
  it. Replacing one uncalibrated number with another better-calibrated number
  does not fix that.
- **(b) keep the developer-machine framing and retire the CI proxy** — the
  remaining option, and the correct one on its own merits.

**Implement (b).** The threshold reads *"exceeds 30 seconds on a developer's
machine of average specs (8-core laptop, SSD, 16 GB RAM, no other heavy
work)"*. CI is not that machine, and since RFC-043 the column rebuilds iced's
entire closure from scratch. **The proxy was always measuring a different
quantity; RFC-043 only made it visible.**

So:

- **Keep the 30-second threshold unchanged.** It is not wrong — it was never
  the thing being measured.
- **Delete the claim that `build_widgets_ms` is the indicator 1 proxy**
  (`feature-gating-criteria.md:67`), replacing it with why it cannot be: wrong
  quantity, plus 36–60% runner spread (RFC-050).
- **State how indicator 1 *is* assessed** — by running the documented command on
  a developer machine — and that it is **currently unassessed**, with the date.
- Keep the CSV reference for context, clearly marked as *not* the assessment.

This is deliberately less satisfying than a live number, and it is honest:
better an indicator openly unassessed than one assessed by a proxy that
measures something else. If the owner prefers (a), that is their call to make —
but do not choose it yourself.

## 4. What to change

### 4.1 The status table — `feature-gating-criteria.md`

Re-date it (it reads **"Current status (snora 0.25.0)"**, ten minors stale) and
give **every row a measured value against its threshold**, per Q-2. Prose
verdicts alone are what let "Within budget" sit beside 96 s for ten minors.

Verified figures — **re-derive these, do not copy**:

| Indicator | Threshold | Current | Met? |
|---|---|---|---|
| 1. Compile time | 30 000 ms, developer machine | **unassessed** (§3) | unknown |
| 2. Binary size | 150 KB stripped | 46 464 B (~45 KB), 0.35.0 | no |
| 3. Heavy optional dep | >500 KB compiled crate | none | no |
| 4. Platform-specific dep | any system library | none | no |
| 5. Field requests | three independent applications | none received | no |

Confirm 3 and 4 against the current manifests rather than inheriting the old
row's "None" — `snora-style` arrived since it was written.

### 4.2 Indicator 2's stated method

It says to diff two `cargo build -p snora-example-hello` invocations. **That is
not how we measure.** Since RFC-041 it is three probe crates, recorded as
`widgets_diff_bytes`.

Correct the method to the probe-based one. **Keep the 150 KB threshold** — the
number is fine, only the instructions are stale.

### 4.3 State that the trigger has not fired

`design-decisions.md`'s `widgets` gate trigger is *"two of the five
feature-gating indicators are met."*

Record explicitly: **at most one indicator is met, indicator 2 is comfortably
under at ~45 KB against 150 KB, so the trigger has not fired** — with the
numbers, so the conclusion is checkable rather than asserted.

This is the sentence the document currently cannot produce, and it is the
point of the whole task. A reader today either trusts a table that contradicts
its own figure or trusts the figures with no statement about a second
indicator. Neither reaches the truth.

### 4.4 The release-checklist pointer

The table already says *"Re-evaluate at each release. Update this table as part
of the release process if anything changed."* Nothing in `release-process.md`
points at it, and it has not been updated for ten minors.

Add one checklist line. This is the RFC-059 pattern: the instruction exists, the
mechanism that fires it does not.

### 4.5 The accessibility-tree trigger — Q1, folded in

`design-decisions.md`'s register carries *"iced exposes an accessibility API"*
with the note that it is *"revisited deliberately rather than left to expire
quietly."* Nothing revisits it.

Attach the check, credit tekstide, and record the result with its date:

```bash
cargo tree -p snora --all-features | grep -i accesskit
```

**Verified 2026-08-18: empty. `iced_core` 0.14 has no accessibility module.**
The trigger has **not** fired and snora's stated position remains accurate.
Re-run it once to confirm before recording.

Note why this replaces the existing check: `design-decisions.md:487` greps
`crates/*/src/`, which detects **our** adoption rather than iced's readiness.
That grep stays where it is — it correctly supports the claim that snora has no
accessibility tree — but it is not the trigger's mechanism.

Add this check to the same checklist line as §4.4.

## 5. Explicit non-change scope

Do **not**:

- **Split any crate or add per-widget feature gates.** The trigger has not
  fired. This task reports; it does not act.
- **Implement Q-1 options (a) or (c)** (§3).
- **Add a measurement column**, or change how compile time or binary size is
  measured. RFC-050 settled the compile-time signal one release ago.
- **Change the 30-second or 150 KB thresholds.** Both stay.
- **Invent mechanisms for the register's judgement-based triggers.** Of nineteen,
  only four or five are mechanically checkable; the rest are judgement calls no
  command could answer, and manufacturing checks for them is busywork dressed as
  rigour.
- **Remove the `crates/*/src/` accessibility grep** (§4.5).
- Touch `design_overhead_ratio` or RFC-050's watch points.
- Modify any `.rs` file. `git diff --stat -- '**/*.rs'` must be empty.

## 6. Required tests

```bash
mdbook build docs && mdbook test docs
git diff --stat -- '**/*.rs'                 # MUST be empty
cargo tree -p snora --all-features | grep -i accesskit   # MUST be empty
```

No code changes, so the compile gates are unaffected — but the release checklist
runs them anyway.

## 7. Required evidence

- The re-derived figures for all five indicators, with the commands used.
- Before/after of the status table.
- The `feature-gating-criteria.md:67` proxy retirement.
- Indicator 2's corrected method.
- The not-fired statement with its numbers.
- The `release-process.md` checklist line.
- The `cargo tree … accesskit` run, showing empty.
- Confirmation that indicators 3 and 4 were re-checked against current
  manifests, not inherited.
- `git diff --stat -- '**/*.rs'` empty.

## 8. Acceptance criteria

RFC-062 §Acceptance criteria 1–7, with Q-1 bound to option (b) per §3.

The two most likely to be got wrong:

- **1** — no row may contradict its own threshold. Check every row, not just
  indicator 1.
- **6** — the not-fired verdict must carry its numbers. An unsupported "not
  fired" is the same failure as an unsupported "within budget".

## 9. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under
`.git-exclude/review-request/062-feature-gating-indicators-are-uncalibrated/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** whether the table can still be read as reporting a
live compile-time assessment after §3's proxy retirement. The failure mode is a
reader seeing a CSV reference beside indicator 1 and concluding it is the
measurement — which is exactly how the current contradiction survived ten
minors.
