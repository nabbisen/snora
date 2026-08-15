# RFC 052 — The compile-time clean never invalidates release artifacts

**Status.** Implemented (v0.31.0)
**Tracks.** Measurement integrity (continues RFC-041, RFC-043, RFC-044).
**Blocks.** [RFC-050](./050-compile-time-measurement-is-runner-noise.md) — see
§"Why this lands first".
**Touches.** `scripts/measure-compile-time.sh`,
`docs/src/reference/build-cost-budget.md`,
`docs/src/contributing/api-freeze-review.md` (gate 9b).
**Release target.** 0.30.0.

## Summary

`measure-compile-time.sh` cleans with `cargo clean -p <packages>` before each
measurement, to make it a cold build of snora's own crates. **That command
cleans the dev profile only.** Four of the six measurements target `release`
or `release-baseline`, so their artifacts survive the clean untouched.

Two columns consequently measure **cargo's freshness check rather than a
build**. This is RFC-043's defect class again — a measurement that does not
measure what it claims — found while answering RFC-050's Q-1.

## Motivation

The script's own design note states the intent, and the intent is right:

> Uses `cargo clean -p <package>` for per-measurement cold builds so only the
> target package is rebuilt, not the entire iced transitive closure. This
> gives a stable, reproducible signal for Snora's own code without penalising
> CI with a full workspace clean.

Only the implementation is wrong. `cargo clean` documents `-r/--release` and
`--profile <NAME>` precisely because `-p` alone does not reach those profiles.

### The evidence

Artifacts survive the clean:

```text
$ cargo build -p snora-core --release
$ ls target/release/libsnora_core.rlib          → present
$ cargo clean -p snora-core -p snora-design -p snora-widgets -p snora
     Removed 160 files, 354.6MiB total          ← the dev profile
$ ls target/release/libsnora_core.rlib          → STILL PRESENT
```

And the consequence, run back-to-back on one tree:

```text
# with a correct --release clean
$ cargo build -p snora-widgets --features design --release
   Compiling snora-design … snora-core … snora-widgets
    Finished in 0.27s

# with the script's clean
$ cargo build -p snora-widgets --features design --release
    Finished in 0.07s          ← nothing recompiled
```

**Correction (2026-08-15, post-implementation):** an earlier revision said the
`build_engine_only` measurement "compiles **nothing at all**". That was true of
one local reproduction and does not generalise. The accurate account, from the
implementation evidence: the buggy clean spared exactly **`snora-core`** — the
only crate with no feature-set variation between measurement steps. `snora`
itself rebuilt in both cases, its fingerprint having already changed for
reasons unrelated to the clean. The measurement was a *partial* rebuild that
silently omitted one dependency, not a freshness check. See the RFC-052 note in
`docs/src/reference/build-cost-budget.md`, which carries the corrected
mechanism.

## Which measurements are affected

The script uses one profile-blind clean for six measurements spanning **three
profiles**:

| Measurement | Profile | Clean reaches it? |
|---|---|---|
| `check_workspace_ms` | dev (`cargo check`) | **yes** — correct today |
| `build_widgets_ms` | `release` | no — but cold anyway, see below |
| `build_engine_only_ms` | `release` | **no — measures nothing** |
| `example_hello_ms` | `release-baseline` | no — but cold anyway |
| `build_widgets_design_ms` | `release` | **no — measures nothing** |
| `example_workbench_ms` | `release-baseline` | no — partially genuine |

`build_widgets_ms` and `example_hello_ms` are cold **by accident**: each is
the first build in its profile in a job that runs without a dependency cache
(uncached deliberately, RFC-043). They are genuine measurements that happen
not to depend on the broken clean, and they stay genuine after the fix.

`example_workbench_ms` does real work despite the no-op clean, because the
workbench pulls `design` features into `release-baseline` that `example_hello`
did not build. Its separate clean —
`cargo clean -p snora-example-design-workbench`, also profile-blind — has the
same defect.

## What is not claimed

**The magnitude of the error is unknown, and this RFC does not assert one.**

A properly-cleaned engine-only rebuild measured 248 ms locally against the
CSV's 323–421 ms — the same order. snora's crates are small (`snora-core` has
no dependencies at all), so corrected values may land near current ones while
measuring something entirely different. A local attempt to compare the two
forms of `build_widgets_design_ms` was contaminated by its own sequencing and
is not relied on here.

The defect stands on the `Compiling` lines above, not on a delta. RFC-043 was
raised on the same basis: its numbers also looked plausible.

## Design

### Clean per profile, not per package only

