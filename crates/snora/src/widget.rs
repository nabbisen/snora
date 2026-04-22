//! Prefab widgets for the skeleton slots.
//!
//! These are **optional** helpers that produce `iced::Element`s suitable
//! for dropping straight into [`crate::AppLayout`] slots. They encode the
//! minimum-viable visual for a typical desktop app (menu bar header,
//! status bar footer, icon-rail sidebar). Each is a plain function —
//! there is no trait to implement, no macro to invoke, and no framework
//! state to thread through.
//!
//! # When to use these
//!
//! * **Use them** to get a working app running quickly, or when your
//!   header / sidebar / footer would be indistinguishable from a generic
//!   desktop chrome.
//! * **Skip them** the moment you want to customise beyond what the
//!   helper exposes. `AppLayout`'s slots accept any `Element`; building
//!   your own chrome is the expected path once your app has taste.
//!
//! # ABDD
//!
//! Every widget in this module accepts a [`snora_core::LayoutDirection`]
//! and mirrors accordingly. `start_controls` / `end_controls` refer to
//! logical edges, not physical sides.

pub mod footer;
pub mod header;
pub mod icon;
pub mod menu;
pub mod sidebar;

pub use footer::app_footer;
pub use header::app_header;
pub use icon::icon_element;
pub use menu::render_menu;
pub use sidebar::app_side_bar;
