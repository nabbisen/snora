# RFC 064 — `rust,ignore` is the default nobody has to justify, and it is hiding a runnable test

**Status.** Proposed
**Tracks.** Quality gates / measurement integrity. Continues the RFC-058 →
RFC-061 → RFC-063 line: a rule with no mechanism behind it.
**Touches.** `crates/snora-style/src/{color,button,text}.rs`,
`docs/src/contributing/documentation-test-policy.md`,
`docs/src/contributing/release-process.md`, `CHANGELOG.md`.
**Release target.** 0.36.0 (recut) — documentation and doc-comment fences only.

## Summary

`snora-style`'s `to_iced_color` doctest is marked `rust,ignore`. **It runs and
passes when un-ignored**, and it carries a real assertion:

```text
test crates/snora-style/src/color.rs - color::to_iced_color (line 12) ... ok
test result: ok. 1 passed; 0 failed; 3 ignored
```

So a quality check that exists, compiles, and asserts something has **never
executed**. Nothing distinguishes "this cannot run" from "nobody tried" —
`ignore` is the fence anyone reaches for, and no gate asks why.

## The evidence

Probed by un-ignoring the fence, running the suite, and reverting (tree verified
clean afterwards).

| Doctest | `ignore` earned? |
|---|---|
| `snora-style/src/color.rs:12` | **No** — proven runnable, has `assert_eq!` |
| `snora-style/src/button.rs:19` | **Probably not** — constructs an `iced::widget::button` value; no event loop |
| `snora-style/src/text.rs:15` | **Probably not** — same shape |
| `snora-style/src/theme.rs:350` | **Yes** — calls `.run()`, and references an undefined `App` |

Per-crate doctest state today:

| Crate | Passing | Ignored |
|---|---|---|
| `snora-core` | 18 | 2 |
| `snora-design` | 8 | **0** |
| `snora-style` | **0** | **4** |
| `snora-widgets` | 0 | 2 |
| `snora` | 0 | 2 |

`snora-design` proves the shape is achievable: a token crate with every doctest
running. `snora-style` sits beside it with none.

## Why it happened — the policy predates two of the five crates

`documentation-test-policy.md` was written when snora had three crates. It gives
`snora-core` bare fences (crate context available) and `snora-widgets`
`rust,ignore` (needs a full `iced::Application`).

**`snora-design` and `snora-style` are not mentioned at all.** `snora-style`
arrived with RFC-055 in 0.32.0, and the widget-crate guidance was applied to it
*by analogy* — but the analogy does not hold. `snora-style`'s functions are pure:
they take `&Tokens` and return an `iced::…::Style` value. Constructing a style
or a widget value needs no renderer; only *running an application* does.

Nobody decided that. It defaulted.

## The policy also asserts a mechanism that does not exist

> The current count is **17 doctests** (tracked in the release checklist).

Both halves are wrong:

- The count is **20** in `snora-core` (18 passing + 2 ignored).
- **There is no doctest line in the release checklist.** Grepped:
  `grep -n "doctest" docs/src/contributing/release-process.md` returns nothing.

This is the same shape as RFC-062's *"Within budget"* beside a 3.2×-over figure,
and RFC-059's answers filed where nobody reads: a document describing a control
that is not there. A stale number is a nuisance; a **claimed mechanism that does
not exist** is why nobody looked.

## Scope

1. **Audit every `ignore` fence in `crates/`.** For each: run it. If it passes,
   un-ignore it. If it cannot run, keep `ignore` **and state why on the line
   above it.**
2. **Un-ignore what runs**, starting with the three `snora-style` fences above.
3. **Amend the policy** so `ignore` must carry its reason — the same move
   RFC-063 made for contrast pairs, one level up: an unexplained `ignore` is an
   undeclared exemption.
4. **Cover all five crates** in the policy, including `snora-design` and
   `snora-style`, with the *pure-function* distinction stated: needing `iced` as
   a **dependency** does not imply needing a renderer at **runtime**.
5. **Correct the count claim**, and either add the release-checklist line it
   promises or delete the promise. Do not leave a third state.

## Non-goals

- **No new tests.** This RFC runs tests that already exist; writing new ones is
  separate work.
- **No `#[ignore]` on unit tests.** There are none in `crates/` — verified — and
  this RFC does not introduce the concept.
- **No change to the `docs/src` prose-fence rules.** Those are about mdBook and
  are working; this is about `///` doc comments in `crates/`.
- **No renderer-based testing.** A doctest that genuinely needs an event loop
  stays ignored, with its reason recorded.
- **No API or behaviour change.** Doc comments only.

## Open questions

**Q-1 — un-ignore, or convert to `no_run`?** For `button.rs` and `text.rs`,
which construct a widget value but assert nothing, `no_run` compiles the snippet
without executing it. That catches API drift, which is the actual value here,
and sidesteps any question of whether constructing an iced widget outside an
application is sound. Suggest: **`no_run` where there is nothing to assert, full
run where there is** (`color.rs`). Report which each fence got and why.

**Q-2 — should an unexplained `ignore` fail CI?** A grep for `rust,ignore` in
`crates/` not immediately preceded by a justification comment is mechanisable.
It would make the rule fire rather than merely exist — the RFC-059 lesson. But
it is a new CI gate on a codebase that has just absorbed several. Suggest
deferring the gate and shipping the audit plus the written rule first, then
reconsidering once the fence count is known to be small and stable.

## Acceptance criteria

1. Every `ignore` fence in `crates/` has been run; those that pass are
   un-ignored or `no_run` per Q-1.
2. Every remaining `ignore` carries a stated reason on the line above it.
3. The policy covers all five crates and states the pure-function distinction.
4. The 17-doctest claim is corrected, and the checklist promise is either
   fulfilled or removed.
5. `cargo test --workspace --all-features` passes, with the new doctest count
   recorded.
6. `render_semantics` unmodified; no `.rs` change outside doc comments —
   `git diff -- 'crates/**/*.rs'` shows comment lines only.

## Compatibility and security

**Compatibility.** Doc comments and documentation only. No API, no behaviour, no
rendering change.

**Security.** None.
