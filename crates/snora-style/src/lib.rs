//! iced style bridge for Snora Design tokens (RFC-055).
//!
//! Six modules, each mapping [`snora_design::Tokens`] to a plain `iced`
//! style struct or a complete `iced::Theme` — no `Element`, no layout,
//! no message. RFC-054 established this layer is structurally *below*
//! the widget layer: it imports nothing from `snora-widgets` or
//! `snora-core`, and the widget layer (`snora-widgets`'s prefab
//! card/button/notice/chip/progress helpers) consumes it as a plain
//! function, not the other way round. Its previous home inside
//! `snora-widgets` was that accident, not a structural requirement —
//! this crate is the correction.
//!
//! [`theme`] joined the other five modules one round later than they
//! did (round 1 of RFC-055's own review left it behind
//! `#[cfg(feature = "widgets")]` in `snora`, reasoning that theming
//! stock iced widgets is only meaningful alongside snora's own prefab
//! ones — round 2 corrected that: `theme.rs` imports nothing from the
//! widget layer either, and the one consumer known to use it does so
//! with zero `snora::widget::*` call sites. Same layer, same move.
//!
//! # Consumers, one vocabulary
//!
//! - The **prefab widgets** in `snora-widgets` (`snora_widgets::design::card`
//!   and its siblings) — the original, and still valid, consumer of the
//!   style functions, reached directly (`use snora_style as style;`
//!   internally, since RFC-056).
//! - The **engine chrome** (`snora::design::render`'s dialog card and
//!   `snora::design::responsive_render`) — reaches the style functions
//!   here directly, without depending on `snora-widgets` at all.
//! - **Applications**, via `snora::design::style::*` /
//!   `snora::design::theme` (the `snora` facade re-exports this crate
//!   at those paths — see `snora`'s own docs), or by depending on this
//!   crate directly.
//!
//! # `snora_widgets::design::{style, theme}` no longer exist (RFC-056)
//!
//! RFC-055 kept `snora_widgets::design::style` and `::theme` as
//! compatibility re-exports while the style layer moved here. RFC-056
//! removed them once `snora::design::style`/`::theme` (the documented
//! consumer route) pointed at this crate directly, on the reasoning
//! that `#[deprecated]` on a bare `pub use` re-export emits no warning
//! at all in this workspace, and no documentation ever directed anyone
//! to `snora-widgets` directly. `snora::design::*` is unaffected; only
//! a direct `snora_widgets::design::style`/`::theme` import breaks, with
//! a compile error naming this crate as the replacement. See
//! `docs/src/guides/migration-0.32-to-0.33.md`.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Color conversion between `snora_design::Color` and `iced::Color`.
pub mod color;

/// Semantic button style functions.
pub mod button;

/// Card and container style functions.
pub mod container;

/// Text style helpers.
pub mod text;

/// Progress bar style functions.
pub mod progress;

/// Token-derived `iced::Theme` emission (RFC-038).
pub mod theme;
