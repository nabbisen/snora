# Implementation handoff — RFC-136: the book's code compiles, or says why not

**RFC.** `rfcs/accepted/136-the-books-code-can-go-stale-without-a-test-failing.md`. Read R-1 to R-5.
**Release target.** 0.66.0, beside the sweeps' housekeeping
(`rfcs/handoffs/0-66-sweeps-own-time/`). Neither touches the other's files.
**Where.** On `main`, by the one-`main` route.

## Unit 1

1. **The inventory:** every `rust,ignore` snippet in `docs/src` outside `guides/migration-*.md`, with
   its page and line, and what it becomes:
   - compiled, with its anchor in `examples/book_snippets`;
   - marked, with its reason;
   - or a signature listing (R-2), compiled or turned into a rustdoc link.

   Build it mechanically, and put the table in the request.
2. **Move each compilable snippet into `examples/book_snippets`** under an anchor, and
   `{{#include}}` it, as the 18 existing ones are (RFC-069).
   - **Keep what the reader sees the same,** unless the snippet was wrong. Then say so; each one is
     a finding.
   - Where a snippet needs scaffolding to compile (an application's `Message`, state, a `view`),
     the scaffolding goes outside the anchor, so the page shows only what it showed.
3. **Mark the rest** with `<!-- not compiled: <reason> -->` on the line before the fence. A reason
   a reader can check, as R-1 lists. "Illustrative" is not one.
4. **R-3's check** (a script in `scripts/`, run in CI's Docs job): it fails on an unmarked
   `rust,ignore` snippet outside the migration guides, and on a marker with no reason. Plants:
   - an unmarked snippet;
   - an empty marker;
   - **a signature change in a function a compiled snippet calls,** which must fail the workspace
     build.

   That last plant is the 0.64 case, and the proof that the rule works.
5. **Report the stale snippets you find** on the way: any snippet that did not compile against
   today's API. Each one is a book defect that a reader may have copied.

## Acceptance criteria

1. The inventory table, with every snippet in scope classified.
2. The check, in CI, with its three plants, each failing. Use named `cp` backups and
   `sha256sum -c`; never `git checkout` or `git restore`.
3. `mdbook build`, `mdbook test`, the built-links check, and the workspace build and gates.
4. **The book's policy page (R-5) is the architect's.** List anything else you would change in it.
