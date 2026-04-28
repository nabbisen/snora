//! Logical layout direction — the foundation of ABDD (Accessible By
//! Default and by Design).
//!
//! snora expresses layout in terms of **logical edges** ([`Edge::Start`] /
//! [`Edge::End`]) rather than physical directions (left / right). An
//! application picks a [`LayoutDirection`] at runtime, and the engine maps
//! logical edges to physical positions accordingly.

/// Reading direction of the application's layout.
///
/// This is a framework-level setting. Individual widgets do not need to be
/// re-authored for RTL — the engine consumes this value at every point
/// where "left" or "right" would otherwise be hardcoded (sidebar side, toast
/// anchor, header end-controls, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayoutDirection {
    /// Left-to-right (e.g. English, Japanese, most languages).
    #[default]
    Ltr,
    /// Right-to-left (e.g. Arabic, Hebrew, Persian).
    Rtl,
}

impl LayoutDirection {
    /// Flip the direction. Useful for a user-facing "Flip LTR / RTL" toggle
    /// during development or accessibility preference changes.
    #[must_use]
    pub fn flipped(self) -> Self {
        match self {
            LayoutDirection::Ltr => LayoutDirection::Rtl,
            LayoutDirection::Rtl => LayoutDirection::Ltr,
        }
    }

    /// Returns `true` if the logical [`Edge::Start`] maps to the *physical*
    /// left side of the screen under this direction.
    ///
    /// Useful for engines when they need to decide whether a start-anchored
    /// element should be pushed first or last in a horizontal row.
    #[must_use]
    pub fn start_is_left(self) -> bool {
        matches!(self, LayoutDirection::Ltr)
    }
}

/// A logical position along a primary axis.
///
/// `Start` is the side a reader's eye begins at; `End` is where it finishes.
/// In LTR this maps to (Left, Right); in RTL it maps to (Right, Left).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    /// The side a reader's eye begins at — physically left under
    /// [`LayoutDirection::Ltr`], right under [`LayoutDirection::Rtl`].
    Start,
    /// The side a reader's eye finishes at — physically right under
    /// [`LayoutDirection::Ltr`], left under [`LayoutDirection::Rtl`].
    End,
}

impl Edge {
    /// Resolve this logical edge to a physical side for a given direction.
    ///
    /// Returns `true` for "left", `false` for "right".
    #[must_use]
    pub fn is_left_under(self, direction: LayoutDirection) -> bool {
        match (direction, self) {
            (LayoutDirection::Ltr, Edge::Start) => true,
            (LayoutDirection::Ltr, Edge::End) => false,
            (LayoutDirection::Rtl, Edge::Start) => false,
            (LayoutDirection::Rtl, Edge::End) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_mapping_is_consistent() {
        assert!(Edge::Start.is_left_under(LayoutDirection::Ltr));
        assert!(!Edge::End.is_left_under(LayoutDirection::Ltr));
        assert!(!Edge::Start.is_left_under(LayoutDirection::Rtl));
        assert!(Edge::End.is_left_under(LayoutDirection::Rtl));
    }

    #[test]
    fn flipping_is_idempotent_twice() {
        let d = LayoutDirection::Ltr;
        assert_eq!(d, d.flipped().flipped());
    }
}
