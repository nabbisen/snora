//! The shortcuts router's data, and the Escape field (RFC-116 R-2).
//!
//! Iced-free, like the rest of this crate: what an application builds in `view`
//! is **data**, a list of routes (an action, the message a click on the target
//! publishes, and which control the target is). The engine (`snora`) turns each
//! route's action into the keys of the platform and places one router around the
//! skeleton inside its `Inert`.
//!
//! # The rules
//!
//! - **Every route is opt-in.** A route the builder does not name answers nothing.
//! - **A shortcut does exactly what a click on that control does:** the same
//!   message, focus taken away from wherever it was, and the Tab starting point on
//!   the target (RFC-121). A disabled or absent target does nothing.
//! - **A shortcut works unless the focused control uses that key,** and a focused
//!   text field is taken to use every arrow key, with any modifier.
//! - **Every binding has a modifier** (`Ctrl`, `Alt` or `Cmd`), so WCAG 2.1.4
//!   (single-character shortcuts) does not apply.
//! - **The digit routes match the physical digit row; named keys match the
//!   logical key.** On an AZERTY keyboard the key marked `1` types `&`.
//!
//! # The bindings
//!
//! | Destination | Windows and Linux | macOS |
//! |---|---|---|
//! | Next / previous tab | `Ctrl+Tab` / `Ctrl+Shift+Tab`, and `Ctrl+PageDown` / `Ctrl+PageUp` | `Control+Tab` / `Control+Shift+Tab`, and `Shift+Cmd+]` / `[` |
//! | Destination *n* (tabs or side bar, as chosen) | `Ctrl+1…9` | `Cmd+1…9` |
//! | Breadcrumb parent | `Alt+↑` | `Cmd+↑` |
//!
//! `Ctrl/Cmd + 1…9` go to **exactly one destination, chosen by the application
//! with [`Digits`]**, and never change while it runs. `9` is the last one, as in
//! the browsers; a digit beyond the count does nothing. **Next and previous tab
//! wrap**, as the browsers' do, and follow **Tab order**: under right-to-left the
//! tab row is mirrored, and "next" is the one Tab calls next.
//!
//! > **The macOS bindings follow Apple's conventions but have not been checked on
//! > a Mac** (how `iced_winit` reports `Cmd+↑`, and whether a field keeps it). The
//! > text-field rule keeps `Cmd+↑` from ever firing in a field. (RFC-116, L-8.)

use crate::{BreadcrumbAction, Crumb, LayoutDirection, SideBar, TabAction, TabBar};

/// The platform whose bindings apply. [`Platform::current`] is the one this was
/// compiled for; setting another is for previews and for tests of the other table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Platform {
    /// Windows, Linux and the rest: `Ctrl` is the command key.
    #[default]
    Other,
    /// macOS: `Cmd` is the command key.
    MacOs,
}

impl Platform {
    /// The platform this was compiled for (`cfg!(target_os = "macos")`).
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Other
        }
    }
}

/// Where `Ctrl/Cmd + 1…9` go: **one** destination, chosen by the application.
///
/// An enum and one builder call, so the digits cannot be given to both. The book
/// recommends [`Digits::Tabs`] when the application has a tab bar (every
/// browser's meaning), and [`Digits::SideBar`] when it has none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Digits {
    /// The *n*th tab.
    Tabs,
    /// The *n*th side bar item.
    SideBar,
}

/// What a route does, before the platform turns it into keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ShortcutAction {
    /// The next tab, in Tab order.
    NextTab,
    /// The previous tab, in Tab order.
    PreviousTab,
    /// The breadcrumb's parent.
    Parent,
    /// The digit `1…9`.
    Digit(u8),
}

/// The part a route's target plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TargetRole {
    /// A side bar item.
    SideBarItem,
    /// A tab.
    Tab,
    /// An ancestor crumb.
    Crumb,
}

/// One route: an action, the message a click on its target publishes, and which
/// control the target is (its role, and its index in the widget's own order).
#[derive(Debug, Clone)]
pub struct ShortcutRoute<Message> {
    /// What the route does.
    pub action: ShortcutAction,
    /// **Exactly the message a click on the target publishes.**
    pub message: Message,
    /// The target's role.
    pub role: TargetRole,
    /// The target's index in its widget, in the order it was given (not the
    /// order the reading direction displays it in).
    pub index: usize,
}

