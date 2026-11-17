//! RFC-136, R-2: **the book's listings of snora's types are compiled.**
//!
//! A page that restates a public type (`pub enum LayoutDirection { Ltr, Rtl }`) cannot be checked by reading it, so the
//! text the page shows is **this file's text**, defined here in a module of its own (the *listed* type), and each is
//! converted to and from the real one by an exhaustive `match` or a struct pattern. **A variant or a field the real type
//! gains, loses or retypes is a compile error here**, and the page is then out of date.
//!
//! Where the real type is `#[non_exhaustive]` the conversion into it cannot be written outside the crate, so only the
//! other direction is: every listed variant or field must exist in the real type, with the listed type; a variant the real
//! type gains is not seen. Each such listing says so beside it.

// ---- reference/vocabulary.md, guides/direction.md ------------------------------------------------------------------------------

#[rustfmt::skip]
mod l_direction {
// ANCHOR: vocab_direction
pub enum LayoutDirection { Ltr, Rtl }
pub enum Edge            { Start, End }
// ANCHOR_END: vocab_direction
}

#[rustfmt::skip]
mod l_edge {
// ANCHOR: direction_edge
pub enum Edge { Start, End }
// ANCHOR_END: direction_edge
}

fn direction_to_listed(x: snora::LayoutDirection) -> l_direction::LayoutDirection {
    use l_direction::LayoutDirection as L;
    match x {
        snora::LayoutDirection::Ltr => L::Ltr,
        snora::LayoutDirection::Rtl => L::Rtl,
    }
}

fn direction_to_real(x: l_direction::LayoutDirection) -> snora::LayoutDirection {
    use l_direction::LayoutDirection as L;
    match x {
        L::Ltr => snora::LayoutDirection::Ltr,
        L::Rtl => snora::LayoutDirection::Rtl,
    }
}

fn edge_to_listed(x: snora::Edge) -> (l_direction::Edge, l_edge::Edge) {
    match x {
        snora::Edge::Start => (l_direction::Edge::Start, l_edge::Edge::Start),
        snora::Edge::End => (l_direction::Edge::End, l_edge::Edge::End),
    }
}

fn edge_to_real(a: l_direction::Edge, b: l_edge::Edge) -> (snora::Edge, snora::Edge) {
    (
        match a {
            l_direction::Edge::Start => snora::Edge::Start,
            l_direction::Edge::End => snora::Edge::End,
        },
        match b {
            l_edge::Edge::Start => snora::Edge::Start,
            l_edge::Edge::End => snora::Edge::End,
        },
    )
}

#[rustfmt::skip]
mod l_toast {
// ANCHOR: vocab_toast
pub enum ToastIntent {
    Debug,
    Info,
    Success,
    Warning,
    Error,
}

pub enum ToastLifetime {
    Transient(std::time::Duration),
    Persistent,
}

pub enum ToastPosition {
    TopEnd,        // default
    TopStart,
    TopCenter,
    BottomEnd,
    BottomStart,
    BottomCenter,
}
// ANCHOR_END: vocab_toast
}

fn toast_intent_to_listed(x: snora::ToastIntent) -> l_toast::ToastIntent {
    use l_toast::ToastIntent as L;
    match x {
        snora::ToastIntent::Debug => L::Debug,
        snora::ToastIntent::Info => L::Info,
        snora::ToastIntent::Success => L::Success,
        snora::ToastIntent::Warning => L::Warning,
        snora::ToastIntent::Error => L::Error,
    }
}

fn toast_intent_to_real(x: l_toast::ToastIntent) -> snora::ToastIntent {
    use l_toast::ToastIntent as L;
    match x {
        L::Debug => snora::ToastIntent::Debug,
        L::Info => snora::ToastIntent::Info,
        L::Success => snora::ToastIntent::Success,
        L::Warning => snora::ToastIntent::Warning,
        L::Error => snora::ToastIntent::Error,
    }
}

fn toast_lifetime_to_listed(x: snora::ToastLifetime) -> l_toast::ToastLifetime {
    use l_toast::ToastLifetime as L;
    match x {
        snora::ToastLifetime::Transient(d) => L::Transient(d),
        snora::ToastLifetime::Persistent => L::Persistent,
    }
}

fn toast_lifetime_to_real(x: l_toast::ToastLifetime) -> snora::ToastLifetime {
    use l_toast::ToastLifetime as L;
    match x {
        L::Transient(d) => snora::ToastLifetime::Transient(d),
        L::Persistent => snora::ToastLifetime::Persistent,
    }
}

