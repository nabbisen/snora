//! Dialogs and edge-anchored sheets — the modal overlay surfaces.
//!
//! Both overlay types are **pure content carriers**. They do not own close
//! handlers; outside-click dismissal is installed once at the
//! [`crate::AppLayout`] level via [`crate::AppLayout::on_close_modals`], so
//! there is exactly one place to wire the close message regardless of which
//! modal is showing.
//!
//! # Sheets
//!
//! A [`Sheet`] is a panel that slides in from one of the four window
//! edges ([`SheetEdge`]) and occupies a configurable size ([`SheetSize`])
//! along the perpendicular axis. The engine resolves the enums to concrete
//! pixels at render time, so this module remains iced-free.
//!
//! [`SheetSize`] is interpreted *along the axis perpendicular to the
//! anchor edge*:
//!
//! * For [`SheetEdge::Top`] / [`SheetEdge::Bottom`] the size is a height
//!   (vertical).
//! * For [`SheetEdge::Start`] / [`SheetEdge::End`] the size is a width
//!   (horizontal).
//!
//! This is intentional: a single `SheetSize::Half` reads naturally as
//! "half of the relevant axis" no matter which edge the sheet attaches to.

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

/// Which window edge a sheet attaches to.
///
/// `Start` / `End` are logical, mirroring under
/// [`crate::LayoutDirection::Rtl`] just like every other axis-aligned
/// vocabulary in snora. `Top` / `Bottom` are direction-independent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SheetEdge {
    /// Slides up from the bottom of the window. The historical default;
    /// matches the "drawer from below" idiom.
    #[default]
    Bottom,
    /// Slides down from the top of the window.
    Top,
    /// Slides in from the logical start edge (LTR=left, RTL=right).
    Start,
    /// Slides in from the logical end edge (LTR=right, RTL=left).
    End,
}

impl SheetEdge {
    /// Whether this edge anchors along the **vertical** axis.
    /// `true` for `Top` / `Bottom`; `false` for `Start` / `End`.
    #[must_use]
    pub fn is_vertical(self) -> bool {
        matches!(self, SheetEdge::Top | SheetEdge::Bottom)
    }

    /// Whether this edge anchors along the **horizontal** axis.
    /// `true` for `Start` / `End`; `false` for `Top` / `Bottom`.
    #[must_use]
    pub fn is_horizontal(self) -> bool {
        !self.is_vertical()
    }
}

/// The size a sheet should occupy along the axis perpendicular to its
/// anchor edge.
///
/// * For a top- or bottom-anchored sheet, `SheetSize` is a height.
/// * For a start- or end-anchored sheet, `SheetSize` is a width.
///
/// Use the named variants for canonical proportions; use [`SheetSize::Ratio`]
/// for arbitrary fractions (clamped to `0.0..=1.0`); use [`SheetSize::Pixels`]
/// for a fixed pixel size independent of window dimensions (discouraged for
/// responsive apps).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SheetSize {
    /// 33 % of the window's relevant axis — the default "drawer" size.
    OneThird,
    /// 50 % of the window's relevant axis.
    Half,
    /// 67 % of the window's relevant axis.
    TwoThirds,
    /// Arbitrary fraction of the window's relevant axis. Values outside
    /// `0.0..=1.0` are clamped by the engine.
    Ratio(f32),
    /// Fixed pixel size. Only use when the content has a natural size that
    /// does not scale with the window.
    Pixels(f32),
}

impl SheetSize {
    /// The default size — one-third of the window.
    pub const DEFAULT: SheetSize = SheetSize::OneThird;

    /// Resolve to a fraction of the relevant axis, if this variant
    /// expresses one. Returns `None` for [`SheetSize::Pixels`].
    #[must_use]
    pub fn as_ratio(self) -> Option<f32> {
        match self {
            SheetSize::OneThird => Some(1.0 / 3.0),
            SheetSize::Half => Some(0.5),
            SheetSize::TwoThirds => Some(2.0 / 3.0),
            SheetSize::Ratio(r) => Some(r.clamp(0.0, 1.0)),
            SheetSize::Pixels(_) => None,
        }
    }

