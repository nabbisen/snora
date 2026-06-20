//! iced style bridge for Snora Design tokens.
//!
//! This module is available when the `design` feature is enabled. It provides:
//!
//! * [`style::color::to_iced_color`] — explicit `snora_design::Color` →
//!   `iced::Color` conversion (no implicit `From` impl, to keep the iced
//!   boundary intentional).
//! * [`style::button`] — semantic button style functions for `primary`,
//!   `secondary`, `ghost`, and `danger` variants.
//! * [`style::container`] — card/container style functions for `surface`,
//!   `raised`, and `selected` variants.
//! * [`style::text`] — text style helpers derived from [`snora_design::Tokens`].
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
//!   → style function (tokens + iced Status)
//!   → iced::widget::button::Style / container::Style
//!   → iced rendering
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use snora_design::Tokens;
//! use snora_widgets::design::style;
//!
//! let tokens = Tokens::light();
//! let btn = button("Save")
//!     .style(move |_theme, status| style::button::primary(&tokens, status));
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

/// iced style functions for Snora Design tokens.
pub mod style;
