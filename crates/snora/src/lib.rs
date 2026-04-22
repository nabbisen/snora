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

// ---- Re-export the vocabulary from snora-core --------------------------
//
// Users should only need to import from `snora`. We forward the whole
// contract surface so that a single `use snora::*` (or targeted imports)
// suffices.
pub use snora_core::{
    AppLayout, BottomSheet, Dialog, Edge, Icon, LayoutDirection, Menu, MenuAction, MenuItem,
    SheetHeight, SideBar, SideBarItem, Toast, ToastIntent, ToastLifetime,
};

// ---- Our own modules ---------------------------------------------------
pub mod direction;
mod overlay;
pub mod render;
pub mod style;
pub mod toast;
pub mod widget;

pub use render::render;

/// Convenience re-export of the Lucide icon constants, for callers using
/// the `lucide-icons` feature.
#[cfg(feature = "lucide-icons")]
pub mod lucide {
    pub use lucide_icons::Icon::*;
}
