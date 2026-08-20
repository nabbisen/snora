# RFC 078 — Measure what iced's `advanced` feature costs, before deciding whether trapping is worth it

**Status.** Proposed
**Tracks.** Measurement / 1.0 readiness.
**Ruled by** the owner, 2026-08-20: RFC-060 Q-1 is answered by **measuring**,
not by closing or holding.
**Touches.** measurement only — **no crate change ships from this RFC.**
**Release target.** None. The measurement lands as a record; any decision that
follows is a separate RFC.

## The question being costed

RFC-060 Q-1: **does full modal focus trapping justify enabling iced's
`advanced` feature?** Trapping needs `focusable::find_focused()`, which is
reachable only with `advanced`. Nothing in snora uses `iced::advanced::` today —
verified, zero hits across all five crates.

`advanced` turns on `iced_core/advanced` and `iced_widget/advanced`.

## Why this needs a design and not just a stopwatch

**Our measurement floors are known and both are large enough to swallow this.**

- **Compile time:** the `design_overhead_ratio`'s noise floor is ~4.4%, measured
  on a zero-code release, and two further zero-code controls this month moved it
  **−4.05%** and **+2.5%**. A single CI sample cannot resolve anything under
  roughly 10%. The absolute millisecond columns are worse — runner-dominated,
  and RFC-050 records them as raw record, never a trend.
- **Binary size:** the ±128 B band, established by two zero-code releases that
  moved *within* it. Anything smaller than that is not visible.

**So the measurement must be designed to beat those floors, or must say it
cannot.** This project has twice declared a gate satisfied on data that did not
support it (RFC-041, and the 9a/9b split that followed). Producing a number
here that is really noise would be a third.

## The design constraint that makes this tractable

**Measure locally, A/B, on one machine, back to back — not across CI runs.**

CI's dominant error term is *between-runner* variance: different CPU models and
neighbour load between jobs. That is what makes the absolute compile-time
columns unusable. An A/B on one machine, same toolchain, same warm state,
alternating the two arms, removes exactly that term. It is a worse measurement
of *absolute* cost and a much better measurement of a *difference*, which is the
only quantity this RFC needs.

Binary size does not have this problem — it is deterministic given a toolchain —
so the A/B there is simply a build with and without the feature.

## Open questions

**Q-1 — which arm is "with"?** `advanced` on the workspace `iced` dependency
affects every crate that depends on iced (`snora`, `snora-widgets`,
`snora-style`). **Suggest measuring the honest arm: the feature as it would
actually be enabled**, not a narrower experiment that flatters the result.

**Q-2 — which probe binaries?** `examples/size_probe_{engine,widgets,design,design_engine}`
already exist and are what `binary-size.csv` measures. **Suggest all four**, so
the answer is expressed in the same units as the budget it would be judged
against.

**Q-3 — what counts as "affordable"?** Decide the threshold **before** seeing
the number, and write it down here. Otherwise the result argues for itself.
A starting proposal, to be ruled on: **under 5% of the 150 KB budget (7.5 KB)
on `widgets_diff_bytes` is affordable; over 15% is not; between is a judgement
that needs the trapping design in hand.** This RFC does not need the thresholds
to be right — it needs them fixed in advance.

**Q-4 — is binary size even the binding cost?** Enabling `advanced` on our
`iced` dependency means **feature unification gives every consumer's iced
`advanced` too**, whether they wanted it or not. That is an API-surface and
supply-chain consequence, not a size one, and it may dominate the decision
regardless of what the bytes say. **Measure the bytes; do not let them settle a
question they do not address.**

## Non-goals

- **No trapping implementation.** This RFC measures a feature flag's cost. If
  the answer is "affordable", trapping is a separate RFC, and it is bound by
  `design-decisions.md`: it must arrive as a **new optional `Dialog`/`Sheet`
  field** under RFC-011-C, never as a behaviour change to existing fields.
- **No feature enabled in a shipped release.** The measurement arm is local and
  reverted.
- **No CSV row.** `binary-size.csv` and `compile-time.csv` record releases. A
  measurement experiment is not a release, and RFC-041 N-1 forbids editing or
  back-filling those files.

## Acceptance criteria

1. Q-3's thresholds ruled and written down **before** any number is taken.
2. `widgets_diff_bytes`-equivalent measured for all four probes, both arms,
   same toolchain, deterministic — with the raw byte counts stated.
3. Compile-time A/B attempted on one machine with alternating arms and repeats;
   **the result stated with its own uncertainty**, including "below what this
   method can resolve" if that is what it is.
4. Q-4's feature-unification consequence stated as a finding in its own right,
   independent of the byte count.
5. The `advanced` arm is reverted; `git status` clean; no CSV touched.
6. A recommendation, with the thresholds from criterion 1 applied — and if the
   answer is "between", it says so rather than rounding to a verdict.

## Compatibility and security

Nothing ships. **Compatibility.** None. **Security.** None.
