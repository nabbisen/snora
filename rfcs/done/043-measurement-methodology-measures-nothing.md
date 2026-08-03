# RFC 043 — The budget measurements do not measure what they claim

**Status.** Implemented (v0.26.0)
**Tracks.** Measurement integrity, follow-on from RFC-041. Bears on 1.0
gate 9 and on DEC-11 (`design` opt-in).
**Touches.** `examples/size_probe_*/src/main.rs`,
`scripts/measure-binary-size.sh`, `scripts/measure-compile-time.sh`,
`.github/workflows/{binary-size,build-cost}.yaml`,
`docs/src/reference/{binary-size-budget,build-cost-budget}.md`,
possibly `docs/src/contributing/feature-gating-criteria.md`.

## Summary

RFC-041 fixed the measurement workflows so they finally run on release
tags. The first real data point arrived with 0.25.3 — and it shows the
methodology itself is broken. The binary-size probes report the marginal
cost of the `widgets` feature as **0 bytes**, and the build-cost numbers
are an order of magnitude too fast to be cold builds.

We fixed the plumbing and discovered the instrument was never calibrated.

## Evidence

### E-1 — The first real binary-size row

```csv
version,engine_bytes,widgets_bytes,widgets_diff_bytes,design_bytes,design_diff_bytes,...
0.25.3,15687888,15687888,0,15688016,128,rustc_1.97.1,Linux,2026-08-02
```

`widgets_diff_bytes = 0`. `design_diff_bytes = 128`. Enabling an entire
prefab widget crate is measured as costing nothing.

### E-2 — Why: the probes never use the features

All three probe sources are **byte-identical**
(`diff` confirms engine == widgets == design), 39 lines each, and their
entire snora surface is:

```rust
use snora::{AppLayout, render};
...
render(AppLayout::new(body))
```

No `snora::widget::*` call. No `snora::design::*` call. The probes' own
doc comment states the intent:

> the point is that all three probes … contain the same application code
> so that `widgets_diff` and `design_diff` are purely the marginal cost of
> the respective feature, not of different application logic.

The intent is sound; the implementation inverts it. Identical code that
never *calls* the feature measures the marginal cost of **compiling but
not using** it — and Rust's dead-code elimination makes that approximately
zero, which is exactly what the data shows.

### E-3 — Build-cost numbers are warm, not cold

```csv
version,check_workspace_ms,build_widgets_ms,build_engine_only_ms,...
0.19.1,56150,96000,330,153000,...
0.25.3,5455,353,323,5049,...
```

`check_workspace_ms` fell from 56 s to 5.5 s, and `build_widgets_ms` from
96 s to 0.35 s. Those are not improvements; the workflows use
`Swatinem/rust-cache`, and `measure-compile-time.sh` cleans only snora's
own packages, so the expensive dependency (`iced`) is restored from cache
and never rebuilt. The numbers measure an incremental rebuild of four
small crates.

### E-4 — Schema drift

`runner_os` reads `Linux` in the new row and `ubuntu-latest` in every
prior row. Cosmetic, but it breaks grouping in any trend analysis.

## Consequences

1. **Gate 9 cannot be re-satisfied on this methodology.** RFC-041 reopened
   it pending "≥2 real post-fix data points"; the first such point is not
   a real measurement. Gate 9 stays ⬜ regardless of how many rows accrue.
2. **`feature-gating-criteria.md` indicator 2 can never fire.** It triggers
   on binary-size drift > 150 KB. A measurement pinned near 0 by
   construction will never reach it.
3. **DEC-11's binary-size justification is unsupported — and may be
   backwards.** See below. This is the finding that matters most.

## The DEC-11 implication

`design` is opt-in, justified by "binary-size / build-cost discipline"
(DEC-11, and RFC-037 N-2 restates it). E-1/E-2 suggest that justification
does not hold in the form it is stated: **an enabled-but-unused feature
appears to cost approximately nothing in binary size**, because unused
code is stripped at link time. The cost materialises only when an
application actually calls the feature — at which point the application
chose the cost.

If that holds under a corrected methodology, then binary size is not a
reason to keep `design` opt-in. The remaining arguments — compile time,
dependency surface, API surface discipline, and not silently restyling
existing applications — are still real, and RFC-037's gating invariant
rests on the last of those, not on bytes.

**This RFC does not propose changing DEC-11.** It proposes measuring
properly first, then revisiting the justification with evidence. Deciding
default-on remains the owner's call and is explicitly out of scope here.

## Proposed changes

### C-1 — Make the probes exercise their features

Replace "identical application code" with **"identical application code
plus a minimal, representative use of the feature under test"**:

| Probe | Should additionally |
|---|---|
| `size_probe_engine` | nothing — baseline (current source is correct) |
| `size_probe_widgets` | call a representative `snora::widget::*` builder (e.g. `app_header` + `app_side_bar`) |
| `size_probe_design` | the widgets set **plus** a representative `snora::design::*` primitive and style bridge call |

The diffs then measure "what a typical adopter of this feature pays",
which is the number the budgets were created to track. Update DEC-14's
description and the probes' doc comments to state the corrected intent —
the current comment is the trap that produced this.

### C-2 — Make build-cost measurement genuinely cold

Either disable dependency caching in `build-cost.yaml`, or clear the
target directory (not just snora's packages) before the timed section.
Cold builds are the point; a cached number is not comparable to anything.
Expect run times to rise from ~1 min to several — that is correct, and
RFC-041's rationale for keeping measurement out of the PR path already
accounts for it.

### C-3 — Stabilise `runner_os`

Emit a consistent value (`ubuntu-latest`) so rows group.

### C-4 — Annotate 0.25.3's row

Per RFC-041 N-1, do **not** delete or rewrite it. Annotate the budget docs
to record that 0.25.3 is the first row from the fixed workflows *and* the
last row from the broken methodology, and is not comparable to what
follows.

## Non-goals

- **N-1. No back-filling.** Unchanged from RFC-041.
- **N-2. No change to DEC-11 or the `design` default.** Measure first.
- **N-3. Not a performance-optimisation exercise.**

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Corrected probes still show a small diff, and the result is read as "features are free" | Medium | Medium | Report the number with its methodology; a small honest number is still a measurement |
| Cold build-cost runs become slow enough to be skipped | Medium | Medium | They run on tags only, not per PR |
| Changing probes invalidates comparison with 0.25.3 | Certain | Low | Intended, and recorded per C-4 |

## Acceptance criteria

1. Each probe exercises the feature it measures; sources are no longer
   byte-identical, and the doc comments say why.
2. `widgets_diff_bytes` and `design_diff_bytes` are non-trivial and
   reproducible across two consecutive tags.
3. Build-cost numbers are cold and comparable in magnitude to the
   pre-cache era.
4. `runner_os` is stable across rows.
5. Budget docs record the 0.25.3 discontinuity; no row deleted or edited.
6. Gate 9 remains ⬜ until two post-fix rows exist under the corrected
   methodology.

## Release implications

Patch-level; CI, examples, and docs only. No crate content change. Gate 9
stays open longer than RFC-041 anticipated — which is the honest outcome,
not a regression.
