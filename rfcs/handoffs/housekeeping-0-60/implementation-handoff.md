# Implementation handoff — 0.60: remove `dismiss_on_escape`, and measure 0.59's size jump

**Ruled.** Part 1 follows the versioning policy: *a deprecated item is removed in the third
minor after the one that deprecated it.* Part 2 is the measurement that indicator 2 recorded
as owed before the next release. The owner put both in 0.60's scope on 2026-11-11.
**Release target.** 0.60.0. **Two parts, one short review.** It may start now. It is
independent of RFC-125 and RFC-123, so land it whenever it is ready.
**Branch.** `housekeeping-0-60`, off `main`. Land as `main` requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## Part 1 — remove `snora::keyboard::dismiss_on_escape` (deprecated since 0.57.0)

Its note says "removed in 0.60.0". The replacement is `AppLayout::escape(Escape { modal, menu })`.
An overlay of the application's own, outside `AppLayout`, needs a three-line `match`.

- **The function** (`crates/snora/src/keyboard.rs:95`), its rustdoc example, and the
  `#[expect(deprecated)]` test module that exercises it (`:177` onward).
- **Every remaining mention of it as a live item.** The two intra-doc links in
  `cycle_zones`'s rustdoc (`:127`, `:134`) would break the Docs job. Its example
  (`:145`, "alongside dismiss_on_escape") moves to the `AppLayout::escape` wiring.
  - `crates/snora/src/lib.rs:164`;
  - `crates/snora-core/src/layout.rs:348, 426` and `shortcuts.rs:338` may keep the history,
    but in the past tense: "replaced `dismiss_on_escape`, removed in 0.60.0";
  - `crates/snora/tests/modal_keyboard.rs:593`: "so the application's own
    `dismiss_on_escape` still sees it" becomes "so the application's own Escape handling
    still sees it". **The test's assertions are unchanged.**
- **`crates/snora-core/src/focus.rs` (`next_zone`):** its rustdoc and comments say it
  "mirrors `dismiss_on_escape`'s shape" (`:20`, `:141`, `:150`, `:179`, `:395`). **Reword those
  references only.** Change nothing in `next_zone`'s signature or behaviour, and nothing in
  what its doc says about `has_menu`. That question is with the owner, separately.

**Check:**
- `grep -rn "dismiss_on_escape" crates examples` finds only past-tense history;
- **no `#[deprecated]` item remains** in the workspace. Say so in the review request;
- `escape.rs`'s and `modal_keyboard.rs`'s Escape tests pass unmodified.

**CHANGELOG `[Unreleased]`, Removed:** `snora::keyboard::dismiss_on_escape`, deprecated in
0.57.0, with its replacement, citing the rule.

## Part 2 — what made the widgets 46 KB larger in 0.59.0

`binary-size.csv`: `widgets_diff_bytes` went from **55,296 (0.58.0) to 101,504 (0.59.0)**,
+46,208 B (+84%). Both rows were built by CI with rustc 1.99.1 on `ubuntu-latest`.
`widgets_bytes` rose 47,096 while `engine_bytes` rose 888. That is 68% of the 150 KB bar
(`feature-gating-criteria.md`, indicator 2), and the largest single move it has recorded.

**Measure and report. Do not change code for size in this unit.**
1. **Reproduce the two numbers locally** with `scripts/measure-binary-size.sh`, at the
   `0.58.0` and `0.59.0` tags, with one toolchain. Report the toolchain. Local absolute
   numbers may differ from CI's; **the delta is what must reproduce.** If it does not, stop
   and report that first.
2. **Attribute the delta** with `cargo bloat` (`--release`, the probe's profile, `--crates`
   and then by function) on the widgets probe (`examples/size_probe_widgets`, package `snora-size-probe-widgets`),
   at both tags. Report the top movers
   between the tags, by crate and by function.
3. **Answer, from what the numbers show:**
   - how much of the delta is `MenuAnchor` and its dropdown overlay (RFC-124);
   - how much is generic code instantiated anew (a second `scrollable`, a new message type,
     overlay machinery from `iced_widget` that the probe did not use before);
   - whether any of it is paid by an application that never opens a header menu.
4. **Then measure `main` at the end of 0.60** the same way, once RFC-125 and RFC-123 have
   landed, so the cut knows what 0.60 adds on top. If that measurement is not possible
   before the cut, say so and the architect records it.

If the measurement shows a cost that could be avoided, **report it**. Whether to act is the
architect's to rule, and a design change is its own RFC.

**Write the results in the review request** (`.git-exclude/review-request/housekeeping-0-60/`,
with the raw `cargo bloat` output under `evidence/`). The architect records indicator 2.

## Acceptance criteria

All of `ci.yaml`'s Rust quality gate commands (16), the Docs job's commands, and the feature
matrix. Then the branch CI. The migration guide is the architect's.
