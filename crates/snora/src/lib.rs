//! # snora
//!
//! The iced engine for the Snora GUI framework.
//!
//! This crate binds [`snora_core`] vocabulary to iced. It exposes a single
//! entry point, [`render`], a toast lifecycle helper module, and — when
//! the `widgets` feature is enabled (the default) — a re-exported set of
//! prefab `iced::Element` builders from the [`snora-widgets`] crate.
//!
//! # Layering
//!
//! ```text
//! Your application
//!         │
//!         ▼
//!  snora::render(AppLayout<Element, Message>) -> Element
//!         │
//!         ├─► snora-widgets   (optional, prefab UI parts)
//!         │         │
//!         │         ▼
//!         │   snora-design   (optional, design tokens — no iced dependency)
//!         ▼
//!  snora-core   (vocabulary: Toast, Dialog, Sheet, SheetSize, …)
//! ```
//!
//! The dependency graph above is strict and the only way crates relate
//! to each other:
//!
//! * `snora-core` has zero iced dependency. It owns the vocabulary
//!   (what choices exist).
//! * `snora-design` has zero iced dependency. It owns the opt-in design
//!   token vocabulary (`Tokens`, `Palette`, contrast utilities).
//! * `snora-widgets` depends on `snora-core`, `iced`, and (behind the
//!   `design` feature) `snora-design`. It owns the prefab widget visuals
//!   and the iced style bridge that turns tokens into `iced::*::Style`.
//! * `snora` depends on `snora-core` and (optionally) `snora-widgets`
//!   and `snora-design`. It owns the engine — `render`, the layer
//!   composition, and the toast lifecycle helpers.
//!
//! Applications normally only depend on `snora` and use it as the single
//! umbrella crate; the workspace split exists so each layer can evolve
//! at its own pace.
//!
//! # A minimal application view
//!
//! ```rust,no_run
//! use iced::{Element, widget::text};
//! use snora::{AppLayout, render, LayoutDirection};
//!
//! struct MyState;
//!
//! #[derive(Debug, Clone)]
//! enum Message {}
//!
//! fn view(_state: &MyState) -> Element<'_, Message> {
//!     let body: Element<'_, Message> = text("Hello, snora!").into();
//!
//!     let layout = AppLayout::new(body)
//!         .direction(LayoutDirection::Ltr);
//!
//!     render(layout)
//! }
//! ```
//!
//! # Engine-only builds
//!
//! Applications that supply 100 % of their UI parts and do not want the
//! prefab widgets compiled in can opt out:
//!
//! ```toml
//! [dependencies]
//! snora = { version = "0.38", default-features = false }
//! ```
//!
//! In this configuration `snora-widgets` is not pulled in and the
//! `snora::widget` module does not exist.
//!
//! [`snora-widgets`]: https://docs.rs/snora-widgets

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// ---- Re-export the vocabulary from snora-core --------------------------
pub use snora_core::{
    AppLayout, BreadcrumbAction, Crumb, Dialog, Edge, Icon, LayoutDirection, Menu, MenuAction,
    MenuItem, Sheet, SheetEdge, SheetSize, SideBar, SideBarItem, Tab, TabAction, TabBar, Toast,
    ToastIntent, ToastLifetime, ToastPosition,
};

// ---- Engine modules (always present) ----------------------------------
/// Stable identifiers snora attaches to the surfaces it renders itself
/// (RFC-047). Private module; the identifier reference page under
/// `docs/src/reference/` is the public-facing form of this contract.
mod identifiers;
/// Keyboard dismissal helper: [`keyboard::dismiss_on_escape`].
pub mod keyboard;
mod overlay;
/// The single rendering entry point: [`render`].
pub mod render;
/// Width-aware rendering: [`responsive::responsive_render`] (RFC-046).
pub mod responsive;
/// Toast rendering and lifecycle helpers
/// ([`subscription`](toast::subscription), [`sweep_expired`](toast::sweep_expired)).
pub mod toast;

pub use render::render;
pub use responsive::responsive_render;

// ---- Widget re-exports (feature-gated) --------------------------------

/// Direction-aware row helpers. Re-exported from `snora-widgets`.
#[cfg(feature = "widgets")]
pub use snora_widgets::direction;

/// Shared style functions used by the prefab widgets.
/// Re-exported from `snora-widgets`.
#[cfg(feature = "widgets")]
pub use snora_widgets::style;

/// Optional prefab `iced::Element` builders for header / sidebar / footer
/// / menu / icon. Re-exported from `snora-widgets`.
///
/// This module is only available when the `widgets` feature is enabled
/// (which is the default).
#[cfg(feature = "widgets")]
pub mod widget;

/// Convenience re-export of Lucide icon constants. Available when both
/// `widgets` and `lucide-icons` features are enabled.
#[cfg(all(feature = "widgets", feature = "lucide-icons"))]
pub use snora_widgets::lucide;

// ---- Design re-exports (feature-gated) --------------------------------

/// Snora Design token types, iced style bridge, and contrast utilities.
///
/// Available when the `design` feature is enabled. Exposes:
///
/// * Token vocabulary from [`snora_design`]: [`design::Tokens`],
///   [`design::Palette`], [`design::Color`], and the full variant /
///   sub-token set.
/// * The iced style bridge under [`design::style`]: color conversion,
///   semantic button styles, and card/container styles.
/// * Shallow UI primitives: [`design::button`], [`design::card`],
///   [`design::notice`], [`design::chip`], [`design::progress`].
/// * Pure-Rust WCAG contrast utilities under [`design::contrast`]:
///   [`design::contrast::relative_luminance`],
///   [`design::contrast::contrast_ratio`],
///   [`design::contrast::composite_over`].
///
/// # iced 0.14 focus limitation
///
/// Standard `button` / `container` styles in iced 0.14 do not expose
/// keyboard-focus state. The style bridge maps every status iced does expose
/// (hover, pressed, disabled); custom focus rings on standard controls are
/// not deliverable in v0.20 through this path. See RFC-025 and
/// `docs/src/contributing/semantic-accessibility.md` for detail.
#[cfg(feature = "design")]
pub mod design;