/// The shortcuts router, built in `view` from the same data the application hands
/// its widgets, and passed with [`AppLayout::shortcuts`](crate::AppLayout::shortcuts).
///
/// ```
/// use snora_core::{Digits, Shortcuts, Tab, TabAction, TabBar};
///
/// #[derive(Clone, PartialEq, Debug)]
/// enum View { Library, Editor }
/// #[derive(Clone, Debug, PartialEq)]
/// enum Message { Tab(TabAction<View>) }
///
/// let bar = TabBar {
///     tabs: vec![
///         Tab { id: View::Library, label: "Library".into(), icon: None },
///         Tab { id: View::Editor, label: "Editor".into(), icon: None },
///     ],
///     active: View::Library,
/// };
/// let shortcuts = Shortcuts::new()
///     .tabs(&bar, &Message::Tab)
///     .digits(Digits::Tabs);
/// // Ctrl+2 is exactly a click on the second tab.
/// let routes = shortcuts.routes(snora_core::LayoutDirection::Ltr);
/// assert!(routes
///     .iter()
///     .any(|r| r.message == Message::Tab(TabAction::Pressed(View::Editor))));
/// ```
#[derive(Debug, Clone)]
pub struct Shortcuts<Message> {
    digits: Option<Digits>,
    side_items: Vec<Message>,
    tabs: Vec<Message>,
    active_tab: usize,
    parent: Option<(usize, Message)>,
    platform: Platform,
}

impl<Message> Default for Shortcuts<Message> {
    fn default() -> Self {
        Self {
            digits: None,
            side_items: Vec::new(),
            tabs: Vec::new(),
            active_tab: 0,
            parent: None,
            platform: Platform::current(),
        }
    }
}

impl<Message: Clone> Shortcuts<Message> {
    /// No routes. Add what the application wants answered.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The side bar's items, for [`Digits::SideBar`]. Each item's route emits
    /// exactly the item's own `on_press`.
    #[must_use]
    pub fn side_bar<ViewId: Clone + PartialEq>(
        mut self,
        side_bar: &SideBar<Message, ViewId>,
    ) -> Self {
        self.side_items = side_bar.items.iter().map(|i| i.on_press.clone()).collect();
        self
    }

    /// The tab bar: next and previous tab, and [`Digits::Tabs`]. Each route emits
    /// exactly what `on_action` makes of a press on that tab.
    #[must_use]
    pub fn tabs<TabId, F>(mut self, bar: &TabBar<TabId>, on_action: &F) -> Self
    where
        TabId: Clone + PartialEq,
        F: Fn(TabAction<TabId>) -> Message,
    {
        self.tabs = bar
            .tabs
            .iter()
            .map(|t| on_action(TabAction::Pressed(t.id.clone())))
            .collect();
        self.active_tab = bar
            .tabs
            .iter()
            .position(|t| t.id == bar.active)
            .unwrap_or(0);
        self
    }

    /// The breadcrumb trail: the parent route. The parent is the ancestor just
    /// before the leaf; a trail with no ancestor has no parent route. It emits
    /// exactly what `on_action` makes of a press on that crumb.
    #[must_use]
    pub fn crumbs<CrumbId, F>(mut self, crumbs: &[Crumb<CrumbId>], on_action: &F) -> Self
    where
        CrumbId: Clone,
        F: Fn(BreadcrumbAction<CrumbId>) -> Message,
    {
        self.parent = crumbs
            .iter()
            .position(|c| c.is_leaf)
            .and_then(|leaf| leaf.checked_sub(1))
            .map(|i| {
                (
                    i,
                    on_action(BreadcrumbAction::Pressed(crumbs[i].id.clone())),
                )
            });
        self
    }

    /// Where `Ctrl/Cmd + 1…9` go. **With no call, there is no digit route.**
    #[must_use]
    pub fn digits(mut self, digits: Digits) -> Self {
        self.digits = Some(digits);
        self
    }

