# RFC 062 — The feature-gating status table contradicts its own threshold

**Status.** Done — shipped in v0.36.0 (2026-08-18). Handoff:
[`handoffs/062-…`](../handoffs/062-feature-gating-indicators-are-uncalibrated/implementation-handoff.md)
**Tracks.** Measurement integrity / governance. Continues RFC-041, RFC-043,
RFC-050.
**Touches.** `docs/src/contributing/feature-gating-criteria.md`,
`docs/src/contributing/design-decisions.md`,
`docs/src/contributing/release-process.md`, `CHANGELOG.md`. **No code.**
**Release target.** 0.36.0, alongside RFC-061 (documentation only in itself).

## Summary

`feature-gating-criteria.md` decides when snora splits `widgets` into per-widget
feature gates. Its status table says:

> **1. Compile time** | **Within budget.** `build_widgets_ms` **≈96 s** on
> ubuntu-latest (v0.19.1 baseline)

The threshold three sections above it is **30 seconds**.

The row calls a figure **3.2× over the rule** "within budget", in the document
that governs the decision. That is not staleness — it is a contradiction on the
face of the rule, and it has been there since v0.19.1.

The cause is not carelessness. **RFC-043 changed how compile time is measured
and nobody recalibrated the indicators that consume it.** This RFC fixes the
contradiction, restates the thresholds against the methodology actually in use,
and records what the indicators now say.

## The evidence

### The contradiction

| | value |
|---|---|
| Indicator 1 threshold (`feature-gating-criteria.md:54`) | **30 000 ms** |
| Status table's own cited figure | **≈96 000 ms** |
| Status table's verdict | **"Within budget"** |
| Latest measured (`0.35.0`) | **84 293 ms** — 2.8× over |

Every row since 0.26.0 reads 2.2×–3.5× over. Twelve consecutive releases.

### The table is ten minors stale, and says so

It is headed **"Current status (snora 0.25.0)"**. Immediately below it:

> Re-evaluate at each release. Update this table as part of the release process
> if anything changed.

That instruction has not been followed for ten minor releases, and nothing in
`release-process.md` points at it — the same shape RFC-059 fixed for
documentation rules by adding a checklist pointer.

### Why the calibration broke

`build_widgets_ms` at v0.25.3 was **353 ms**. At 0.26.0 it was **101 637 ms** —
288× larger, one release later, with no corresponding change to snora.

RFC-043 removed the dependency cache so the measurement rebuilds iced's entire
transitive closure from scratch. That was correct: it made the measurement
honest. But the 30-second threshold was written against the *cached* era, and
against *"a developer's machine of average specs building `snora-widgets`
cold"* — which is not what the column measures now.

**The threshold and its proxy measure different quantities**, and have since
RFC-043.

### Indicator 2 has the same disease, less visibly

Its stated method is:

```bash
cargo build --release -p snora-example-hello
cargo build --release -p snora-example-hello --no-default-features
```

**That is not how we measure it.** Since v0.25 (RFC-041) binary size comes from
three probe crates, recorded as `widgets_diff_bytes`. The indicator's stated
command and the project's actual measurement have been different for ten minors.

Its *number* is fine — see below. Its instructions are not.

## What the indicators actually say, measured

| Indicator | Threshold | Current | Met? |
|---|---|---|---|
| 1. Compile time | 30 000 ms | 84 293 ms (0.35.0) | **apparently yes** — but see below |
| 2. Binary size | 150 KB stripped | **46 464 B (~45 KB)** | no |
| 3. Heavy optional dep | >500 KB compiled crate | none | no |
| 4. Platform-specific dep | any system library | none | no |
| 5. Field requests | three independent applications | none received | no |

**Indicator 5 has stronger evidence than "none received", added 2026-08-18.**
Consumer replies confirm that **two of the three adopters compile zero prefab
widgets**, and one — arama, at 0.33.0 — **removed `snora-widgets` from its build
graph entirely** and verified it was gone rather than trusting the feature flag.
apimokka has had zero `snora::widget::*` call sites since 0.25.2.

That is not a request for per-widget gates. It is the opposite: **the coarse
gate is doing its job.** RFC-055 made `design` and `widgets` independent at
0.32.0, and the first consumer to want engine-plus-design-without-widgets simply
took it. A consumer who can already opt out at the granularity they need does
not ask for a finer one.

Record this under indicator 5 rather than leaving a bare "none" — a "no" with
evidence behind it is what makes the not-fired verdict checkable.

The `widgets` gate's own reconsideration trigger, in `design-decisions.md`, is
**"two of the five feature-gating indicators are met."**

**So the trigger has not fired.** At most one indicator is met, and indicator 2
— the natural corroborator — is at 45 KB against a 150 KB bar, comfortably
under.

That is the reassuring answer, and it is exactly what the document cannot
currently deliver: today a reader either believes the status table (indicator 1
"within budget", contradicting its own figure) or believes the numbers
(indicator 1 met, and no statement about whether a second is). Neither path
tells them the truth, which is *one indicator possibly met, one clearly not, no
reconsideration due.*

## Scope

1. **Fix the contradiction.** Indicator 1's status must agree with its own
   numbers, whatever the resolution in Q-1 is.
