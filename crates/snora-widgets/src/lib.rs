//! # snora-widgets
//!
//! Optional prefab `iced::Element` builders for snora's skeleton slots.
//!
//! These widgets are entirely **optional** — the snora engine
//! (`snora::render`) consumes any `iced::Element` in any slot. Use
//! these helpers to get a working app on screen quickly, and replace
//! them with hand-written elements the moment you need to customize
//! beyond what they expose.
//!
//! # Crate boundary
//!
//! This crate exists so that snora's engine (the `snora` crate) can
//! evolve independently of widget visuals. Applications normally do
//! **not** depend on `snora-widgets` directly — `snora` re-exports
//! everything here under the `snora::widget` module when its
//! `widgets` feature is enabled (the default).
//!
//! Direct `snora-widgets` use is supported for two cases:
//!
//! * Engine-only applications that opt out of `snora`'s `widgets`
//!   feature and want to add the widget set back selectively.
//! * Alternative engines built against `snora-core` that want to
//!   reuse the widget visuals.
//!
//! # ABDD
//!
//! Every widget that has start/end asymmetry takes a
//! [`snora_core::LayoutDirection`] argument and mirrors accordingly.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod direction;
mod footer;
mod header;
mod icon;
mod menu;
mod sidebar;
pub mod style;

pub use footer::app_footer;
pub use header::app_header;
pub use icon::{icon_element, icon_element_sized};
pub use menu::render_menu;
pub use sidebar::app_side_bar;

/// Convenience re-export of the Lucide icon constants, available when
/// the `lucide-icons` feature is enabled.
#[cfg(feature = "lucide-icons")]
pub mod lucide {
    pub use lucide_icons::Icon::*;
}
