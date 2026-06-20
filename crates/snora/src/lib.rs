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
//!         ▼
//!  snora-core   (vocabulary: Toast, Dialog, Sheet, SheetSize, …)
//! ```
//!
//! The dependency graph above is strict and the only way crates relate
//! to each other:
//!
//! * `snora-core` has zero iced dependency. It owns the vocabulary
//!   (what choices exist).
//! * `snora-widgets` depends on `snora-core` and `iced`. It owns the
//!   prefab widget visuals.
//! * `snora` depends on `snora-core` and (optionally) `snora-widgets`.
//!   It owns the engine — `render`, the layer composition, and the
//!   toast lifecycle helpers.
//!
//! Applications normally only depend on `snora` and use it as the single
//! umbrella crate; the workspace split exists so each layer can evolve
//! at its own pace.
//!
//! # A minimal application view
//!
//! ```ignore
//! use iced::{Element, widget::text};
//! use snora::{AppLayout, render, LayoutDirection};
//!
//! fn view(state: &MyState) -> Element<'_, Message> {
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
//! snora = { version = "0.18", default-features = false }
//! ```
//!
//! In this configuration `snora-widgets` is not pulled in and the
//! `snora::widget` module does not exist.
//!
//! [`snora-widgets`]: https://docs.rs/snora-widgets

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// ---- Re-export the vocabulary from snora-core --------------------------
//
// Users should only need to import from `snora`. We forward the whole
// contract surface so that a single `use snora::*` (or targeted imports)
// suffices.
pub use snora_core::{
    AppLayout, BreadcrumbAction, Crumb, Dialog, Edge, Icon, LayoutDirection, Menu, MenuAction,
    MenuItem, Sheet, SheetEdge, SheetSize, SideBar, SideBarItem, Tab, TabAction, TabBar, Toast,
    ToastIntent, ToastLifetime, ToastPosition,
};

// ---- Engine modules (always present) ----------------------------------
mod overlay;
/// The single rendering entry point: [`render`].
pub mod render;
/// Toast rendering and lifecycle helpers
/// ([`subscription`](toast::subscription), [`sweep_expired`](toast::sweep_expired)).
pub mod toast;
/// Keyboard dismissal helper: [`keyboard::dismiss_on_escape`].
pub mod keyboard;

pub use render::render;

// ---- Widget re-exports (feature-gated) --------------------------------
//
// When `widgets` is enabled (the default), expose the prefab widget set
// from `snora-widgets` under `snora::widget` and `snora::direction` /
// `snora::style`. These import paths preserve the 0.5.x shape so most
// applications need no change.

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
pub mod widget {
    pub use snora_widgets::{
        app_breadcrumb, app_footer, app_header, app_side_bar, app_tab_bar, icon_element,
        icon_element_sized, render_menu,
    };

    /// The `icon` submodule path (kept for source-compat with 0.5.x
    /// callers using `snora::widget::icon::icon_element`).
    pub mod icon {
        pub use snora_widgets::{icon_element, icon_element_sized};
    }
}

/// Convenience re-export of Lucide icon constants. Available when both
/// `widgets` and `lucide-icons` features are enabled.
#[cfg(all(feature = "widgets", feature = "lucide-icons"))]
pub use snora_widgets::lucide;

// ---- Design re-exports (feature-gated) --------------------------------
//
// When `design` is enabled (opt-in, requires `widgets`), expose the Snora
// Design token types and iced style bridge under `snora::design`.

/// Snora Design token types and iced style bridge.
///
/// Available when the `design` feature is enabled. Exposes:
///
/// * Token vocabulary from [`snora_design`]: [`design::Tokens`],
///   [`design::Palette`], [`design::Color`], and the full variant /
///   sub-token set.
/// * The iced style bridge under [`design::style`]: color conversion,
///   semantic button styles, and card/container styles.
///
/// # iced 0.14 focus limitation
///
/// Standard `button` / `container` styles in iced 0.14 do not expose
/// keyboard-focus state. The style bridge maps every status iced does expose
/// (hover, pressed, disabled); custom focus rings on standard controls are
/// not deliverable in v0.20 through this path. See RFC-025 and
/// `docs/src/contributing/semantic-accessibility.md` for detail.
#[cfg(feature = "design")]
pub mod design {
    // Token types (enumerated, not glob).
    pub use snora_design::{
        Color, Density, Emphasis, FocusTokens, Palette, Radius, Size, Spacing, TextRole, Tokens,
        Tone, Typography,
    };

    /// iced style functions derived from Snora Design tokens.
    pub mod style {
        pub use snora_widgets::design::style::button;
        pub use snora_widgets::design::style::color;
        pub use snora_widgets::design::style::container;
        pub use snora_widgets::design::style::text;
    }

    /// Pilot button helpers (RFC-028).
    ///
    /// Each function wraps `iced::widget::button` with Snora Design token
    /// styling. Token ownership is handled internally via `Clone`; callers
    /// do not need to annotate lifetimes.
    pub mod button {
        pub use snora_widgets::design::button::{
            danger, danger_maybe, ghost, ghost_maybe, primary, primary_maybe, secondary,
            secondary_maybe,
        };
    }

    /// Pilot card helpers (RFC-029).
    ///
    /// Each function wraps `iced::widget::container` with Snora Design token
    /// styling. Cards in v0.20 are non-interactive visual grouping surfaces;
    /// application behaviour lives outside the card.
    pub mod card {
        pub use snora_widgets::design::card::{raised, selected, surface};
    }
}