    /// Resolve to a pixel value, if this variant expresses one.
    /// Returns `None` for the ratio-based variants.
    #[must_use]
    pub fn as_pixels(self) -> Option<f32> {
        match self {
            SheetSize::Pixels(p) => Some(p),
            _ => None,
        }
    }
}

/// A panel that slides in from one of the window edges.
///
/// Like [`Dialog`], a sheet is content only. The dim backdrop and its
/// outside-click-to-close behavior are owned by the parent [`crate::AppLayout`].
///
/// # Builder usage
///
/// ```ignore
/// use snora::{Sheet, SheetEdge, SheetSize};
///
/// let sheet = Sheet::new(my_content)
///     .at(SheetEdge::Start)
///     .with_size(SheetSize::Half);
/// ```
pub struct Sheet<Node, Message> {
    /// The sheet's body content. The engine wraps this in a styled surface
    /// sized according to `size` and anchored to `edge`.
    pub content: Node,
    /// Where the sheet attaches. Defaults to [`SheetEdge::Bottom`].
    pub edge: SheetEdge,
    /// Size of the sheet along the axis perpendicular to `edge`.
    /// Defaults to [`SheetSize::DEFAULT`].
    pub size: SheetSize,
    _marker: PhantomData<Message>,
}

impl<Node, Message> Sheet<Node, Message> {
    /// Build a sheet with default edge ([`SheetEdge::Bottom`]) and size
    /// ([`SheetSize::DEFAULT`]).
    pub fn new(content: Node) -> Self {
        Self {
            content,
            edge: SheetEdge::default(),
            size: SheetSize::DEFAULT,
            _marker: PhantomData,
        }
    }

    /// Override the anchor edge.
    #[must_use]
    pub fn at(mut self, edge: SheetEdge) -> Self {
        self.edge = edge;
        self
    }

    /// Override the size.
    #[must_use]
    pub fn with_size(mut self, size: SheetSize) -> Self {
        self.size = size;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_resolves_correctly() {
        assert_eq!(SheetSize::OneThird.as_ratio(), Some(1.0 / 3.0));
        assert_eq!(SheetSize::Half.as_ratio(), Some(0.5));
        assert_eq!(SheetSize::TwoThirds.as_ratio(), Some(2.0 / 3.0));
        assert_eq!(SheetSize::Ratio(0.25).as_ratio(), Some(0.25));
        assert_eq!(SheetSize::Pixels(240.0).as_ratio(), None);
    }

    #[test]
    fn ratio_is_clamped() {
        assert_eq!(SheetSize::Ratio(1.5).as_ratio(), Some(1.0));
        assert_eq!(SheetSize::Ratio(-0.1).as_ratio(), Some(0.0));
    }

    #[test]
    fn default_sheet_edge_is_bottom() {
        assert_eq!(SheetEdge::default(), SheetEdge::Bottom);
    }

    #[test]
    fn vertical_horizontal_partition() {
        for edge in [SheetEdge::Top, SheetEdge::Bottom, SheetEdge::Start, SheetEdge::End] {
            assert_ne!(edge.is_vertical(), edge.is_horizontal());
        }
        assert!(SheetEdge::Top.is_vertical());
        assert!(SheetEdge::Bottom.is_vertical());
        assert!(SheetEdge::Start.is_horizontal());
        assert!(SheetEdge::End.is_horizontal());
    }

    #[test]
    fn sheet_builder_overrides() {
        let s: Sheet<(), ()> = Sheet::new(())
            .at(SheetEdge::Start)
            .with_size(SheetSize::Half);
        assert_eq!(s.edge, SheetEdge::Start);
        assert_eq!(s.size, SheetSize::Half);
    }
}
