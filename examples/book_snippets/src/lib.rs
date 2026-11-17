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

#![allow(
    dead_code,
    unused_variables,
    unused_imports,
    unused_assignments,
    unused_must_use
)]
// The page's text is the unit here, not clippy's opinion of it (a borrow the reader keeps for clarity, a `vec!` that is
// the page's illustration, a function with more arguments than clippy likes).
#![allow(clippy::all)]

mod design_pages;
mod getting_started_header;
mod getting_started_hello;
mod getting_started_toasts;
mod guide_accessibility;
mod guide_direction;
mod guide_icons;
mod guide_keyboard;
mod guide_menus;
mod guide_menus_items;
mod guide_overlays;
mod guide_responsive;
mod guide_testing;
mod high_contrast;
mod iced_style_bridge;
mod listings;
mod readability;
mod recipes_background_task;
mod recipes_empty_state;
mod recipes_error_recovery;
mod recipes_result_card;
mod reference_overlays;
mod reference_widgets;
mod scaffold;
mod theme;
mod tokens;
mod typography;
