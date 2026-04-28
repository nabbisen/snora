//! # snora
//!
//! The iced engine for the Snora GUI framework.
//!
//! This crate binds [`snora_core`] vocabulary to iced. It exposes a single
//! entry point, [`render`], plus a small set of direction-aware widget
//! helpers and a toast-lifecycle utility.
//!
//! # Layering
//!
//! ```text
//! Your application
//!         │
//!         ▼
//!  snora::render(AppLayout<Element, Message>) -> Element
//!         │
//!         ▼  consumes vocabulary from …
//!   snora_core  (Toast, Dialog, SheetHeight, LayoutDirection, Icon, …)
//! ```
//!
//! `snora-core` owns the **vocabulary** (what choices exist); `snora`
//! owns the **engine** (how those choices become pixels). The split is
//! intentionally keyed on iced-dependence: snora-core has none, so an
//! alternative engine (test double, WGPU frontend, HTML frontend) can be
//! built against the same vocabulary without touching iced.
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
//! See the `widget` module for prefab header / footer / sidebar helpers,
//! and the `toast` module for the lifecycle subscription.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// ---- Re-export the vocabulary from snora-core --------------------------
//
// Users should only need to import from `snora`. We forward the whole
// contract surface so that a single `use snora::*` (or targeted imports)
// suffices.
pub use snora_core::{
    AppLayout, BottomSheet, Dialog, Edge, Icon, LayoutDirection, Menu, MenuAction, MenuItem,
    SheetHeight, SideBar, SideBarItem, Toast, ToastIntent, ToastLifetime, ToastPosition,
};

// ---- Our own modules ---------------------------------------------------
/// Direction-aware row helpers — write logical rows without per-call match
/// statements on [`LayoutDirection`].
pub mod direction;
mod overlay;
/// The single rendering entry point: [`render`].
pub mod render;
/// Shared style functions used by the prefab widgets.
pub mod style;
/// Toast rendering and lifecycle helpers
/// ([`subscription`](toast::subscription), [`sweep_expired`](toast::sweep_expired)).
pub mod toast;
/// Optional prefab `iced::Element` builders for header / sidebar / footer / menu / icon.
pub mod widget;

pub use render::render;

/// Convenience re-export of the Lucide icon constants, for callers using
/// the `lucide-icons` feature.
#[cfg(feature = "lucide-icons")]
pub mod lucide {
    pub use lucide_icons::Icon::*;
}
