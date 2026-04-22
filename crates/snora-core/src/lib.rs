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
//!   [`SheetHeight`], [`Icon`].
//! * Data contracts for secondary surfaces: [`Toast`], [`Dialog`],
//!   [`BottomSheet`], [`Menu`], [`MenuItem`], [`SideBar`], [`SideBarItem`].
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

pub mod direction;
pub mod icon;
pub mod layout;
pub mod menu;
pub mod overlay;
pub mod sidebar;
pub mod toast;

pub use direction::{Edge, LayoutDirection};
pub use icon::Icon;
pub use layout::AppLayout;
pub use menu::{Menu, MenuAction, MenuItem};
pub use overlay::{BottomSheet, Dialog, SheetHeight};
pub use sidebar::{SideBar, SideBarItem};
pub use toast::{Toast, ToastIntent, ToastLifetime};
