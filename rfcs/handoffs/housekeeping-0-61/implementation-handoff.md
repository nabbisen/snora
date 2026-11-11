# Implementation handoff — 0.61 housekeeping: the build-cost ratio's +18%, and CI without Ubuntu's mirrors

**Ruled.** The architect's, at 0.61's scoping (2026-11-12), under the owner's instruction to
review the scope against the fundamental philosophy. Neither part changes snora's code or
behaviour. RFC-126 and RFC-127 are separate, and wait for the owner.
**Release target.** 0.61.0. **Two parts, one review.** It may start now.
**Branch.** `housekeeping-0-61`, off `main`. Land as `main` requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## Part 1 — measure the build-cost ratio's +18%. Report; change nothing

`compile-time.csv`, the 0.59.0 and 0.60.0 rows, both built with rustc 1.99.1 on `ubuntu-latest`:

| | `example_hello_ms` | `example_workbench_ms` | `design_overhead_ratio` |
|---|---|---|---|
| 0.59.0 | 112,881 | 5,368 | 0.047555 |
| 0.60.0 | 90,465 | 5,086 | **0.056221 (+18.2%)** |

`build-cost-budget.md`:
- the ratio is the trend signal;
- its investigate threshold is about 10%, read against a largest zero-code move of −6.35%;
- the rule: *"investigate a step-change in the ratio without a corresponding, explicable
  change to `example_workbench_ms`'s or `example_hello_ms`'s own dependency graph."*

No dependency changed in 0.60.0. The absolute columns fell 13 to 20%. **The workbench fell 5%
and hello 20%, so the ratio rose.**

**Answer, with numbers:**
1. **Is it the runner or the code?** Rebuild both examples at the `0.59.0` and `0.60.0` tags on
   one machine, cold, with `scripts/measure-compile-time.sh`. Run each tag **at least three
   times, interleaved** (A B A B A B), so that a drift in the machine falls on both. Report every
   run's ratio, not only a median.
   - **If the two tags' ratios overlap,** the move is the runner: say so, and the architect
     records it as the largest move with no cause in the code.
   - **If they separate,** go to 2.
2. **Which code?** The two inputs measure different things (`measure-compile-time.sh:96-106`):
   - `example_hello_ms` is a cold build of the whole graph;
   - `example_workbench_ms` cleans only `snora-example-design-workbench`, so it times that one
     crate. That is where snora's generic code is instantiated for the example's `Message`.

   0.60.0 added generic code to `snora::render` (`MenuKeys<Message>`, the anchor's reply). That
   is **one candidate, not a finding**. `cargo build --timings` on the workbench at each tag shows
   whether that crate's own compile grew, and by how much.
3. **Local numbers are not CI's** (`build-cost-budget.md`: the ratio is only comparable within one
   cold-build path). Compare tag with tag on your machine. Never compare your numbers with the
   CSV's.

If a cost in the code turns up, **report it.** Whether to act is the architect's to rule.

## Part 2 — the rendered jobs must survive a mirror outage

On 2026-11-11, `ci.yaml`'s "Install lavapipe" step (`:411`, `apt-get install -y
mesa-vulkan-drivers`) could not resolve Ubuntu's mirrors for about three hours. Four runs
failed with no snora test run, and the 0.60.0 release waited.

**Make the step independent of the mirrors' availability, as Workflow lint is of actionlint's
download** (`workflow-lint.yaml:48` onward: a cached, pinned archive, verified every run).

**Constraints:**
- **What runs is what was verified.** A cached `.deb` (or whatever you cache) is checked against
  a checksum recorded in the repository before it is installed. A mismatch fails the step, as
  for actionlint.
- **Pinned, not floating.** Today the job takes whatever `mesa-vulkan-drivers` the mirror
  serves. A pinned version makes the rendered output stable between runs, which parity relies
  on. **Say how a version is bumped,** and who does it: the commit that changes the checksum.
- **A fallback only when the cache is empty,** and the fallback is verified against the same
  checksum.
- **Both rendered jobs and parity pass unchanged** on the pinned version. If the pinned
  version's pixels differ from today's, stop and report: a baseline change is its own decision.
- **Document it** where the step is, as `workflow-lint.yaml` documents its cache.

**Show it can fail:** a run with the checksum altered fails the step (record the run, then
restore).

## Check

- `ci.yaml`'s other jobs and `release.yaml` are unchanged.
- `scripts/check-workflows.sh` and actionlint pass.

## Acceptance criteria

1. Part 1's table and answers in the review request
   (`.git-exclude/review-request/housekeeping-0-61/`), with the raw runs under `evidence/`.
2. Part 2's change, with the failing-checksum run and a green branch CI. Include **both**
   rendered jobs and parity.
3. All of `ci.yaml`'s Rust quality gate commands are untouched by this change, so the branch CI
   is the evidence.
