//! Tab bar — horizontal selector for switching between sibling views.
//!
//! Like [`crate::menu`] and [`crate::sidebar`], the tab bar is described
//! as plain data here and rendered by `snora-widgets::app_tab_bar`. The
//! engine never inspects these types directly; they exist so that
//! applications and widget engines speak the same shape.
//!
//! # Choosing between tabs and a sidebar
//!
//! Both let users switch among sibling views, but they imply different
//! information density and depth:
//!
//! * **Tabs** — flat, horizontal, label-first. Three to seven peer views
//!   that the user expects to switch among frequently. Sits below the
//!   header.
//! * **Sidebar** ([`crate::SideBar`]) — vertical, icon-first, scales to
//!   more entries. Use when the navigation is the primary structural
//!   element of the app.
//!
//! Use both at once when tabs subdivide a sidebar-selected workspace.

use crate::Icon;

/// One tab in a [`TabBar`]. Carries an application-defined `TabId` so
/// that the application is the source of truth for which tab is which.
#[derive(Debug, Clone)]
pub struct Tab<TabId: Clone + PartialEq> {
    /// Application-defined identifier. Compared against [`TabBar::active`]
    /// to decide which tab to highlight.
    pub id: TabId,
    /// Visible label.
    pub label: String,
    /// Optional leading icon.
    pub icon: Option<Icon>,
}

/// A horizontal tab strip.
///
/// `TabBar` is generic over `TabId` so that applications can use any
/// `Clone + PartialEq` type — typically a small enum.
#[derive(Debug, Clone)]
pub struct TabBar<TabId: Clone + PartialEq> {
    /// Tabs in display order. The widget engine respects this order;
    /// horizontal mirroring under [`crate::LayoutDirection::Rtl`] is
    /// the engine's responsibility.
    pub tabs: Vec<Tab<TabId>>,
    /// The currently selected tab id. The widget renders this tab with
    /// an active treatment (typically an underline). If `active` does
    /// not match any tab in `tabs`, no tab is highlighted.
    pub active: TabId,
}

/// What happens when the user interacts with a tab.
///
/// Tab bars only emit one kind of event — a tab being pressed. We
/// still wrap it in an enum (rather than a bare `TabId`) so that
/// future extensions (close button on a tab, drag-to-reorder) can be
/// added without breaking the existing handler shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabAction<TabId> {
    /// The user pressed a tab. The application typically responds by
    /// updating its `active` state and re-rendering.
    Pressed(TabId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Eq, Debug)]
    enum DemoTab {
        A,
        B,
        C,
    }

    #[test]
    fn tab_bar_is_constructible() {
        let bar = TabBar {
            tabs: vec![
                Tab {
                    id: DemoTab::A,
                    label: "A".into(),
                    icon: None,
                },
                Tab {
                    id: DemoTab::B,
                    label: "B".into(),
                    icon: None,
                },
            ],
            active: DemoTab::A,
        };
        assert_eq!(bar.tabs.len(), 2);
        assert_eq!(bar.active, DemoTab::A);
    }

    #[test]
    fn tab_action_carries_id() {
        let action = TabAction::Pressed(DemoTab::C);
        assert_eq!(action, TabAction::Pressed(DemoTab::C));
    }
}
