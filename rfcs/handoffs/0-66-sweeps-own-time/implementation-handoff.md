# Handoff — 0.66 housekeeping: the drawn sweeps' own time

**Scope.** Ruled at RFC-134 Unit 1's review (addendum, question 2): try it, measured first.
**Release target.** 0.66.0.
**Where.** On `main`, by the one-`main` route. **The probe is scratch.** A change lands only if it
passes the bar.

## Why

Each sweep is one test that draws its cells in sequence, so test threads do not help it. The
tiny-skia sweeps' jobs are now the longest jobs in CI: 12 to 14 minutes, against 4 to 7 for the
others (0.65).

## Measure first

- **Where each sweep's time goes, on both renderers:**
  - the frames, by scale (2x, 4x);
  - the reads;
  - taking a renderer from the pool.
- **The runner's cores:** four on `ubuntu-latest`. Say what you measured it on.

## The change to try

**Draw the six presets on parallel threads**, with each thread taking its own renderer from the pool
(`Lent`, Part B's).
- **The pool is a `static` that is never dropped,** because a thread-local one aborted at exit on
  wgpu. Keep that, and say how it holds with more than one thread.
- **The order of the reported failures must stay deterministic,** sorted, so a failure reads the same
  every run.

## The bar (all of it, or the change does not land)

1. **Every cell of every sweep identical,** to the last digit, before and after, on tiny-skia and on
   wgpu: `nontext_sweep`, `text_sweep`, `form_unstyled_drawn`, and the no-lucide combination's.
2. **Every plant still failing:** re-run the breakage scripts of RFC-131, RFC-133, RFC-134 and
   RFC-135 (RFC-131's F05 from Part B's patched copy).
3. **The time saved, per sweep and per renderer,** locally and from main's run after landing. Report
   the **longest job**, not the wall clock: the queue is account-wide.

## Acceptance criteria

The bar above; the gates (all of `ci.yaml`'s commands, `scripts/rendered-targets.sh check`, and
`scripts/check-workflows.sh` with shellcheck); named `cp` backups and `sha256sum -c` for any plant;
never `git checkout` or `git restore`.
