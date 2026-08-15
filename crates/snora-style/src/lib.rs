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
//! - The **card widget** (`snora_widgets::design::card`) and the other
//!   prefab widgets in `snora-widgets` — the original, and still valid,
//!   consumer of the style functions.
//! - The **engine chrome** (`snora::design::render`'s dialog card and
//!   `snora::design::responsive_render`) — reaches the style functions
//!   here directly, without depending on `snora-widgets` at all.
//! - **Applications**, via `snora::design::style::*` /
//!   `snora::design::theme` or `snora_widgets::design::style::*` /
//!   `snora_widgets::design::theme` (all four re-export this crate at
//!   their existing paths — see those crates' own docs; no import here
//!   changed by this crate's existence).
//!
//! # The `snora_widgets` re-exports are now compatibility shims
//!
//! Not deprecated in this release — deliberately: `snora::design::*`,
//! the paths applications actually use, are re-exported *through*
//! `snora-widgets` today, so deprecating the widgets paths first would
//! warn consumers who did nothing wrong. See RFC-055 §"Q-4" for the
//! planned order (re-point `snora`'s own re-exports at this crate first,
//! document this crate as consumer-facing, *then* deprecate).

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