2. **Recalibrate or re-scope indicator 1** — Q-1.
3. **Correct indicator 2's stated measurement method** to the probe-based one
   actually in use, keeping the 150 KB threshold and recording the current 45 KB
   reading.
4. **Re-date and re-populate the status table** for the current release, with
   every indicator's measured value, not a prose verdict alone.
5. **Add a `release-process.md` checklist line** pointing at the table, so it is
   re-evaluated as the instruction already says it should be. RFC-059
   established this pattern; it is the mechanism that makes the instruction fire.
6. **State that the two-of-five trigger has not fired**, with the numbers behind
   it, so the conclusion is checkable rather than asserted.
7. **Attach a check to the accessibility-tree trigger** — see below.

## Folded in: the accessibility-tree trigger gets a mechanism

`design-decisions.md`'s register carries:

> No interim accessibility tree; ABDD bounded to layout + visual | Accepted |
> **iced exposes an accessibility API**

with the note that it is *"revisited deliberately rather than left to expire
quietly."* Nothing revisits it.

**Checked, 2026-08-18: the trigger has not fired.**
`cargo tree -p snora --all-features` contains no `accesskit`, and `iced_core`
0.14 has no accessibility module. snora's stated position remains accurate.

tekstide (Q1) offered the mechanism, and theirs is better than ours: our only
existing check greps `crates/*/src/`, which detects *our* adoption rather than
iced's readiness. Theirs watches the dependency graph.

**Scope:** attach `cargo tree -p snora --all-features | grep -i accesskit` to
that trigger, record today's empty result and its date, and add it to the same
release-checklist line as item 5. Credit tekstide.

This is a two-line addition riding along, not a justification for its own RFC.
Of the register's nineteen triggers, only four or five are mechanically
checkable at all; the rest are judgement calls no command could answer. **This
RFC does not propose a mechanism for every trigger** — that would be busywork
dressed as rigour.

## Non-goals

- **No crate split, and no per-widget feature gates.** The trigger has not
  fired. This RFC reports; it does not act.
- **No change to how compile time or binary size is measured.** RFC-050 just
  settled the compile-time trend signal; changing the measurement again would
  reset the comparability clock a fourth time.
- **No mechanism for judgement-based triggers.**
- **No change to the `design_overhead_ratio` watch points** (RFC-050).
- **No code.**

## Open questions

**Q-1 — what should indicator 1 become?** Three shapes, and the choice is a
judgement about what the indicator is *for*:

- **(a) Restate the threshold in CI terms.** Pick a number calibrated against
  the post-RFC-043 uncached measurement. Simple, but the resulting figure
  measures "rebuild iced from scratch", which is not the developer-experience
  cost the indicator was written to protect.
- **(b) Keep the developer-machine framing and stop using the CI column as its
  proxy.** Honest about what the rule means; leaves the indicator unmeasured
  unless someone runs it locally.
- **(c) Measure the marginal cost instead of the absolute.** What `widgets`
  *adds* over an engine-only build is the quantity a feature-gating decision
  actually turns on, and it is closer to what `widgets_diff_bytes` does for
  indicator 2. Requires deciding whether an existing column expresses it.

**Resolved during handoff (2026-08-18): option (b).** Checking closed the other
two rather than weighing them.

- **(c) is blocked.** No existing column expresses the marginal compile cost of
  `widgets` over an engine-only build — `build_widgets_ms` includes iced,
  `build_engine_only_ms` builds against a warm graph. It needs a new column, and
  RFC-050's non-goals forbid one: *"Adding one would reset the comparability
  clock again."* Coherent, and unavailable at acceptable cost.
- **(a) inherits the defect RFC-050 documented.** The absolute columns carry
  36–60% spread, so *any* absolute CI threshold decides a crate split on runner
  luck near the line. Recalibrating the number does not fix the instrument.
- **(b) is right on its own merits**, not merely by elimination. The threshold
  says *"a developer's machine of average specs"*; CI is not that machine, and
  since RFC-043 the column rebuilds iced's whole closure. The proxy was always
  measuring a different quantity — RFC-043 only made it visible.

So: keep the 30-second threshold, retire the proxy claim, state how the
indicator *is* assessed, and record it as currently unassessed. Less satisfying
than a live number, and honest — an indicator openly unassessed beats one
assessed by an instrument measuring something else.

**Q-2 — should a "measured, not met" verdict carry its numbers?** Suggest yes,
and that this becomes the table's format: every row states its measured value
and its threshold, so a future reader can check the verdict rather than trust
it. The current table's prose verdicts are precisely what allowed "within
budget" to sit beside 96 s for ten minors.

## Acceptance criteria

1. No row of the status table contradicts its own indicator's threshold.
2. Indicator 1 resolved per Q-1, with the reasoning recorded.
3. Indicator 2's stated measurement method matches the probe-based one in use;
   150 KB threshold unchanged; current ~45 KB reading recorded.
4. The table is re-dated to the current release and every row carries a measured
   value against its threshold.
5. `release-process.md` has a checklist line re-evaluating the table.
6. The two-of-five trigger is stated as **not fired**, with the numbers.
7. The accessibility-tree trigger carries `cargo tree … | grep -i accesskit`,
   today's empty result, its date, and credit to tekstide.

## Compatibility and security

**Compatibility.** Documentation only. No API, no rendering, no measurement
methodology change, no gate rows moved.

**Security.** None.
