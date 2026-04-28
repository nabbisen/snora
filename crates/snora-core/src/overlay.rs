//! Dialogs and bottom sheets — the modal overlay surfaces.
//!
//! Both overlay types are **pure content carriers**. They do not own close
//! handlers; outside-click dismissal is installed once at the
//! [`crate::AppLayout`] level via [`crate::AppLayout::on_close_modals`], so
//! there is exactly one place to wire the close message regardless of which
//! modal is showing.
//!
//! # Sheet height
//!
//! [`BottomSheet`] carries a [`SheetHeight`] enum rather than a raw number.
//! The engine resolves the enum to a concrete height at render time. This
//! keeps the vocabulary in snora-core (iced-free) and confines physical
//! pixel decisions to the engine.

use std::marker::PhantomData;

/// A modal dialog.
///
/// The engine centers the content on the screen and installs a dim backdrop
/// that captures outside clicks (configured via the parent
/// [`crate::AppLayout::on_close_modals`]).
///
/// The `Message` type parameter is preserved for future extension (e.g.
/// per-dialog animations or lifecycle hooks) without breaking API shape.
pub struct Dialog<Node, Message> {
    /// The dialog body content. The engine centers this in the window and
    /// paints the dim backdrop around it.
    pub content: Node,
    _marker: PhantomData<Message>,
}

impl<Node, Message> Dialog<Node, Message> {
    /// Wrap a content node as a dialog.
    pub fn new(content: Node) -> Self {
        Self {
            content,
            _marker: PhantomData,
        }
    }
}

/// The vertical size a bottom sheet should occupy, relative to the window.
///
/// Use the named variants for canonical proportions; use [`SheetHeight::Ratio`]
/// for arbitrary fractions (clamped to `0.0..=1.0`); use
/// [`SheetHeight::Pixels`] for a fixed pixel height independent of window
/// size (discouraged for responsive apps).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SheetHeight {
    /// 33 % of the window height — the default canonical "drawer" size.
    OneThird,
    /// 50 % of the window height.
    Half,
    /// 67 % of the window height.
    TwoThirds,
    /// Arbitrary fraction of window height. Values outside `0.0..=1.0` are
    /// clamped by the engine.
    Ratio(f32),
    /// Fixed pixel height. Only use when the content has a natural size that
    /// does not scale with the window.
    Pixels(f32),
}

impl SheetHeight {
    /// The default height — one-third of the window. Matches the canonical
    /// "drawer from the bottom" feel without dominating the screen.
    pub const DEFAULT: SheetHeight = SheetHeight::OneThird;

    /// Resolve to a fraction of the window, if this variant expresses one.
    /// Returns `None` for [`SheetHeight::Pixels`].
    #[must_use]
    pub fn as_ratio(self) -> Option<f32> {
        match self {
            SheetHeight::OneThird => Some(1.0 / 3.0),
            SheetHeight::Half => Some(0.5),
            SheetHeight::TwoThirds => Some(2.0 / 3.0),
            SheetHeight::Ratio(r) => Some(r.clamp(0.0, 1.0)),
            SheetHeight::Pixels(_) => None,
        }
    }

    /// Resolve to a pixel value, if this variant expresses one.
    /// Returns `None` for the ratio-based variants.
    #[must_use]
    pub fn as_pixels(self) -> Option<f32> {
        match self {
            SheetHeight::Pixels(p) => Some(p),
            _ => None,
        }
    }
}

/// A sheet that slides up from the bottom of the window.
///
/// Like [`Dialog`], a sheet is content only. The dim backdrop and its
/// outside-click-to-close behavior are owned by the parent [`crate::AppLayout`].
pub struct BottomSheet<Node, Message> {
    /// The drawer's body content. The engine sizes the surrounding surface
    /// according to `height` and paints the dim backdrop above it.
    pub content: Node,
    /// Vertical size of the sheet. Defaults to [`SheetHeight::DEFAULT`].
    pub height: SheetHeight,
    _marker: PhantomData<Message>,
}

impl<Node, Message> BottomSheet<Node, Message> {
    /// Build a sheet with [`SheetHeight::DEFAULT`].
    pub fn new(content: Node) -> Self {
        Self {
            content,
            height: SheetHeight::DEFAULT,
            _marker: PhantomData,
        }
    }

    /// Override the height.
    #[must_use]
    pub fn with_height(mut self, height: SheetHeight) -> Self {
        self.height = height;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_resolves_correctly() {
        assert_eq!(SheetHeight::OneThird.as_ratio(), Some(1.0 / 3.0));
        assert_eq!(SheetHeight::Half.as_ratio(), Some(0.5));
        assert_eq!(SheetHeight::TwoThirds.as_ratio(), Some(2.0 / 3.0));
        assert_eq!(SheetHeight::Ratio(0.25).as_ratio(), Some(0.25));
        assert_eq!(SheetHeight::Pixels(240.0).as_ratio(), None);
    }

    #[test]
    fn ratio_is_clamped() {
        assert_eq!(SheetHeight::Ratio(1.5).as_ratio(), Some(1.0));
        assert_eq!(SheetHeight::Ratio(-0.1).as_ratio(), Some(0.0));
    }
}