fn toast_position_to_listed(x: snora::ToastPosition) -> l_toast::ToastPosition {
    use l_toast::ToastPosition as L;
    match x {
        snora::ToastPosition::TopEnd => L::TopEnd,
        snora::ToastPosition::TopStart => L::TopStart,
        snora::ToastPosition::TopCenter => L::TopCenter,
        snora::ToastPosition::BottomEnd => L::BottomEnd,
        snora::ToastPosition::BottomStart => L::BottomStart,
        snora::ToastPosition::BottomCenter => L::BottomCenter,
    }
}

fn toast_position_to_real(x: l_toast::ToastPosition) -> snora::ToastPosition {
    use l_toast::ToastPosition as L;
    match x {
        L::TopEnd => snora::ToastPosition::TopEnd,
        L::TopStart => snora::ToastPosition::TopStart,
        L::TopCenter => snora::ToastPosition::TopCenter,
        L::BottomEnd => snora::ToastPosition::BottomEnd,
        L::BottomStart => snora::ToastPosition::BottomStart,
        L::BottomCenter => snora::ToastPosition::BottomCenter,
    }
}

#[rustfmt::skip]
mod l_sheet {
// ANCHOR: vocab_sheet
pub enum SheetEdge {
    Bottom,             // default
    Top,
    Start,              // logical (LTR=left, RTL=right)
    End,                // logical (LTR=right, RTL=left)
}

pub enum SheetSize {
    OneThird,           // default
    Half,
    TwoThirds,
    Ratio(f32),         // clamped 0.0..=1.0
    Pixels(f32),
}
// ANCHOR_END: vocab_sheet
}

fn sheet_edge_to_listed(x: snora::SheetEdge) -> l_sheet::SheetEdge {
    use l_sheet::SheetEdge as L;
    match x {
        snora::SheetEdge::Bottom => L::Bottom,
        snora::SheetEdge::Top => L::Top,
        snora::SheetEdge::Start => L::Start,
        snora::SheetEdge::End => L::End,
    }
}

fn sheet_edge_to_real(x: l_sheet::SheetEdge) -> snora::SheetEdge {
    use l_sheet::SheetEdge as L;
    match x {
        L::Bottom => snora::SheetEdge::Bottom,
        L::Top => snora::SheetEdge::Top,
        L::Start => snora::SheetEdge::Start,
        L::End => snora::SheetEdge::End,
    }
}

fn sheet_size_to_listed(x: snora::SheetSize) -> l_sheet::SheetSize {
    use l_sheet::SheetSize as L;
    match x {
        snora::SheetSize::OneThird => L::OneThird,
        snora::SheetSize::Half => L::Half,
        snora::SheetSize::TwoThirds => L::TwoThirds,
        snora::SheetSize::Ratio(r) => L::Ratio(r),
        snora::SheetSize::Pixels(p) => L::Pixels(p),
    }
}

fn sheet_size_to_real(x: l_sheet::SheetSize) -> snora::SheetSize {
    use l_sheet::SheetSize as L;
    match x {
        L::OneThird => snora::SheetSize::OneThird,
        L::Half => snora::SheetSize::Half,
        L::TwoThirds => snora::SheetSize::TwoThirds,
        L::Ratio(r) => snora::SheetSize::Ratio(r),
        L::Pixels(p) => snora::SheetSize::Pixels(p),
    }
}

// The icon: with the two features its variants are behind, as CI builds this crate (`--all-features`).
#[rustfmt::skip]
mod l_icon {
// ANCHOR: vocab_icon
pub enum Icon {
    Text(String),
    #[cfg(feature = "lucide-icons")] Lucide(lucide_icons::Icon),
    #[cfg(feature = "svg-icons")]    Svg(std::path::PathBuf),
}
// ANCHOR_END: vocab_icon
}

#[rustfmt::skip]
mod l_icon_guide {
// ANCHOR: icons_enum
pub enum Icon {
    Text(String),                      // always available
    #[cfg(feature = "lucide-icons")]
    Lucide(lucide_icons::Icon),
    #[cfg(feature = "svg-icons")]
    Svg(std::path::PathBuf),
}
// ANCHOR_END: icons_enum
}

fn icon_to_listed(x: snora::Icon) -> (l_icon::Icon, l_icon_guide::Icon) {
    match x {
        snora::Icon::Text(s) => (l_icon::Icon::Text(s.clone()), l_icon_guide::Icon::Text(s)),
        #[cfg(feature = "lucide-icons")]
        snora::Icon::Lucide(i) => (l_icon::Icon::Lucide(i), l_icon_guide::Icon::Lucide(i)),
        #[cfg(feature = "svg-icons")]
        snora::Icon::Svg(p) => (l_icon::Icon::Svg(p.clone()), l_icon_guide::Icon::Svg(p)),
    }
}

