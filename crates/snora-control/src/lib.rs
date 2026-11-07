//! The focusable, key-activatable control behind snora's widgets.
//!
//! **This crate exists to serve snora's own crates, and applications should
//! not depend on it.** Depend on [`snora`](https://docs.rs/snora) instead:
//! every control this crate makes possible is reached through it, with the
//! API snora commits to keeping stable. This crate is published only because
//! `snora` and `snora-widgets` cannot be, without it; it is **versioned in
//! lockstep with them**, and its public items are exactly the ones those two
//! crates use — no more.
//!
//! # Why it exists at all
//!
//! iced 0.14's `button` handles mouse and touch events only, and registers no
//! focus target. Only `text_input` and `text_editor` implement `Focusable`.
//! So no control built on `button` can be focused, and no key presses one:
//! the gap RFC-116 closes. [`Control`] is `button`'s behaviour plus a focus
//! registration, Enter and Space, and a focus ring.
//!
//! # What else is here: marks sized in ems
//!
//! RFC-115 sizes a pictogram at 1 em of the text it sits with and the ×
//! close mark at 1.125 em. On the unstyled path the text size is the host's,
//! which view code never sees: only a widget's layout can read it. [`Glyph`]
//! (a string drawn at so many ems) and [`EmBox`] (a square of so many ems,
//! for an SVG) resolve it there, and [`Control`] does the same for its
//! target. They live here because the engine's toast close mark and the
//! widgets' icons both need them, and neither can use the other's.
//!
//! # Why a crate of its own (RFC-116, ruling of 2026-11-07)
//!
//! The toast's close button lives in the `snora` engine, whose dependency on
//! `snora-widgets` is **optional** so that engine-only builds stay small. The
//! prefab widgets live in `snora-widgets`. One control had to serve both, and
//! it could not sit in either: not under the engine, which would then need
//! `snora-widgets` after all, and not in `snora-core`, which is iced-free by
//! design. Putting it below both gives one implementation and no twin.
//!
//! # Layering
//!
//! ```text
//!  snora            snora-widgets
//!      \                /
//!       \              /
//!        snora-control          (this crate: iced and iced_core, nothing of snora's)
//! ```
//!
//! It imports nothing from `snora-core`, `snora-design` or `snora-style`, so
//! the iced-free guarantees of the first two, and the style bridge's own
//! position below the widget layer, are untouched.
//!
//! # Not enabled: iced's `advanced` feature
//!
//! `iced_core::Widget` and `iced_core::widget::operation::Focusable` are
//! public with no feature gate; `advanced` only re-exports them. This crate
//! depends on `iced_core` directly instead, at the same `0.14` requirement as
//! `iced`, so Cargo unifies the two into one `iced_core` in the graph.

#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod control;
mod em;
mod underlined;

pub use control::{Control, Ring, StyleFn};
pub use em::{EmBox, Glyph};
pub use underlined::Underlined;
