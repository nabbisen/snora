//! # snora-design
//!
//! The **iced-free** design vocabulary of the Snora GUI framework's optional
//! design system (Snora Design).
//!
//! This crate defines design *tokens* as plain data — colors, spacing,
//! typography, radius, focus, and a small semantic variant vocabulary — plus
//! four built-in [`Tokens`] presets ([`Tokens::light`], [`Tokens::dark`],
//! [`Tokens::high_contrast_light`], [`Tokens::high_contrast_dark`]).
//!
//! It has **no dependency on iced**. Converting these tokens into iced widget
//! styles is the job of the `snora-widgets` style bridge; this crate stays
//! renderer-independent so it can be unit-tested without a renderer and reused
//! by any future engine.
//!
//! ## Usage
//!
//! ```
//! use snora_design::Tokens;
//!
//! let tokens = Tokens::light();
//! let gap = tokens.spacing.md;
//! let title = tokens.typography.title;
//! assert!(title.size > 0.0);
//! ```
//!
//! Tokens are plain data: an application picks a preset, optionally tweaks
//! fields it owns, and stores the result in its own state. Snora does not own
//! token state.
//!
//! ## Accessibility note
//!
//! The built-in palettes are written to target WCAG AA contrast for body text
//! on their primary surfaces and are protected by automated, pure-Rust
//! contrast tests (see the [`contrast`] module and the crate test suite).
//! Snora Design provides accessibility-oriented *defaults*; it cannot
//! guarantee that arbitrary application content is accessible.
//!
//! ## Non-goals
//!
//! * No iced style types (those live in `snora-widgets`).
//! * No font loading or default font family.
//! * No OS theme/contrast detection.
//! * No global theme registry or CSS-like cascade.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Abstract, renderer-independent color (`Color`).
pub mod color;
/// Pure-Rust contrast utilities (relative luminance, contrast ratio).
pub mod contrast;
/// Visible-focus styling tokens ([`FocusTokens`]).
pub mod focus;
/// Semantic color [`Palette`] roles.
pub mod palette;
/// Built-in [`Tokens`] presets (reached via `Tokens::light()` etc.).
mod presets;
/// Corner [`Radius`] scale.
pub mod radius;
/// [`Spacing`] scale.
pub mod spacing;
/// The top-level [`Tokens`] bundle and its constructors.
pub mod tokens;
/// Text role and [`Typography`] scale (size + line-height).
pub mod typography;
/// Shared semantic variant vocabulary ([`Tone`], [`Emphasis`], [`Size`], [`Density`]).
pub mod variants;

pub use color::Color;
pub use focus::FocusTokens;
pub use palette::Palette;
pub use radius::Radius;
pub use spacing::Spacing;
pub use tokens::Tokens;
pub use typography::{TextRole, Typography};
pub use variants::{Density, Emphasis, Size, Tone};

#[cfg(test)]
mod tests;
