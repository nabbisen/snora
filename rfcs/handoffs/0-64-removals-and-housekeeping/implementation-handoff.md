# Handoff — 0.64: the F6 removal, and two housekeeping measurements

**Scope.** Ruled at 0.63's scoping and cut (ROADMAP "Next — 0.64.0"). RFC-126 deprecated these types
in 0.61.0 with "removed in 0.64.0". The two measurements were set at RFC-135 Unit 1's review and the
0.63.0 post-tag.
**Release target.** 0.64.0, a breaking release, with RFC-133.
**Where.** On `main`, by the one-`main` route. **Part B's probes are scratch:** described and not
committed, unless a change is accepted.

## Part A — remove the deprecated F6 types (RFC-126 R-4)

**What goes:** `snora::focus::next_zone`, `FocusZone` and `ZonePresence`, and their re-exports.
They are deprecated `since = "0.61.0"` in `crates/snora-core/src/focus.rs:45, :77, :187`.
- **Find every reference mechanically:**
  `git grep -n -E "next_zone|FocusZone|ZonePresence"` over `crates`, `examples` and `docs/src`,
  outside migration guides and the CHANGELOG (about 120 lines in 11 files today).
  - **Code:** removed.
  - **Tests of the removed items:** removed, each named in the request.
  - **Tests that use them as a fixture:** moved to `focus_zone`.
  - **Book pages:** listed for the architect, not edited.
- **Nothing that is not deprecated changes.** `cycle_zones`, `Cycle` and `focus_zone` stay as they
  are.
- **A test that the items are gone** is not needed: the compiler is that test. Say so.
- **CHANGELOG `[Unreleased]`, Removed:** the three, with the two-line replacement from the 0.60 →
  0.61 guide.

## Part B — measure, then decide (no change lands unless it passes its bar)

**B-1. The drawn sweeps' CI time** (209 s on tiny-skia, 635 s on wgpu at 0.63). Measure two
changes, each alone and both together:
- **(i) draw each frame once,** per scene, preset, surface, state and direction, and sample every
  cell from it;
- **(ii) read boundaries at 2x** with scale-relative sampling offsets, instead of 4x.

**The bar:** over every cell of today's two sweeps, on **both** renderers, **no verdict changes and
no measured ratio moves by more than the sweep's own slack** (0.05 text, 0.02 non-text).
- Report the time saved by each.
- Land (i) if it passes. **Land (ii) only if it passes and (i) alone does not bring wgpu under five
  minutes.** A cheaper read that gives up accuracy is the wrong trade.
- Either change must keep every plant of RFC-131 and RFC-135 failing: re-run both breakage scripts.

**B-2. The build-cost ratio** (`example_workbench_ms / example_hello_ms`). It read 0.05578 at
0.63.0, +16% on 0.62.0, with the same compiler; the last four rows span 0.0479 to 0.0562.
- **Measure locally,** five runs each, on the 0.62.0 tag and the 0.63.0 tag. Use the commands
  `build-cost.yaml` runs and the same toolchain. Report the mean and the spread.
- **Say whether 0.63 moved it** beyond the run-to-run spread. If it did, say which crate's change
  (`cargo build --timings` is enough).
- **Say whether the 10% noise floor fits a build this short** (about 7 s), from the spread you
  measured. The architect rules on the floor.

## Acceptance criteria

1. **Part A:** every reference handled as above, and the gates and the Docs job green.
2. **Part B:**
   - the measurements, with their method;
   - a change lands only if it passes its bar, with the breakage re-runs as evidence.
3. Named `cp` backups and `sha256sum -c` for any plant; never `git checkout` or `git restore`.
