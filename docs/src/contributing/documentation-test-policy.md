# Documentation test policy

This page defines how Snora keeps code blocks in `docs/src` valid as
the vocabulary evolves.

## Code fence classifications

Every Rust code block in `docs/src` uses one of four fences:

| Fence | Meaning | When to use |
|---|---|---|
| `rust,ignore` | Not compiled at all — the fence stays this way for almost every book snippet, and the reason is structural, not per-snippet; see [Why nearly every book fence is `ignore`](#why-nearly-every-book-fence-is-ignore) | Anything importing `snora`, which is nearly everything |
| `rust,no_run` | Compiles, does not execute | A **self-contained** snippet with no `snora` import — e.g. pure `std`/`iced`-only code — see the caveat below |
| `rust` (plain) | **Prohibited in prose docs** | Only used in crate doctests (`///`/`//!` comments in `src/`), where crate context is available |
| `bash` / `toml` / `text` | Non-Rust code | Shell commands, config, plain diagrams |

### Rule: no bare `rust` fence in `docs/src`

Bare ` ```rust ` blocks in mdBook prose are sent to `rustc` during
`mdbook test`. Because most prose snippets are partial or require
a full `iced::Application` context, they fail unless tagged. The
rule is simple: **tag every Rust fence in `docs/src` at write time**.

The CI `docs` job runs `mdbook test docs` and enforces this.

## Why nearly every book fence is `ignore`

**111 of 111 Rust fences in `docs/src` are `rust,ignore` (RFC-069).**
That is not 111 independent per-snippet judgements landing on the same
answer — it is one structural fact about the build, repeated 111 times:
`docs/book.toml` has no `[rust]` section, and the docs CI job runs bare
`mdbook test docs` with no `-L`. **No fence importing `snora` can
compile, however small or however complete the snippet is.** A five-line
snippet constructing one token value — pure, no event loop, no
renderer — is exactly as stuck at `ignore` as a full application shell,
because neither can resolve `use snora::…` at all.

An author who reads only the classification table above and tries to
"climb the ladder" on a clean, complete snippet will find `no_run`
simply fails to compile — not because the snippet was wrong, but
because the rung was never attached to anything for a fence that
imports `snora`. `no_run` is genuinely reachable only for a fence with
**no `snora` import at all** (rare in this book, since nearly every
example exists to show `snora` usage).

**This means the per-fence reason rule crate doctests carry (previous
section, RFC-064) does not extend to the book, on purpose.** Requiring
each of 111 fences to state "no library path" would produce 111 copies
of one sentence — and RFC-064's own rationale for the reason rule was
that identical justifications drift apart from each other without
anyone noticing. At 111 copies, that failure mode would be the design,
not a risk. The structural cause is recorded here, once, instead.

**Two consequences that follow from the cause being structural rather
than per-snippet:**

- **Giving the book a library path was considered and rejected**
  (RFC-069). Measured over the 90 non-migration-guide fences: 23
  reference `self` (need an `impl` context the fence doesn't have) and
  34 reference a binding they never declare — roughly half cannot
  compile *regardless* of a library path, because they are fragments,
  not because they lack one. The docs CI job today performs no `cargo`
  build at all; adding a library path means building the full
  workspace, `iced` included, for partial coverage of the half that was
  already least likely to rot symbol-for-symbol.
- **Migration-guide fences are permanently excluded from any compile
  mechanism.** 20 of the 111 live under `docs/src/guides/migration-*.md`
  and deliberately show *old* APIs as they were at the time of the
  break. Compiling one against current source would be wrong — the
  staleness is the content, not a defect in it. Do not add a migration
  guide fence to the pilot below, or any future extension of it.

## The validated path: purpose-written, compiled snippets, `{{#include}}`d in

A `rust,ignore` fence tag does not mean a snippet's correctness is
unverifiable — it means *that fence* is not the thing verifying it.
The validated path, when a book page needs a reader-facing example on
`snora`'s frozen covenant surface (the design tokens and the public
`snora_style` bridge functions — see `api-governance.md`), is:

1. A small, `publish = false` workspace member holding the actual
   source, anchored with mdBook's `// ANCHOR: name` / `// ANCHOR_END: name`
   comments, compiled and tested like any other workspace crate — CI
   proves it compiles because a crate compiles it, not because a fence
   tag claims it does.
2. The book page pulls the anchored region in with mdBook's
   `\{{#include}}` directive, naming the source path and the anchor
   (see `docs/src/design/tokens.md` for a live example). The fence tag
   around the include directive stays `rust,ignore` — `\{{#include}}`
   inserts raw text into the fence at build time; it does not change
   how `mdbook test` treats the page, which still sees an `ignore`
   fence and skips it.

This is `{{#include}}`-from-source, not a second doctest runner: the
same mechanism the 20 example crates under `examples/` already use for
full applications, applied to fragment-sized illustrative snippets that
don't correspond to a whole example app. The `examples/workbench` app
(RFC-012-B) remains the primary living validation of full widget
builder usage; the anchored member is for a shorter illustrative
excerpt a page wants to show inline.

**A `rust,ignore` block not backed by this mechanism should link to the
example crate that validates it instead** — the older, still-valid form
of the same rule (`grep -rn "examples/" docs/src` for existing
instances). Neither form is required for every fence; most `ignore`
fences are illustrative prose with no compiled counterpart at all, and
that is fine — the point is that a fence *claiming* validation (a
"compile-checked against…" sentence, an implied "this is exactly what
ships") must actually have one behind it, by one of these two
mechanisms, not by the fence tag alone.

### Current link-compliance count

As of the RFC-069 pilot: **111** `rust,ignore` fences in `docs/src`.
**12** are now backed by the anchored `{{#include}}` mechanism above
(the RFC-069 pilot, on the frozen covenant surface); a further handful
carry an `examples/` link per the older form. The remaining majority
carry neither, and that is expected, not a defect on its own — most
`ignore` fences are illustrative prose with nothing to compile, per the
previous section.

This count is **not a CI gate**, deliberately, following RFC-064's own
precedent for the crate-side `ignore` grep: ship the audit and the
written rule first, so the number is known and stable before a gate is
pointed at it. Re-derive it with:

```bash
grep -rn '^[[:space:]]*```rust,ignore' docs/src --include="*.md"
```

**Anchor the pattern to (optional leading whitespace then) the start of
the line.** An unanchored `grep -rn '```rust,ignore\|```ignore'` also
matches the literal string wherever it appears in prose — inside this
page's own classification table, for instance — and drifts further
from the true count every time this page is edited to talk about fence
syntax. The anchored form counts fence *openers*, nothing else, and
returns exactly 111 against the current tree.

Then check each hit's surrounding lines for `{{#include` or
`examples/`. A future extension of the pilot beyond the current 12 is a
separate decision with this release's measured cost in hand, not an
automatic next step.

## Crate doctests

Crate-level doctests (`///`/`//!` comments in `src/`) are tested by
`cargo test -p <crate> --doc` (or `cargo test --workspace --all-features`
for all five crates at once). Unlike the `docs/src` prose fences above,
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

As of 0.36.1 (RFC-064 audit); re-verified unchanged at 0.38.2:

| Crate | Full run + `no_run` | `ignore` |
|---|---|---|
| `snora-core` | 20 | 0 |
| `snora-design` | 8 | 0 |
| `snora-style` | 3 | 1 |
| `snora-widgets` | 6 | 0 |
| `snora` | 5 | 2 |
| **Total** | **42** | **3** |

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
3. Run `mdbook test docs` locally before committing.
4. If the page is direction-sensitive, complete the
   [ABDD checklist](abdd-checklist.md).
