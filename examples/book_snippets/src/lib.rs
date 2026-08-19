//! Compiled source for the `docs/src` `{{#include}}` pilot (RFC-069).
//!
//! Not an example application — `publish = false`, no `[[bin]]`. Each
//! module below anchors (`// ANCHOR: name` / `// ANCHOR_END: name`)
//! the illustrative snippets that used to live only as `rust,ignore`
//! prose in `docs/src`, so CI proves they compile because this crate
//! compiles them, not because a fence tag claims it. The fence tags in
//! `docs/src` stay `rust,ignore` — `{{#include}}` does not change how
//! `mdbook test` treats a page (see `documentation-test-policy.md`).
//!
//! None of this is ever called at runtime; it exists to compile, not to
//! run. `dead_code`/`unused_variables` are allowed crate-wide rather
//! than fought line-by-line in illustrative code that has no caller by
//! design.

#![allow(dead_code, unused_variables, unused_imports, unused_assignments)]

mod high_contrast;
mod iced_style_bridge;
mod readability;
mod theme;
mod tokens;
mod typography;
