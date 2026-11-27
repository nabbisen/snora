# Documentation test policy

This page defines how Snora keeps code blocks in `docs/src` valid as
the vocabulary evolves.

## Code fence classifications

Every Rust code block in `docs/src` uses one of four fences:

| Fence | Meaning | When to use |
|---|---|---|
| `rust,ignore` | Not compiled by mdBook. **Since 0.66 it is compiled elsewhere, or marked:** outside the migration guides, every such fence is an `\{{#include}}` of compiled code in `examples/book_snippets`, or carries `<!-- not compiled: <reason> -->`; see [Book snippets](#book-snippets-compiled-or-marked-with-the-reason-rfc-136-066) | Anything importing `snora`, which is nearly everything |
| `rust,no_run` | Compiles, does not execute | A **self-contained** snippet with no `snora` import — e.g. pure `std`/`iced`-only code — see the caveat below |
| `rust` (plain) | **Prohibited in prose docs** | Only used in crate doctests (`///`/`//!` comments in `src/`), where crate context is available |
| `bash` / `toml` / `text` | Non-Rust code | Shell commands, config, plain diagrams |

### Rule: no bare `rust` fence in `docs/src`

Bare ` ```rust ` blocks in mdBook prose are sent to `rustc` during
`mdbook test`. Because most prose snippets are partial or require
a full `iced::Application` context, they fail unless tagged. The
rule is simple: **tag every Rust fence in `docs/src` at write time**.

The CI `docs` job runs `mdbook test docs` and enforces this.

## Book snippets: compiled, or marked with the reason (RFC-136, 0.66)

**Every Rust snippet in `docs/src` outside the migration guides is either compiled or marked with
the reason it is not.** `scripts/check-book-snippets.py` enforces it in CI's Docs job, beside
`mdbook test`.

### Why the fence tag is still `rust,ignore`

`mdbook test` has no library path to `snora` (`docs/book.toml` has no `[rust]` section), so no fence
that imports `snora` can compile *inside mdBook*. The tag says only that mdBook does not compile the
fence. **It does not mean nothing does:** since 0.66, something else compiles nearly every one.

### Compiled: `examples/book_snippets` and `\{{#include}}` (RFC-069's mechanism)

1. The code lives in `examples/book_snippets`, a `publish = false` workspace member that the
   workspace build compiles (and `cargo test -p snora-book-snippets` runs, for the snippets that are
   tests). It sits between mdBook anchors, `// ANCHOR: name` and `// ANCHOR_END: name`.
2. The page pulls the anchored region in with `\{{#include}}`, naming the path and the anchor.
3. **What a fragment takes for granted goes outside the anchor:** an application's `Message`, its
   state, `self`, a `direction`. It is in `scaffold.rs` for the design pages, and in each guide's own
   module for the guides. The page shows only what it showed.
4. **Anchors sit at column 0, with `#[rustfmt::skip]` on the wrapper,** because mdBook keeps the
   indentation of an included region.
5. **Type listings** (a page that shows `pub enum LayoutDirection { … }`) are kept as the page shows
   them. An exhaustive `match` or a struct pattern converts the listing to and from the real type,
   so a variant or field the real type gains, loses or retypes fails the build.
6. **Signatures** become calls with every argument named, so a changed signature is a compile error.
   A copied signature is not kept.
7. **Clippy runs on `book_snippets` like any crate.** A lint a page's text needs is allowed by name,
   on the narrowest item, with `#[expect(…, reason = "…")]`. A `correctness` or `suspicious` lint is
   never allowed: if a page trips one, the page is wrong.

### Marked: `<!-- not compiled: <reason> -->`

On the line directly before the fence, an HTML comment with a reason a reader can check:
- pseudo-code;
- a sketch of a type snora does not have;
- a fragment that spans several functions of an application;
- private engine arithmetic;
- or similar.

**"Illustrative" is not a reason,** and the check rejects it, as it rejects an empty marker and a
marker above no snippet. 0.64's eight signature changes left ten "illustrative" pages wrong with no
test failing. That is why the rule changed.

### Out of scope: the migration guides

Snippets under `guides/migration-*.md` show the APIs of their time on purpose. Compiling them against
today's would make them false, so they are neither compiled nor marked.

### Counts (0.66.0)

| | Snippets |
|---|---|
| Compiled, through `\{{#include}}` | 89 |
| Marked, with a reason | 10 |
| Migration guides (out of scope) | 39 |

The check counts them on every run of the Docs job. **A change to the API that a compiled snippet
calls fails the workspace build,** and that was shown by planting one (RFC-136, K06).

## Crate doctests

Crate-level doctests (`///`/`//!` comments in `src/`) are tested by
`cargo test -p <crate> --doc` (or `cargo test --workspace --all-features`
for all six crates at once). Unlike the `docs/src` prose fences above,
these run inside the crate's own compilation context, so a bare
` ```rust ` fence compiles **and executes**.

**Needing `iced` as a dependency does not imply needing a renderer at
runtime.** Constructing a style or a widget *value* is a pure function
call; only running an `iced::application` needs an event loop. This
distinction was missed when `snora-style` (RFC-055, 0.32.0) inherited
`snora-widgets`'s "needs a full `iced::Application`" guidance by analogy,
which is how it ended up with all four of its doctests `ignore`d while
`snora-design` — genuinely pure, no `iced` dependency at all — ran 8 of 8.

### The three-rung ladder

Every crate doctest fence is placed at the **highest rung it can reach**:

| Rung | Fence | Behaviour | Use when |
|---|---|---|---|
| Full run | bare ` ```rust ` | compiles **and executes** | the snippet asserts something (e.g. `assert_eq!`) |
| `no_run` | ` ```rust,no_run ` | compiles, does not execute | complete and compilable, but only demonstrates API shape |
| `ignore` | ` ```rust,ignore ` | **not compiled at all** | genuinely cannot compile — partial fragments, undefined types, event loops |

`ignore` is the weakest rung and buys nothing: an ignored fence is not a
test, it is a comment that looks like one, and it silently rots as the API
moves. `no_run` is the important middle rung — it costs no runtime and
still catches API drift.

**Every fence left at `ignore` must carry a one-line reason directly above
the fence**, stating what specifically prevents it from compiling (a
partial fragment, an undefined type standing in for the reader's own
application, a real event loop with no headless mode). "Obvious to
whoever wrote it" is not a reason; it is how fences drift apart from
identical justifications without anyone noticing (RFC-064).

A grep for unexplained `ignore` fences
(`grep -rn '```rust,ignore\|```ignore' crates/ --include="*.rs"`) is
mechanisable and could become a CI gate. **Deferred as of RFC-064**: the
audit and this written rule ship first, so the fence count is known,
small, and stable before a gate is pointed at it.

### Current counts

As of **0.66.0**, re-counted at the cut (`cargo test --workspace --all-features
--doc`). Against the table's previous figures: `snora` +1 (`keeps_focus`'s example, RFC-137) and
`snora-widgets` +1 (`horizontal`'s, RFC-138). No new `ignore`.

| Crate | Full run + `no_run` | `ignore` |
|---|---|---|
| `snora-core` | 25 | 0 |
| `snora-design` | 11 | 0 |
| `snora-style` | 6 | 1 |
| `snora-interaction` | 0 | 0 |
| `snora-widgets` | 12 | 0 |
| `snora` | 17 | 1 |
| **Total** | **71** | **2** |

Tracked in the [release checklist](release-process.md). Do not
accidentally break these by changing vocabulary without updating the
examples in the doc comments.

## Running docs tests locally

```bash
mdbook test docs
```

Requires mdBook ≥ 0.5. Install with:

```bash
cargo install mdbook --no-default-features --features search --vers "^0.5" --locked
```

## How to add a new doc page

1. Write the page.
2. Choose the right fence tag for every Rust block at write time.
3. **For a Rust snippet, put the code in `examples/book_snippets` and `\{{#include}}` it, or mark it
   `<!-- not compiled: <reason> -->`** ([Book snippets](#book-snippets-compiled-or-marked-with-the-reason-rfc-136-066)).
4. Run `mdbook test docs` and `python3 scripts/check-book-snippets.py` locally before committing.
5. If the page is direction-sensitive, complete the
   [ABDD checklist](abdd-checklist.md).
