//! Breadcrumb — a hierarchical-position indicator that doubles as a
//! navigation widget for ancestor levels.
//!
//! The application owns the trail and decides which entries are
//! navigable; the engine renders them with direction-aware separators.
//!
//! # Choosing between breadcrumbs and tabs
//!
//! * **Tabs** ([`crate::TabBar`]) — peer-level views. All entries are
//!   reachable in one click; the user is expected to switch frequently.
//! * **Breadcrumbs** ([`Crumb`]) — ancestor levels. Only the *parents*
//!   of the current location are clickable; the leaf is the current
//!   page and is rendered as plain text.
//!
//! Use both at once in deeply nested apps where each level has tabs
//! and the breadcrumb conveys depth.

/// One step in a breadcrumb trail.
///
/// `id` is application-defined and identifies which step was pressed
/// when the user navigates back. The leaf step (the current page)
/// typically has `is_leaf: true` and the engine renders it as plain
/// text — non-clickable, visually muted differently from ancestors.
#[derive(Debug, Clone)]
pub struct Crumb<CrumbId: Clone> {
    /// Application-defined identifier for this step. Returned in
    /// [`BreadcrumbAction::Pressed`] when the user clicks an ancestor.
    pub id: CrumbId,
    /// Visible label.
    pub label: String,
    /// Whether this is the *current* (last) step. Leaves are rendered
    /// non-clickable. The application is responsible for marking
    /// exactly one entry as the leaf.
    pub is_leaf: bool,
}

impl<CrumbId: Clone> Crumb<CrumbId> {
    /// Build an ancestor crumb (clickable).
    pub fn ancestor(id: CrumbId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            is_leaf: false,
        }
    }

    /// Build a leaf crumb — the current page. Non-clickable.
    pub fn leaf(id: CrumbId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            is_leaf: true,
        }
    }
}

/// What happens when the user interacts with the breadcrumb trail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BreadcrumbAction<CrumbId> {
    /// The user pressed an ancestor crumb. The application typically
    /// responds by navigating up to that level.
    Pressed(CrumbId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancestor_constructor_marks_non_leaf() {
        let c: Crumb<u32> = Crumb::ancestor(1, "Home");
        assert!(!c.is_leaf);
        assert_eq!(c.label, "Home");
    }

    #[test]
    fn leaf_constructor_marks_leaf() {
        let c: Crumb<u32> = Crumb::leaf(2, "Profile");
        assert!(c.is_leaf);
        assert_eq!(c.label, "Profile");
    }

    #[test]
    fn breadcrumb_action_carries_id() {
        let action: BreadcrumbAction<u32> = BreadcrumbAction::Pressed(1);
        assert_eq!(action, BreadcrumbAction::Pressed(1));
    }
}
