# Implementation handoff — RFC-108 R-3 and R-4: both renderers in CI, and a parity check

**RFC.** `rfcs/accepted/108-the-default-renderer-is-never-tested-and-contrast-is-never-read-from-pixels.md`
**Follows** the first handoff (Unit 0, R-1, R-2), whose package is accepted.
**Release target.** 0.53.0 if it is ready before the cut; it does not hold the cut.

**Unit 0 ruled, 2026-10-19: feasible.** 5/5 runs succeeded on lavapipe, with no
crashes, flakes or wgpu-only failures. The suite takes 194–293 s, against
tiny-skia's 23–35 s (about 8.8×). Under Q-1 and Q-2: **R-3 goes ahead, on
lavapipe.**

**Touch list.**
- `.github/workflows/ci.yaml`;
- `crates/snora/tests/drawn_contrast.rs` (parity output);
- `crates/snora/tests/common/mod.rs`, if the output helper lives there;
- new `scripts/check-renderer-parity.py`;
- `scripts/README.md`;
- `CHANGELOG.md` (`[Unreleased]`).

**Not** any style function or colour.

## R-3 — a `rendered-wgpu` job

- **What it runs.** `ubuntu-latest`, install `mesa-vulkan-drivers`, set
  `ICED_TEST_BACKEND=wgpu`, then run
  `cargo test -p snora --all-features --tests -- --test-threads=1`.
  - **No doctests:** `--tests` avoids the `keyboard.rs` illustrative doctests that
    tripped Unit 0's first attempt.
  - **No bounded-work step.** Its ratio means something different on wgpu (3.91,
    against 10.77 on tiny-skia), because the per-frame constant cost dominates. The
    tiny-skia job keeps it.
- **It runs in parallel** with the existing jobs. Record CI wall time before and
  after, over three runs each: it should rise by little or nothing if no existing
  job is shorter than this one.
- **The log must show the renderer.** `frame_pixels` already asserts it. Also print
  one `renderer=wgpu` line per test binary, so a reader of the log sees the proof
  rather than inferring it from passes.
- **It gates the release:** `release.yaml` requires the whole `ci.yaml` run to
  succeed. That is intended: a release must pass on the renderer users get.

## R-4 — parity, loose by design

The two renderers' frames cannot be compared inside one process: the backend is
fixed per process by `ICED_TEST_BACKEND`. Do not work around that with
`std::env::set_var`. **Compare across jobs instead:**

1. **Each rendered job writes** a line per R-2 element when
   `SNORA_PARITY_OUT` is set:
   `element <TAB> renderer <TAB> core colour <TAB> rounded physical bounds`.
   The core colour is the modal colour of the element's **interior**, excluding the
   edge rows. The bounds are rounded to whole physical pixels. Write it to a file
   named after the renderer, and upload it as an artifact.
2. **A `renderer-parity` job** has `needs:` both rendered jobs, downloads both files
   and runs `scripts/check-renderer-parity.py`. That fails if:
   - the two element sets differ;
   - any element's core colour differs (exact; Unit 0 measured identical cores for
     the rule and the underline);
   - any bounds differ by more than 1 physical px on any edge.

   Edge rows are allowed to differ, since antialiasing differs by design.

## Check each assertion can fail

Back up to named files with `cp`, break, run, restore, then `sha256sum -c`.
**Never `git checkout`/`restore`.**

| Assertion | Breakage |
|---|---|
| the wgpu job really is wgpu | in a branch run, remove `mesa-vulkan-drivers`: the job must **fail** on `frame_pixels`'s renderer assertion, not pass on a fallback |
| parity: core colour | **reintroduce RFC-102's defect**: copy the pre-RFC-102 active-tab style (shadow behind a transparent quad) from history with `git show <commit>:<path>` into place, with a named backup. tiny-skia filled the tab and wgpu did not |
| parity: element sets | drop one element from one renderer's output |
| parity: bounds | shift one element's bounds by 2 px in one file |

The first two rows need CI runs on a throwaway branch, as Unit 0 did. Dispatch the
runs sequentially (the concurrency group cancels parallel runs), then delete the
branch.

## Acceptance criteria

1. The `rendered-wgpu` job is in `ci.yaml` on `main`, green, with `renderer=wgpu` in
   its log, plus the wall-time figures.
2. Each row above shown failing, with the CI run links for the two branch rows.
3. `check-workflows.sh` passes.
4. CHANGELOG under Added. **Do not write "REQ-005 met".** The architect rules that
   on the evidence, at the cut.