fn icon_to_real(a: l_icon::Icon, b: l_icon_guide::Icon) -> (snora::Icon, snora::Icon) {
    (
        match a {
            l_icon::Icon::Text(s) => snora::Icon::Text(s),
            #[cfg(feature = "lucide-icons")]
            l_icon::Icon::Lucide(i) => snora::Icon::Lucide(i),
            #[cfg(feature = "svg-icons")]
            l_icon::Icon::Svg(p) => snora::Icon::Svg(p),
        },
        match b {
            l_icon_guide::Icon::Text(s) => snora::Icon::Text(s),
            #[cfg(feature = "lucide-icons")]
            l_icon_guide::Icon::Lucide(i) => snora::Icon::Lucide(i),
            #[cfg(feature = "svg-icons")]
            l_icon_guide::Icon::Svg(p) => snora::Icon::Svg(p),
        },
    )
}

#[rustfmt::skip]
mod l_menu {
// ANCHOR: vocab_menu_action
pub enum MenuAction<MenuId, MenuItemId> {
    MenuPressed(MenuId),
    MenuItemPressed { menu_id: MenuId, menu_item_id: MenuItemId },
}
// ANCHOR_END: vocab_menu_action
}

fn menu_action_to_listed<A, B>(x: snora::MenuAction<A, B>) -> l_menu::MenuAction<A, B> {
    use l_menu::MenuAction as L;
    match x {
        snora::MenuAction::MenuPressed(id) => L::MenuPressed(id),
        snora::MenuAction::MenuItemPressed {
            menu_id,
            menu_item_id,
        } => L::MenuItemPressed {
            menu_id,
            menu_item_id,
        },
    }
}

fn menu_action_to_real<A, B>(x: l_menu::MenuAction<A, B>) -> snora::MenuAction<A, B> {
    use l_menu::MenuAction as L;
    match x {
        L::MenuPressed(id) => snora::MenuAction::MenuPressed(id),
        L::MenuItemPressed {
            menu_id,
            menu_item_id,
        } => snora::MenuAction::MenuItemPressed {
            menu_id,
            menu_item_id,
        },
    }
}

#[rustfmt::skip]
mod l_tab {
use snora::Icon;
// ANCHOR: vocab_tab
pub struct Tab<TabId: Clone + PartialEq> {
    pub id: TabId,
    pub label: String,
    pub icon: Option<Icon>,
}

pub struct TabBar<TabId: Clone + PartialEq> {
    pub tabs: Vec<Tab<TabId>>,
    pub active: TabId,
}

pub enum TabAction<TabId> {
    Pressed(TabId),
}
// ANCHOR_END: vocab_tab
}

fn tab_bar_to_listed<T: Clone + PartialEq>(x: snora::TabBar<T>) -> l_tab::TabBar<T> {
    l_tab::TabBar {
        tabs: x
            .tabs
            .into_iter()
            .map(|t| l_tab::Tab {
                id: t.id,
                label: t.label,
                icon: t.icon,
            })
            .collect(),
        active: x.active,
    }
}

fn tab_bar_to_real<T: Clone + PartialEq>(x: l_tab::TabBar<T>) -> snora::TabBar<T> {
    snora::TabBar {
        tabs: x
            .tabs
            .into_iter()
            .map(|t| snora::Tab {
                id: t.id,
                label: t.label,
                icon: t.icon,
            })
            .collect(),
        active: x.active,
    }
}

fn tab_action_to_listed<T>(x: snora::TabAction<T>) -> l_tab::TabAction<T> {
    match x {
        snora::TabAction::Pressed(id) => l_tab::TabAction::Pressed(id),
    }
}

fn tab_action_to_real<T>(x: l_tab::TabAction<T>) -> snora::TabAction<T> {
    match x {
        l_tab::TabAction::Pressed(id) => snora::TabAction::Pressed(id),
    }
}

#[rustfmt::skip]
mod l_crumb {
// ANCHOR: vocab_crumb
pub struct Crumb<CrumbId: Clone> {
    pub id: CrumbId,
    pub label: String,
    pub is_leaf: bool,
}

pub enum BreadcrumbAction<CrumbId> {
    Pressed(CrumbId),
}
// ANCHOR_END: vocab_crumb
}

fn crumb_to_listed<T: Clone>(x: snora::Crumb<T>) -> l_crumb::Crumb<T> {
    l_crumb::Crumb {
        id: x.id,
        label: x.label,
        is_leaf: x.is_leaf,
    }
}

fn crumb_to_real<T: Clone>(x: l_crumb::Crumb<T>) -> snora::Crumb<T> {
    snora::Crumb {
        id: x.id,
        label: x.label,
        is_leaf: x.is_leaf,
    }
}

fn breadcrumb_action_to_listed<T>(x: snora::BreadcrumbAction<T>) -> l_crumb::BreadcrumbAction<T> {
    match x {
        snora::BreadcrumbAction::Pressed(id) => l_crumb::BreadcrumbAction::Pressed(id),
    }
}

