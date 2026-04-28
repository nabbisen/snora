//! # snora-core
//!
//! The contract and vocabulary layer of the Snora GUI framework.
//!
//! This crate has **no dependency on iced**. It defines:
//!
//! * The shape of an application layout ([`AppLayout`]) — the skeleton an
//!   engine implementation is expected to render.
//! * Vocabulary enums that spell out the canonical *choices* an application
//!   can make: [`LayoutDirection`], [`ToastIntent`], [`ToastLifetime`],
//!   [`ToastPosition`], [`SheetEdge`], [`SheetSize`], [`Icon`].
//! * Data contracts for secondary surfaces: [`Toast`], [`Dialog`],
//!   [`Sheet`], [`Menu`], [`MenuItem`], [`SideBar`], [`SideBarItem`].
//!
//! The `snora` sibling crate binds these contracts to iced and provides the
//! actual render engine. Other engines (e.g. a test double, a WGPU frontend,
//! a WASM/HTML backend) could be built against this vocabulary without
//! depending on iced.
//!
//! # Non-goals
//!
//! * **No trait-driven rendering.** Earlier drafts of snora exposed a
//!   `PageContract` trait that declared `view`, `dialog`, `toasts`, etc.
//!   In practice the render engine did not consume the non-`view` methods,
//!   forcing users to plumb them manually. v0.4 drops the trait and keeps
//!   all overlay state as plain fields on [`AppLayout`].
//!
//! * **No user-extensible close hooks.** Closing an overlay is a single
//!   concern with a single channel: [`AppLayout::on_close_menus`] and
//!   [`AppLayout::on_close_modals`]. Individual `Dialog` / `BottomSheet`
//!   values do *not* carry their own close messages.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Reading direction and logical edges (`Start`, `End`).
pub mod direction;
/// Icon enum and its source variants (text, Lucide, SVG).
pub mod icon;
/// The application skeleton ([`AppLayout`]).
pub mod layout;
/// Header / context menu vocabulary.
pub mod menu;
/// Modal overlay surfaces — [`Dialog`], [`BottomSheet`].
pub mod overlay;
/// Vertical navigation rail.
pub mod sidebar;
/// Toast notifications and lifetime / position vocabulary.
pub mod toast;

pub use direction::{Edge, LayoutDirection};
pub use icon::Icon;
pub use layout::AppLayout;
pub use menu::{Menu, MenuAction, MenuItem};
pub use overlay::{Dialog, Sheet, SheetEdge, SheetSize};
// Deprecated aliases re-exported for source compatibility with 0.5.x.
// Allow the deprecated lint here so the re-export itself does not warn —
// callers using the aliases will receive the deprecation hint.
#[allow(deprecated)]
pub use overlay::{BottomSheet, SheetHeight};
pub use sidebar::{SideBar, SideBarItem};
pub use toast::{Toast, ToastIntent, ToastLifetime, ToastPosition};
