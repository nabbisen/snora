# Developer Handoff — RFC-069 book examples cannot be compiled

**Governing RFC.** [RFC-069](../../accepted/069-book-examples-cannot-be-compiled.md)
**Status.** Inherited from RFC-069 — Accepted (owner, 2026-08-19).
**Release target.** 0.38.0. **Documentation, CI, and one new `publish = false`
workspace member.** No crate change, no public API change, no rendered output.
**Implementation units.** Three, in order. **Unit 3 may be cut** — see §9.

---

## 1. Task title

Correct the documentation-test policy's account of why book examples are
`ignore`, and prove one bounded subset of them compiles.

## 2. Purpose

111 of 111 Rust fences in `docs/src` are `rust,ignore`. The policy presents that
as a per-snippet judgement. It is not: `docs/book.toml` has no `[rust]` section
and the docs CI job runs bare `mdbook test docs` with no `-L`, so **nothing
importing snora can compile, however small or complete.**

## 3. Q-1 is decided: not a library path, and not the existing examples

RFC-069 left the mechanism open and asked for measurement rather than guesswork.
The measurement was taken before writing this handoff, over the **91
non-migration fences**:

| property | count of 91 |
|---|---|
| references `self` (needs an `impl` context) | 24 |
| references a binding it never declares (`tokens`, `state`, `app`, `theme`) | 34 |
| ≤ 6 lines | 38 |
| contains `fn main` | **1** |

Median length is 7 lines; the longest is 61.

**Option (b) — give the book a library path — is rejected.** Roughly half of the
91 cannot compile *regardless* of a library path, because they are fragments
with no `impl` and no declarations. And the cost is not small: the docs CI job
today installs mdBook and runs two mdbook commands — **it performs no cargo
build at all.** A library path means building the workspace, iced included, in
that job. Large new cost, partial coverage, and the partial coverage is the
half that was already least likely to rot.

**Option (a) — `{{#include}}` from the existing `examples/`** — is right in
mechanism and wrong in source. The 20 example crates are full applications; a
7-line median fragment illustrating one builder call does not correspond to a
region of one.

**The ruling: (a) with purpose-written source.** A new workspace member holding
compilable snippets, anchored, included into the book. Snippets that are
compiled because a crate compiles them, not because a fence tag claims it.

## 4. Q-2 is decided: migration guides are excluded, permanently

**20 of the 111 are in `docs/src/guides/migration-*.md`.** They deliberately
show APIs as they were. Compiling them against current source would be wrong —
their staleness is the content. **Never include a migration guide fence in any
compile mechanism**, and write that exclusion down with its reason so nobody
"fixes" it later.

## 5. Unit 1 — correct the policy page (do this first, it stands alone)

`docs/src/contributing/documentation-test-policy.md`:

1. **Resolve the `no_run` contradiction.** The classification table calls
   `rust,no_run` *"highlighted but not compiled"*. The ladder table forty lines
   later calls it *"compiles, does not execute … still catches API drift."*
   **The ladder is right.** Fix the first; do not touch the second.
2. **State the structural cause.** Book fences are `ignore` because `mdbook
   test` has no library path, not because of snippet shape. An author reading
   the current page is told to reach for a rung that is not attached to
   anything. Say what is actually true, once, on this page.
3. **Do not extend the per-fence reason rule to the book.** 111 copies of one
   sentence is exactly the drift failure RFC-064 existed to prevent. The
   structural reason is one fact about the build and belongs in one place.
4. **Leave the crate-side rule and its grep untouched.** They are correctly
   scoped — the sentence sits under `## Crate doctests` and the grep is scoped
   to `crates/`. Section and grep agree. **This is not a bug; do not "align"
   them.**

## 6. Unit 2 — the pilot, scoped to the covenant surface

**Fifteen fences**, chosen on principle rather than convenience: they exercise
the surface RFC-036's covenant freezes — the design tokens and the public
`snora_style` bridge functions.

| file | fences | use `self` |
|---|---|---|
| `docs/src/design/tokens.md` | 5 | 1 |
| `docs/src/design/iced-style-bridge.md` | 5 | 0 |
| `docs/src/design/typography.md` | 2 | 0 |
| `docs/src/design/theme.md` | 1 | 0 |
| `docs/src/design/high-contrast.md` | 1 | 0 |
| `docs/src/guides/readability.md` | 1 | 0 |

**Only one of the fifteen uses `self`**, and eight reference an undeclared
binding that a two-line prelude (`let tokens = Tokens::light();`) supplies. This
subset was selected *because the measurement says it converts*, not because it
looked convenient.