`measure_ms` cleans all three profiles for snora's own crates before each
measurement:

```text
cargo clean -p <snora crates>                              # dev
cargo clean -p <snora crates> --release                    # release
cargo clean -p <snora crates> --profile release-baseline   # release-baseline
```

Cleaning all three unconditionally, rather than passing the target profile per
call site, is deliberate: it removes the coupling between a measurement and
its profile, so a future measurement cannot be added with the wrong one. The
cost is negligible — it invalidates only snora's crates, never iced's closure,
which is the property the original design note cares about.

The separate `snora-example-design-workbench` clean gets the same treatment.

### What the columns then mean — and the trap for RFC-050

After the fix the columns are **still heterogeneous**, and this must be
recorded rather than assumed away:

- `build_widgets_ms` remains the first release build in an uncached job, so it
  measures **iced's closure plus snora's crates**.
- `build_engine_only_ms` and `build_widgets_design_ms` run after it, so they
  measure **snora's crates only**, with iced warm.

These are different quantities that happen to share a unit. **Any ratio must
pair like with like**, which is precisely where RFC-050's
`widgets_design_ratio` fails: `build_widgets_design_ms / build_widgets_ms`
divides a snora-only rebuild by an iced-plus-snora cold build. That was
already wrong before this defect was found; the defect made it worse by
turning the numerator into a freshness check.

## Why this lands first

RFC-050 selects ratios **on the basis of what the columns measure**. Two of
those columns are about to change meaning, so implementing RFC-050 now would
build a trend signal on quantities that are about to be redefined.

RFC-050's central finding is unaffected and carries over: the common-mode
runner-variance analysis was computed from `check_workspace_ms`,
`build_widgets_ms` and `example_hello_ms`, all genuine, as was the
0.28.0 → 0.28.1 documentation-only evidence. It is the ratio *selection* that
must be re-derived on post-fix data.

## The cost, stated plainly

**This is a third methodology discontinuity**, after RFC-043 and RFC-044.

- Rows before the fix and rows after it are **not comparable** for
  `build_engine_only_ms` and `build_widgets_design_ms`.
- Gate 9b's closure condition — ≥2 comparable post-change data points —
  **resets again**. Gate 9b moves further away before it moves closer.

That is worth paying. The alternative is a budget that keeps producing numbers
nobody can act on, which is the situation RFC-041 was raised to end.

## Non-goals

- **No back-filling or editing of historical rows.** Append-only, RFC-041 N-1.
  The discontinuity is recorded, not repaired.
- **No change to the uncached-CI policy** (RFC-043). Its rationale in
  `build-cost.yaml` stands.
- **No new columns.** That is RFC-050's job, after this.
- **No CI failure gate**, and no change to the absolute 30 s watch point.
- **No change to binary-size measurement.** Gate 9a is satisfied and unaffected.
- **No attempt to make `build_widgets_ms` comparable** to the snora-only
  columns by pre-warming iced. That is a design question for RFC-050's
  re-derivation, not a bug fix.

## Open questions

**Q-1 — Should `check_workspace_ms` keep its dev-profile clean only?**
It is correct today by luck rather than design — `cargo check` targets dev, and
the clean happens to match. Under the proposed all-profiles clean it stays
correct. Confirm that cleaning release profiles before a `cargo check` has no
side effect on the check's own timing.

**Q-2 — Does `example_workbench_ms` change materially?**
Its clean is currently a no-op, but real compilation happens anyway via the
feature difference. Report the before/after so the discontinuity note can say
which columns actually moved rather than assuming all four did.

## Acceptance criteria

1. `measure_ms` cleans dev, `release` and `release-baseline` for snora's own
   crates before every measurement; the workbench clean does the same.
2. A local run demonstrates that `build_engine_only` and
   `build_widgets_design` now show `Compiling` lines for snora's crates —
   evidence, not assertion.
3. `build-cost-budget.md` gains a **fourth data-integrity note** recording the
   defect, the evidence, and the discontinuity, in the style of the existing
   three.
4. Q-2 is answered: which columns moved, and by how much.
5. Gate 9b's row records that its clock reset, and why.
6. No historical row edited.

## Compatibility and security

**Compatibility.** No library change, no API, no CSV schema change. The values
in future rows change meaning for two columns; that is the point, and it is
recorded as a discontinuity.

**Security.** No new data flow, dependency, or integration.

## Release implications

**0.30.0.** No migration guide — nothing downstream consumes this file. A
`CHANGELOG.md` entry under **Fixed**, naming the two affected columns, so the
discontinuity is discoverable from the changelog and not only from the budget
page.
