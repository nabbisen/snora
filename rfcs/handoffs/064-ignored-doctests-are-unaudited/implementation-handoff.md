# Developer Handoff — RFC-064 ignored doctests

**Governing RFC.** [RFC-064](../../done/064-ignored-doctests-are-unaudited.md)
**Status.** Inherited from RFC-064 — Accepted (owner, 2026-08-18).
**Release target.** 0.36.1. **Documentation and doc comments only** — no
behaviour change.
**Implementation units.** One.

---

## 1. Task title

Audit all 19 `ignore` fences in `crates/`, promote each to the strongest rung it
can reach, require a stated reason for those that stay ignored, and correct the
policy's false claim about a mechanism that does not exist.

## 2. Purpose

`snora-style`'s `to_iced_color` doctest is `rust,ignore` and **runs and passes
when un-ignored**, carrying a real `assert_eq!`. A quality check that exists,
compiles, and asserts something has never executed.

Nothing distinguishes *"this cannot run"* from *"nobody tried."*

## 3. The ladder — this is not "un-ignore everything"

Frame every fence against three rungs, and put each at the **highest one it can
reach**:

| Rung | Behaviour | Use when |
|---|---|---|
| **full run** (bare ```` ```rust ````) | compiles **and executes** | the snippet asserts something |
| **`no_run`** | compiles, does not execute | complete and compilable, but only demonstrates API shape |
| **`ignore`** | **not compiled at all** | genuinely cannot compile — partial fragments, undefined types, event loops |

**`ignore` is the weakest rung and buys nothing.** An ignored fence is not a
test; it is a comment that looks like one, and it silently rots as the API moves.
`no_run` is the important middle rung: it costs no runtime and still catches API
drift, which is the actual value for most of these.

Expect **most of the 19 to stay `ignore`** — many are deliberately partial
(`// In your subscription:` fragments). That is fine. What is not fine is that
today nothing records which are which.

## 4. The inventory — 19 fences, machine-derived

```bash
grep -rn '```rust,ignore\|```ignore' crates/ --include="*.rs"
```

**Re-run it** and confirm the count before starting; this was taken 2026-08-18.

| Crate | Count | Files |
|---|---|---|
| `snora-core` | 2 | `layout.rs` ×2 |
| `snora-style` | 4 | `button.rs`, `color.rs`, `text.rs`, `theme.rs` |
| `snora-widgets` | 6 | `design.rs`, `design/{button,card,chip,notice,progress}.rs` |
| `snora` | 7 | `lib.rs`, `responsive.rs`, `keyboard.rs` ×2, `design/render.rs` ×2, `toast.rs` |

**Two are not tagged as Rust at all** — `snora/src/lib.rs` and
`snora/src/toast.rs` use bare ```` ```ignore ```` rather than
```` ```rust,ignore ````. Both behave the same to rustdoc, but the bare form
loses syntax highlighting. Normalise to `rust,ignore` while you are there.

## 5. The four already assessed — start here

Probed by un-ignoring, running, and reverting:

| Fence | Verdict | Evidence |
|---|---|---|
| `snora-style/src/color.rs:12` | **full run** | proven: `... color::to_iced_color (line 12) ... ok`, has `assert_eq!` |
| `snora-style/src/button.rs:19` | **`no_run`** likely | constructs an `iced::widget::button` value; asserts nothing |
| `snora-style/src/text.rs:15` | **`no_run`** likely | same shape |
| `snora-style/src/theme.rs:350` | **stays `ignore`** | calls `.run()` (event loop) *and* references an undefined `App` |

`theme.rs` is the model for a *correct* ignore — and it still needs its reason
written down, because "obvious to whoever wrote it" is how the other three ended
up beside it.

## 6. Q-1 is decided: promote by what the snippet does

- **Asserts something** → full run.
- **Compiles but asserts nothing** → `no_run`.
- **Cannot compile** → `ignore` + reason.

Report which rung each of the 19 got and why. A one-line table in the review
request is enough.

## 7. Q-2 is decided: no CI gate this round

A grep for unexplained `ignore` fences is mechanisable and RFC-064 raises it.
**Do not build it now.** Ship the audit and the written rule first, so the fence
count is known, small and stable before a gate is pointed at it. Adding a gate
to a set nobody has yet cleaned is how gates get disabled.

Note in the policy that the gate is a deferred option, so the idea is not lost.

## 8. The policy corrections — `documentation-test-policy.md`

**8.1 It asserts a mechanism that does not exist.**

> The current count is **17 doctests** (tracked in the release checklist).

Both halves are false. The count is **20** in `snora-core` (18 passing + 2
ignored), and `grep -n "doctest" docs/src/contributing/release-process.md`
returns **nothing**.

Fix both, and **do not leave a third state**: either add the checklist line the
policy promises, or delete the promise. A claimed control that is not there is
worse than no claim — it is why nobody looked.

Recommended: add the line, since the count is now meaningful.

**8.2 It predates two of the five crates.**

The policy covers `snora-core` and `snora-widgets` only. `snora-design` and
`snora-style` are unmentioned — and `snora-style` (RFC-055, 0.32.0) inherited the
widget-crate `ignore` guidance *by analogy*, which is how it ended up with 4 of
4 ignored while `snora-design` runs 8 of 8.

Cover all five crates, and state the distinction that the analogy missed:

> **Needing `iced` as a dependency does not imply needing a renderer at
> runtime.** Constructing a style or a widget *value* is a pure function call;
> only running an `iced::application` needs an event loop.

**8.3 Record the ladder** (§3) as the rule, and that an `ignore` must carry its
reason.

## 9. Change scope

| File | Purpose |
|---|---|
| the fences in §4 that get promoted | rung change + reason comments |
| `docs/src/contributing/documentation-test-policy.md` | §8.1–8.3 |
| `docs/src/contributing/release-process.md` | the doctest-count line (§8.1) |
| `CHANGELOG.md` | **Changed** |

## 10. Explicit non-change scope

Do **not**:

- **Write new tests.** This runs tests that already exist. New coverage is
  separate work with its own scope.
- **Change any code outside a doc comment.**
  `git diff -- 'crates/**/*.rs'` must show comment lines only.
- **Add `#[ignore]` to a unit test.** There are none in `crates/` — verified —
  and this RFC does not introduce the concept.
- **Build the CI gate** (§7).
- **Change the `docs/src` prose-fence rules.** Those govern mdBook and are
  working; this is about `///` comments in `crates/`.
- **Force a fence to run that genuinely cannot.** A snippet made compilable by
  padding it with scaffolding nobody would write is worse than an honest
  `ignore` — say so and leave it.
- Modify `render_semantics.rs`.

## 11. Required tests

```bash
cargo test --workspace --all-features        # the new doctest counts
cargo test -p snora --test render_semantics  # MUST pass unmodified
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
mdbook build docs && mdbook test docs
git diff -- 'crates/**/*.rs'                 # doc-comment lines only
```

Any fence promoted to full run must be **seen to pass**, not assumed — that is
the whole point of the RFC.

## 12. Required evidence

- The re-run inventory grep, before and after.
- The per-fence rung table (§6): 19 rows, each with its rung and reason.
- Per-crate doctest counts before and after, so the change is quantified.
- The passing run for every fence promoted to full run.
- The policy diff covering §8.1, §8.2 and §8.3.
- The release-checklist line.
- `git diff -- 'crates/**/*.rs'` showing comment lines only.

## 13. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under
`.git-exclude/review-request/064-ignored-doctests-are-unaudited/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** the rung table — specifically any fence left at
`ignore` whose stated reason would also apply to one you promoted. Inconsistency
there is the sign the ladder was applied by feel rather than by rule, which is
exactly how `snora-style` got its four in the first place.