fn breadcrumb_action_to_real<T>(x: l_crumb::BreadcrumbAction<T>) -> snora::BreadcrumbAction<T> {
    match x {
        l_crumb::BreadcrumbAction::Pressed(id) => snora::BreadcrumbAction::Pressed(id),
    }
}

#[rustfmt::skip]
mod l_cycle {
// ANCHOR: vocab_cycle
pub enum Cycle { Forward, Backward }
// ANCHOR_END: vocab_cycle
}

fn cycle_to_listed(x: snora::focus::Cycle) -> l_cycle::Cycle {
    match x {
        snora::focus::Cycle::Forward => l_cycle::Cycle::Forward,
        snora::focus::Cycle::Backward => l_cycle::Cycle::Backward,
    }
}

fn cycle_to_real(x: l_cycle::Cycle) -> snora::focus::Cycle {
    match x {
        l_cycle::Cycle::Forward => snora::focus::Cycle::Forward,
        l_cycle::Cycle::Backward => snora::focus::Cycle::Backward,
    }
}

// ---- design/tokens.md, design/typography.md --------------------------------------------------------------------------------------

#[rustfmt::skip]
mod l_tokens {
use snora::design::{Density, FocusTokens, Palette, Radius, Spacing, TargetTokens, Typography};
// ANCHOR: tokens_structure
pub struct Tokens {             // #[non_exhaustive]
    pub palette:    Palette,    // semantic color roles
    pub spacing:    Spacing,    // xs / sm / md / lg / xl / xxl
    pub typography: Typography, // body / title / heading / …
    pub radius:     Radius,     // sm / md / lg / pill
    pub focus:      FocusTokens,// ring_width / ring_offset / ring_color
    pub targets:    TargetTokens,// the minimum pointer target (0.55, RFC-115)
    pub density:    Density,    // *spacing* density — Comfortable; Compact deferred
}
// ANCHOR_END: tokens_structure
}

/// `Tokens` is `#[non_exhaustive]`: the fields are read (so each listed field exists, with its type), and a field the real
/// type gains is not seen from here.
fn tokens_to_listed(x: snora::design::Tokens) -> l_tokens::Tokens {
    let snora::design::Tokens {
        palette,
        spacing,
        typography,
        radius,
        focus,
        targets,
        density,
        ..
    } = x;
    l_tokens::Tokens {
        palette,
        spacing,
        typography,
        radius,
        focus,
        targets,
        density,
    }
}

#[rustfmt::skip]
mod l_typography {
// ANCHOR: typography_structure
pub struct Typography {
    pub body: TextRole,        // ordinary explanatory text
    pub body_small: TextRole,  // secondary metadata, compact help
    pub label: TextRole,       // button, field and chip labels
    pub title: TextRole,       // card / dialog / notice title
    pub heading: TextRole,     // page or section heading
    pub display: TextRole,     // rare major page title
}

pub struct TextRole {
    pub size: f32,        // logical pixels
    pub line_height: f32, // multiplier, e.g. 1.4 = 140% of size
}
// ANCHOR_END: typography_structure
}

fn typography_to_listed(x: snora::design::Typography) -> l_typography::Typography {
    let snora::design::Typography {
        body,
        body_small,
        label,
        title,
        heading,
        display,
        ..
    } = x;
    let role = |r: snora::design::TextRole| l_typography::TextRole {
        size: r.size,
        line_height: r.line_height,
    };
    l_typography::Typography {
        body: role(body),
        body_small: role(body_small),
        label: role(label),
        title: role(title),
        heading: role(heading),
        display: role(display),
    }
}

// ---- contributing/semantic-accessibility.md: iced's `button::Status`, which snora has no say in -------------------------------

#[rustfmt::skip]
mod l_status {
// ANCHOR: iced_button_status
pub enum Status {
    Active,
    Hovered,
    Pressed,
    Disabled,
    // No Focused variant.
}
// ANCHOR_END: iced_button_status
}

fn status_to_listed(x: iced::widget::button::Status) -> l_status::Status {
    use iced::widget::button::Status as R;
    match x {
        R::Active => l_status::Status::Active,
        R::Hovered => l_status::Status::Hovered,
        R::Pressed => l_status::Status::Pressed,
        R::Disabled => l_status::Status::Disabled,
    }
}

fn status_to_real(x: l_status::Status) -> iced::widget::button::Status {
    use iced::widget::button::Status as R;
    match x {
        l_status::Status::Active => R::Active,
        l_status::Status::Hovered => R::Hovered,
        l_status::Status::Pressed => R::Pressed,
        l_status::Status::Disabled => R::Disabled,
    }
}
