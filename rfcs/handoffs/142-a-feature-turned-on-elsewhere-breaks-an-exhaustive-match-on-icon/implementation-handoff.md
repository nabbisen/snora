# Implementation handoff — RFC-142: `Icon` becomes `#[non_exhaustive]`

**RFC.** `rfcs/accepted/142-a-feature-turned-on-elsewhere-breaks-an-exhaustive-match-on-icon.md`. Read the finding,
R-1 to R-3 and the Rulings (every question as suggested).
**Release target.** 0.67.0. **Breaking**, and named in the migration guide.
**Order.** After RFC-141. It is small, so Unit 0 and Unit 1 may go in one request if Unit 0 shows nothing unexpected.
Say so if you combine them.
**Where.** On `main`, by the one-`main` route, **after the review approves it**. Probes are scratch, described and not
committed.

## Unit 0 — the defect, demonstrated (nothing committed)

- **A scratch crate, outside the workspace,** that depends on `snora` by path with `lucide-icons` but not
  `svg-icons`, and matches `Icon` exhaustively (`Text`, `Lucide`). It compiles.
- **Add a second scratch dependency that enables `snora/svg-icons`.** The first crate's match now fails (E0004).
  Record the error.
- **With R-1 and a `_` arm,** it compiles in both graphs.
- **Every `match` on `Icon` in the workspace, listed.** `crates/snora-widgets/src/icon.rs` has two (`icon_element` and
  `icon_element_sized`). The tests and the examples may have more. Matches inside `snora-core` itself are unaffected.

## Unit 1

- **R-1:** `#[non_exhaustive]` on `Icon` (`crates/snora-core/src/icon.rs`). The doc comment says why: a feature turned
  on by another crate adds a variant, so match with a `_` arm.
- **R-2:** each match outside `snora-core` gains a `_` arm that draws nothing. Its comment says it is unreachable in a
  consistent workspace, and why it does not panic.
- **The test that keeps the exhaustiveness the compiler no longer gives `snora-widgets`:** under `--all-features`,
  construct every variant and draw it through both renderers' functions, and assert that none takes the `_` arm.
  - The simplest observable signal is the drawn ink: the `_` arm draws nothing, so an empty element fails. Or use any
    other signal you can show failing.
  - **Breakage row:** route one variant to the `_` arm, and the test fails.
- **CHANGELOG `[Unreleased]`, Changed (breaking):** what breaks (an exhaustive match on `Icon` outside snora), why,
  and the one-line fix.

## Acceptance criteria

1. The defect demonstrated, and gone with R-1, in the scratch crate.
2. The test shown able to fail. Every `ci.yaml` command run locally before the push, after the review approves it.
3. **The book is the architect's:** the icons guide's sentence, and the 0.66 → 0.67 migration guide.
