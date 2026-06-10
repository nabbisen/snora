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

## `snora-core` crate doctests

Crate-level doctests (`///` comments in `src/`) are tested by
`cargo test -p snora-core`. They may use bare code blocks because
they run inside the crate's own compilation context. The current
count is 17 doctests (tracked in the release checklist). Do not
accidentally break these by changing vocabulary without updating
the examples in the doc comments.

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
