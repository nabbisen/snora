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

## Part C (added 2026-11-17, at Part B's landing) — run each sweep once per combination

**What Part B's landing showed:**
- `ci.yaml` still takes **26 min 19 s** of wall-clock time, and **the Rust quality gate is all of it
  (26 min 16 s).** The rendered jobs, which run beside it, take 13 min 33 s (wgpu) and 9 min 26 s
  (tiny-skia).
- Inside the quality gate, two steps take almost all of the time:
  - "Test engine, all features" (663 s);
  - "Test engine, design without lucide-icons" (643 s).
- They run the drawn-contrast sweeps again on tiny-skia, which the rendered jobs already run.
- Part B cut the wgpu job, but the wall clock is set by the quality gate.

**The rule:** every combination of renderer and feature set that is swept today is still swept,
**exactly once,** on every push to `main`. No coverage is lost, and nothing runs twice.
1. **Measure first:** which tests take the two steps' time (the sweeps, by name, and anything else
   over 30 s), and which combinations each step and each rendered job covers today. The
   no-`lucide-icons` build draws `×` where the others draw lucide's `X`, so its sweep is a coverage of
   its own.
2. **Propose,** then do it, in one commit: where each sweep runs, and how the others skip it. Use an
   explicit mechanism a reader can find (an `#[ignore]` with `--include-ignored` in the job that owns
   it, or a named feature or environment variable), stated in the test's doc and in `ci.yaml`'s
   comment.
   - The steps that skip a sweep must **still compile it** (`--no-run` or the build itself), so a
     feature-specific break is caught.
3. **Show the bar is met:**
   - a table of combination against job, before and after, with every combination present once
     after;
   - ci.yaml's wall clock before and after, from main's runs.

**Acceptance:** the table; the time; every plant of RFC-131, RFC-133 and RFC-135 still failing where
its test now runs (re-run the three breakage scripts); the gates.