    /// The platform whose bindings apply. Defaults to [`Platform::current`].
    ///
    /// **An application normally leaves this alone**: the default is the platform
    /// it was built for, which is the table its users expect. It exists for tests
    /// and previews of the other platform's table.
    #[must_use]
    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    /// The platform this router's bindings are for.
    #[must_use]
    pub fn platform_in_use(&self) -> Platform {
        self.platform
    }

    /// Every route, for a layout read in `direction`.
    ///
    /// - **Digits:** `n` is the *n*th destination, `9` is the last, and a digit
    ///   beyond the count has no route. No destination (digits set to tabs, and
    ///   no tab bar) means no route.
    /// - **Next and previous tab follow Tab order,** which the tab row mirrors
    ///   under [`LayoutDirection::Rtl`], and **wrap**. With fewer than two tabs
    ///   there is nothing to go to.
    /// - **Parent:** the ancestor before the leaf.
    #[must_use]
    pub fn routes(&self, direction: LayoutDirection) -> Vec<ShortcutRoute<Message>> {
        let mut routes = Vec::new();

        // The digits: one destination, never both.
        if let Some(digits) = self.digits {
            let (messages, role) = match digits {
                Digits::Tabs => (&self.tabs, TargetRole::Tab),
                Digits::SideBar => (&self.side_items, TargetRole::SideBarItem),
            };
            for n in 1..=9u8 {
                // 9 is the last, as in the browsers; the others are themselves.
                let index = if n == 9 {
                    messages.len().checked_sub(1)
                } else {
                    Some(usize::from(n) - 1).filter(|i| *i < messages.len())
                };
                if let Some(index) = index {
                    routes.push(ShortcutRoute {
                        action: ShortcutAction::Digit(n),
                        message: messages[index].clone(),
                        role,
                        index,
                    });
                }
            }
        }

        // Next and previous tab, in Tab order.
        let count = self.tabs.len();
        if count >= 2 {
            let after = (self.active_tab + 1) % count;
            let before = (self.active_tab + count - 1) % count;
            // The row is built in reverse under RTL, so Tab order is too.
            let (next, previous) = match direction {
                LayoutDirection::Ltr => (after, before),
                LayoutDirection::Rtl => (before, after),
            };
            routes.push(ShortcutRoute {
                action: ShortcutAction::NextTab,
                message: self.tabs[next].clone(),
                role: TargetRole::Tab,
                index: next,
            });
            routes.push(ShortcutRoute {
                action: ShortcutAction::PreviousTab,
                message: self.tabs[previous].clone(),
                role: TargetRole::Tab,
                index: previous,
            });
        }

        if let Some((index, message)) = &self.parent {
            routes.push(ShortcutRoute {
                action: ShortcutAction::Parent,
                message: message.clone(),
                role: TargetRole::Crumb,
                index: *index,
            });
        }
        routes
    }
}

/// What Escape does on an overlay of the layout, with named fields: the dialog and
/// the sheet (`modal`), the menus (`menu`). Passed with
/// [`AppLayout::escape`](crate::AppLayout::escape).
///
/// **The rule:** Escape publishes the overlay's escape message if one is set, and
/// otherwise its click-outside message (`on_close_modals`, `on_close_menus`). The
/// click-outside message still answers clicks. An overlay with neither is left for
/// the application, and the key stays `Ignored`.
///
/// It replaced `snora::keyboard::dismiss_on_escape` (deprecated in 0.57.0, removed in
/// 0.60.0), whose two `bool`s and two
/// `Option`s in a row were easy to pass in the wrong order with no compiler error.
/// The field answers on the overlay's own layer, so it works while the overlay is
/// open; and **Escape does not dismiss toasts**, so a stray key cannot clear an
/// unread persistent error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Escape<Message> {
    /// What Escape publishes while a dialog or a sheet is open.
    pub modal: Option<Message>,
    /// What Escape publishes while a menu is open.
    pub menu: Option<Message>,
}

impl<Message> Default for Escape<Message> {
    fn default() -> Self {
        Self {
            modal: None,
            menu: None,
        }
    }
}