**Why this surface and not a random slice.** It is frozen by covenant, so the
function names cannot drift — but paths, return types, and values can, and those
are exactly what a symbol-level check misses. RFC-068 is adding six functions to
this same surface in this same release; the pilot covers them on arrival.

Mechanics:

- A new workspace member, `publish = false`, matching the existing examples'
  `Cargo.toml` shape (see `examples/hello/Cargo.toml`).
- `// ANCHOR: name` / `// ANCHOR_END: name` regions, pulled in with
  `{{#include ...:name}}`. **No example crate carries anchors today** — verify
  the syntax against the pinned mdBook (`^0.5`) before converting all fifteen.
  Convert **one** first and confirm it renders.
- Where a fence and its snippet must differ (a prelude the reader does not need
  to see), the anchor excludes the prelude. **The rendered page must not gain
  scaffolding noise** — if the reader now sees five lines of setup they did not
  see before, the conversion made the docs worse and is not acceptable.
- The fence tag stays `rust,ignore`. The compilation happens in the crate, not
  in mdbook. **Do not change fence tags anywhere in this work.**

## 7. Unit 3 — make the existing link rule checkable

The policy already states the validated path: *"a `rust,ignore` block in docs
linked to the relevant example crate."* There are **111 fences and 2 links.**

Add the grep the crate side already has — `ignore` fences with no nearby
`examples/` or `{{#include}}` reference — and **record the count**. Do not make
it a CI gate in this release: RFC-064's precedent is that the audit and the
written rule ship first, so the number is known and stable before a gate points
at it. Follow that precedent.

## 8. Change scope

| File / path | Change |
|---|---|
| `docs/src/contributing/documentation-test-policy.md` | Unit 1 |
| a new `publish = false` workspace member | Unit 2 — anchored snippet source |
| the six files in §6 | Unit 2 — `{{#include}}` replacing inline snippet bodies |
| `.github/workflows/ci.yaml` | only if Unit 2 needs the member built; measure the delta |
| `CHANGELOG.md` | `[Unreleased]` |

## 9. Explicit non-change scope

- **No fence tag changes anywhere**, in the book or the crates.
- **No change to RFC-064's crate-side rule or grep.**
- **No migration-guide fence is touched** (§4).
- **No library path for `mdbook test`** (§3).
- **No CI gate** on the new grep (§7).
- **No conversion beyond the fifteen.** If the pilot works, the remaining ~76
  are a separate decision with the pilot's measured cost in hand.
- **`readability.md:72`** — the false *"Compile-checked against the pinned iced
  0.14"* line — **belongs to RFC-068's handoff**, which is editing that exact
  block. If it is still present when you arrive, RFC-068 shipped incorrectly;
  say so rather than silently fixing it.
- **Unit 3 may be cut** if Unit 2 overruns. Units 1 and 2 are the release; Unit
  3 is bookkeeping that can wait a release without harm.

## 10. Required evidence

- `mdbook build docs && mdbook test docs` — pass
- `cargo build -p <new member>` — pass, and the workspace still builds
- **CI job wall-clock before and after**, if the docs job changed. State the
  delta. If it did not change, say that explicitly — the RFC asked for this to
  be measured rather than estimated.
- **Rendered-page comparison for the fifteen**: the visible code the reader sees
  before and after. The requested proof is that it is *unchanged* or better —
  see §6's scaffolding-noise rule.
- The Unit 3 grep output with its count.
- `git diff --stat -- 'crates/'` — **expected empty.**

## 11. Acceptance criteria

1. The `no_run` contradiction is gone, resolved toward the ladder table.
2. The policy states the structural cause, once, and does not extend the
   per-fence reason rule to the book.
3. RFC-064's crate-side rule and grep are byte-identical to before.
4. The migration-guide exclusion is written down **with its reason**.
5. Fifteen fences are `{{#include}}`-driven from a compiled `publish = false`
   member; the reader-visible code is unchanged or improved.
6. No fence tag anywhere changed; `git diff -- 'crates/'` empty.
7. The CI cost delta is stated as a measurement, or its absence stated.
8. Unit 3 delivered or explicitly deferred with a reason — not silently dropped.

## 12. Required review-request format

Package under `.git-exclude/review-request/069-book-examples-cannot-be-compiled/`,
`README.md` as entry point, evidence under `evidence/`. Relative paths. State
the single entry-point path in the completion summary.

**Requested review focus:** the rendered-page comparison (§10). The failure mode
of this work is docs that are now verified and worse to read. Everything else is
mechanical.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.** If the anchor mechanism does not work against the pinned mdBook, stop
after Unit 1 and report — do not substitute a library path.
