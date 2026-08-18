# Documentation test policy

This page defines how Snora keeps code blocks in `docs/src` valid as
the vocabulary evolves.

## Code fence classifications

Every Rust code block in `docs/src` uses one of four fences:

| Fence | Meaning | When to use |
|---|---|---|
| `rust,ignore` | Illustrative partial — will not compile standalone | Full-app-shaped snippets, partial `impl` blocks, event-loop shapes |
| `rust,no_run` | Type declaration excerpt — highlighted but not compiled | `enum`/`struct` definitions shown for reference, not executable |
| `rust` (plain) | **Prohibited in prose docs** | Only used in `snora-core` crate doctests, where crate context is available |
| `bash` / `toml` / `text` | Non-Rust code | Shell commands, config, plain diagrams |

### Rule: no bare `rust` fence in `docs/src`

Bare ` ```rust ` blocks in mdBook prose are sent to `rustc` during
`mdbook test`. Because most prose snippets are partial or require
a full `iced::Application` context, they fail unless tagged. The
rule is simple: **tag every Rust fence in `docs/src` at write time**.

The CI `docs` job runs `mdbook test docs` and enforces this.

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

As of 0.36.1 (RFC-064 audit):

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

## `snora-widgets` builder examples

Widget builder code that requires `iced` cannot run as a `snora-core`
doctest. The validated path for such examples is:

1. A small `examples/` crate that compiles in the workspace check; or
2. A `rust,ignore` block in docs linked to the relevant example crate.

The `examples/workbench` app (RFC-012-B) serves as the primary living
validation of widget builder usage.

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
