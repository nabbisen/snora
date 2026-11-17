# RFC-136 — The book's code can go stale without a test failing

**Status.** Accepted 2026-11-18 (owner): the scope (Q-1) and the rule (Q-2) were ruled while scoping
0.66.
**Handoff.** [`handoffs/136-…`](../handoffs/136-the-books-code-can-go-stale-without-a-test-failing/implementation-handoff.md).
**Raised.** 2026-11-18, architect, from the 0.64 book work.
**Release target.** 0.66.0.
**Priority.** Adopters copy the book's code. The getting-started pages are the first code a new
adopter runs.

---

## The finding

- **0.64 changed eight signatures, and ten book pages kept the old ones** (`chips.md`,
  `notices.md`, `progress.md`, two recipes, `v021-primitives.md`, `03-add-a-header.md`,
  `reference/widgets.md`, and others). **No test failed.** The dev team found them by searching, and
  the architect corrected them by hand.
- **The cause:** the snippets are `rust,ignore`, which `mdbook test` never compiles.
- **Today, 134 Rust snippets in `docs/src` are `rust,ignore`:**
  - 35 are in migration guides;
  - about 18 are backed by RFC-069's mechanism, compiled in `examples/book_snippets` and
    `{{#include}}`d;
  - **the rest, about 80, are checked by nothing.**
- **RFC-069's rule permitted it:** "most `ignore` fences are illustrative prose with no compiled
  counterpart at all, and that is fine". The stale snippets of 0.64 were all illustrative.

## The rule (ruled)

**R-1. Every Rust snippet in the book, outside the migration guides, is either compiled or marked
with the reason it is not.**
- **Compiled:** the code lives in `examples/book_snippets`, which the workspace builds, under an
  anchor, and the page `{{#include}}`s it (RFC-069's mechanism).
- **Marked:** an HTML comment on the line before the fence,
  `<!-- not compiled: <reason> -->`. The reason is one a reader can check:
  - pseudo-code;
  - a fragment that names an application's own types and cannot stand alone;
  - a `Cargo.toml`;
  - a shell command;
  - or similar.

  "Illustrative" is not a reason.

**R-2. Signature listings are not copies.** A block that restates a public signature (for example,
`reference/widgets.md`'s `pub fn app_footer(…)`) either:
- becomes a compiled call of that function, which then breaks when the signature changes; or
- is replaced by a link to the item's rustdoc.

A copied signature cannot be checked, so it is not kept.

**R-3. A check fails on any unmarked `rust,ignore` snippet** outside the migration guides, and on a
marker with no reason. The check:
- runs in CI's Docs job, beside `mdbook test`;
- has a plant for each way it can fail.

**R-4. The migration guides are out of scope.** They record the APIs of their time on purpose, and
compiling them against today's would make them false.

**R-5. The documentation-test policy page says the new rule,** and drops "illustrative is fine",
with the reason (0.64's ten pages).

## Review against snora's principles

- **Nobody is confused.** Code an adopter copies from the book compiles against the release it
  documents, or the page says why it cannot.
- **Robust.** The next signature change fails a build instead of a reader.
- **Predictable.** One rule for every snippet, with one marker for every exception.
- **The initial cost is not the deciding factor.** About 80 snippets to move or mark, once.

## Acceptance criteria

1. Every snippet in scope is compiled or marked, with an inventory table (page, snippet, and
   which).
2. R-2's listings are handled.
3. R-3's check, in CI, with its plants. **A signature change in a plant snippet fails the build.**
4. **The book:** the documentation-test policy page (R-5), by the architect.
