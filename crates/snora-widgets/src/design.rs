//! Prefab design widgets for Snora Design tokens.
//!
//! This module is available when the `design` feature is enabled. It
//! provides the token-styled prefab widgets — `button`, `card`,
//! `notice`, `chip`, `progress`, and `widget` (chrome geometry). Each
//! wraps a plain iced widget and applies [`snora_style`] styling
//! internally; none of them expose the underlying style functions
//! themselves.
//!
//! **The iced style bridge — the `Tokens → iced style struct` mapping
//! functions these widgets are built on — is [`snora_style`], not this
//! module.** It moved there in RFC-055 and the compatibility re-export
//! that used to live here (`snora_widgets::design::{style, theme}`) was
//! removed in RFC-056: `snora-widgets` never documented a supported
//! path to it, and the widgets in this module consume it internally, so
//! there was nothing here for an application to keep depending on.
//! Applications wanting the style functions directly use
//! `snora::design::style::*` / `snora::design::theme` (through the
//! `snora` facade) or depend on `snora-style` directly.
//!
//! # iced 0.14 focus limitation
//!
//! `iced::widget::button::Status` exposes `Active | Hovered | Pressed |
//! Disabled` only — there is **no focused state**. The style bridge maps the
//! statuses iced does expose; custom focus rings on standard buttons/cards are
//! not deliverable in iced 0.14 through this path. `FocusTokens` remain valid
//! vocabulary for future iced versions or custom widgets that do expose focus.
//!
//! See `docs/src/contributing/semantic-accessibility.md` for the documented
//! limitation.
//!
//! # Data flow
//!
//! ```text
//! snora_design::Tokens
//!   → snora_style function (tokens + iced Status)
//!   → iced::widget::button::Style / container::Style
//!   → this module's prefab widgets
//!   → iced rendering
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use snora_design::Tokens;
//! use snora_widgets::design::button;
//!
//! #[derive(Clone)]
//! enum Message { Save }
//!
//! let tokens = Tokens::light();
//! let btn = button::primary(&tokens, "Save", Message::Save);
//! ```

/// Ergonomic pilot button helpers (RFC-028).
///
/// Wraps `iced::widget::button` with Snora Design token styling.
/// See [`button::primary`], [`button::secondary`], [`button::ghost`],
/// [`button::danger`], and their `*_maybe` disabled-state variants.
pub mod button;

/// Ergonomic pilot card helpers (RFC-029).
///
/// Wraps `iced::widget::container` with Snora Design token styling.
/// See [`card::surface`], [`card::raised`], [`card::selected`].
pub mod card;

/// Notice banner primitive (RFC-032).
///
/// Builder-style wrapper: tone, optional title, body, optional action
/// button, optional dismiss button. All interactive controls are native
/// iced buttons.
pub mod notice;

/// Filter and removable chip primitives (RFC-032).
///
/// See [`chip::filter`] and [`chip::removable`]. Both use
/// `iced::widget::button` and are keyboard-reachable.
pub mod chip;

/// Progress row and card primitives (RFC-032).
///
/// See [`progress::row`] and [`progress::card`]. Backed by
/// `iced::widget::progress_bar`. Indeterminate state is rendered as
/// 0% with a "…" suffix (iced 0.14 limitation).
pub mod progress;

/// Token-derived chrome geometry — styled variants of the prefab chrome
/// widgets (RFC-040).
///
/// See the module documentation for the full spacing/radius mapping.
pub mod widget;

/// RFC-093's channel register: asserts that `notice` and `progress`
/// vary their tone-dependent style by colour alone, exhaustively over
/// [`snora_design::Tone`]. Not a 1.4.1 conformance check — see the
/// module's own documentation for what it does and does not prove.
#[cfg(test)]
mod channel_register;
