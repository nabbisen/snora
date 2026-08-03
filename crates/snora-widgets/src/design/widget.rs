//! Token-derived chrome geometry for Snora Design (RFC-040).
//!
//! RFC-038 made chrome *colours* follow the emitted theme, because the
//! prefab widgets already read `theme.extended_palette()`. Geometry does
//! not follow, and cannot — an `iced::Theme` carries no spacing or
//! radius. This module adds a styled variant of each chrome widget that
//! takes `&Tokens` and maps its numeric literals (padding, inter-element
//! gaps, corner radii) to the [`Spacing`]/[`Radius`] scales, leaving the
//! unstyled `snora::widget::*` set exactly as it renders today.
//!
//! # One implementation, two geometry sources
//!
//! Every widget's body is written exactly once — in its home module
//! (`crate::header`, `crate::footer`, `crate::sidebar`, `crate::tab`,
//! `crate::crumb`), as a `pub(crate) fn build_*` taking a small geometry
//! struct. The unstyled `snora::widget::*` function passes that struct's
//! `::unstyled()` constructor (today's literals, unchanged); the styled
//! functions here pass a token-derived one instead. Drift between the two
//! is structurally impossible, not merely discouraged: there is nowhere
//! for a second copy of a widget body to live.
//!
//! # The mapping (Step 3 — deliberate, not reverse-engineered)
//!
//! `Spacing::comfortable()` is `{xs: 4, sm: 8, md: 12, lg: 16, xl: 24,
//! xxl: 32}`; `Radius::default_roles()` is `{sm: 4, md: 6, lg: 10, pill:
//! 999}`. Each geometry value below is mapped to the token whose own
//! documented semantic fits it, not to whichever token happens to equal
//! today's number — several *do* land on today's exact literal, because
//! the original hardcoded numbers already followed something close to
//! this scale; that is stated per-value below, not hidden.
//!
//! | Widget | Value | Today | Token | Exact? | Why |
//! |---|---|---|---|---|---|
//! | header | `pad_y` | 8 | `Spacing::sm` | yes | "compact internal gap" fits a shallow bar inset |
//! | header | `pad_x` | 16 | `Spacing::lg` | yes | "section spacing" fits outer chrome padding |
//! | header | `gap` | 12 | `Spacing::md` | yes | "ordinary component gap" between sibling controls |
//! | header | `radius` | 0 | `Radius::sm` | **no** | RFC-040's stated target — see "Chrome radius" below |
//! | footer | `pad_x` | 16 | `Spacing::lg` | yes | same role as header's `pad_x` |
//! | footer | `pad_y` | 6 | `Spacing::sm` | **no** | 6 has no clean token; `sm` chosen for cross-bar rhythm with header |
//! | footer | `radius` | 0 | `Radius::sm` | **no** | same as header |
//! | sidebar | `gap` | 16 | `Spacing::md` | **no** | reclassified: sibling icon buttons in one list read as "ordinary component gap", not "section spacing" — deliberately not reproducing 16 |
//! | sidebar | `padding` | 16 | `Spacing::lg` | yes | rail's outer edge padding, same role as header/footer `pad_x` |
//! | sidebar | `button_radius` | 6 | `Radius::md` | yes | `Radius::md` is documented "buttons, chips, notices" |
//! | tab | `bar_gap` | 2 | `Spacing::xs` | **no** | no clean equivalent; `xs` is the smallest available |
//! | tab | `bar_pad_x` | 12 | `Spacing::md` | yes | ordinary component padding |
//! | tab | `content_gap` | 6 | `Spacing::sm` | **no** | shared "icon-label inline gap" rhythm with menu/crumb, below |
//! | tab | `tab_pad_x` | 12 | `Spacing::md` | yes | button horizontal padding |
//! | tab | `tab_pad_y` | 8 | `Spacing::sm` | yes | button vertical padding, smaller than horizontal |
//! | tab | `bar_border_radius` | 0 | `Radius::sm` | **no** | RFC-040's stated target |
//! | crumb | `gap` | 6 | `Spacing::sm` | **no** | shared inline-gap rhythm (see below) |
//! | crumb | `row_pad_x` | 12 | `Spacing::md` | yes | trail's horizontal padding |
//! | crumb | `row_pad_y` | 4 | `Spacing::xs` | yes | trail's shallow vertical padding |
//! | crumb | `btn_pad_x` | 4 | `Spacing::xs` | yes | per-crumb button horizontal padding |
//! | crumb | `btn_pad_y` | 2 | `Spacing::xs` | **no** | no token smaller than `xs`; used as the floor |
//! | crumb | `btn_radius` | 3 | `Radius::sm` | **no** | `sm` is the smallest radius available |
//! | menu | `gap` | 6 | `Spacing::sm` | **no** | shared inline-gap rhythm (see below) |
//!
//! **Shared inline-gap rhythm.** Tab's icon-to-label gap, crumb's
//! item-to-separator gap, and menu's icon-to-label gap were all
//! independently hardcoded at `6` — the same value, for the same kind of
//! thing (a compact gap between two adjacent inline elements), in three
//! unrelated files. All three map to `Spacing::sm` (8): `sm` is
//! literally documented as "compact internal gap", the closest semantic
//! fit, and using it uniformly gives these three sites one shared rhythm
//! instead of an accidentally-identical-but-unrelated `6` — which is the
//! whole point of a token scale, per RFC-040's own framing.
//!
//! **Chrome radius.** `header`, `footer`, and the tab bar's own border
//! all hardcode `radius: 0.0` — square corners — which RFC-040 names
//! directly as "a large part of why stock snora chrome reads as flat and
//! dated". All three map to `Radius::sm` (4): the smallest available
//! radius, a modest rounding appropriate for a full-width chrome strip
//! rather than the more pronounced rounding `Radius::lg` (cards) or
//! `Radius::md` (buttons) would give.
//!
//! **Unmapped literals** (no `Spacing`/`Radius` equivalent, left as
//! literal constants, identical in both the unstyled and styled paths):
//! sidebar's and crumb's button border *widths* (`0.0` — "no border" is
//! not expressible on either scale); the tab bar container's border
//! *width* (`1.0` — border widths aren't part of either scale, same as
//! `style.rs`'s own `chrome_container_style` border width); the tab
//! bar's vertical padding (`0.0` — structural: tabs supply their own
//! vertical padding, this is an absence of a value, not a design
//! literal).
//!
//! # Density (RFC-040 Q-1)
//!
//! Every function below reads geometry from `tokens.spacing`/
//! `tokens.radius` directly — **not** by branching on `tokens.density`.
//! This is deliberate, not an oversight: `snora_design::Density::Compact`
//! is documented as "reserved; not resolved" — `Spacing` has only a
//! `comfortable()` constructor, no compact scale exists to select. Adding
//! one would be inventing token values RFC-040 does not authorize (its
//! own scope: no new token roles or scales). Reading `tokens.spacing`/
//! `tokens.radius` directly means this module is already
//! density-*correct* in the sense the owner asked for: whatever a future
//! compact scale resolves to (or a hand-mutated custom `Tokens` supplies
//! today) flows through unchanged, with no widget-level branch to keep in
//! sync. `widget/tests.rs` verifies this plumbing with a hand-mutated
//! `Tokens` pair, since no built-in preset currently offers two distinct
//! `Spacing` values to compare.
//!
//! [`Spacing`]: snora_design::Spacing
//! [`Radius`]: snora_design::Radius

use iced::Element;
use snora_core::{
    BreadcrumbAction, Crumb, LayoutDirection, Menu, MenuAction, SideBar, TabAction, TabBar,
};
use snora_design::Tokens;
use std::fmt::Debug;

use crate::crumb::{CrumbGeometry, build_breadcrumb};
use crate::footer::{FooterGeometry, build_footer};
use crate::header::{HeaderGeometry, build_header};
use crate::sidebar::{SideBarGeometry, build_side_bar};
use crate::tab::{TabGeometry, build_tab_bar};

/// Token-derived styled variant of [`crate::app_header`] (RFC-040).
/// Colors already follow the theme (RFC-038); this maps geometry —
/// padding, gap, and corner radius — to `tokens.spacing`/`tokens.radius`.
/// See the module documentation for the full mapping table.
#[allow(clippy::too_many_arguments)]
pub fn app_header<'a, Message, MenuId, MenuItemId, F>(
    tokens: &Tokens,
    title: &'a str,
    menus: Vec<Menu<MenuId, MenuItemId>>,
    on_menu_action: &'a F,
    active_menu_id: Option<&MenuId>,
    end_controls: Option<Element<'a, Message>>,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    MenuId: Clone + Debug + PartialEq + 'a,
    MenuItemId: Clone + Debug + 'a,
    F: Fn(MenuAction<MenuId, MenuItemId>) -> Message + 'a,
{
    build_header(
        title,
        menus,
        on_menu_action,
        active_menu_id,
        end_controls,
        direction,
        header_geometry(tokens),
    )
}

fn header_geometry(tokens: &Tokens) -> HeaderGeometry {
    HeaderGeometry {
        gap: tokens.spacing.md,
        pad_x: tokens.spacing.lg,
        pad_y: tokens.spacing.sm,
        radius: tokens.radius.sm,
        menu_gap: tokens.spacing.sm,
    }
}

/// Token-derived styled variant of [`crate::app_footer`] (RFC-040).
pub fn app_footer<'a, Message>(
    tokens: &Tokens,
    content: Element<'a, Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    build_footer(content, footer_geometry(tokens))
}

fn footer_geometry(tokens: &Tokens) -> FooterGeometry {
    FooterGeometry {
        pad_x: tokens.spacing.lg,
        pad_y: tokens.spacing.sm,
        radius: tokens.radius.sm,
    }
}

/// Token-derived styled variant of [`crate::app_side_bar`] (RFC-040).
pub fn app_side_bar<'a, Message, ViewId>(
    tokens: &Tokens,
    side_bar: SideBar<Message, ViewId>,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    ViewId: Clone + PartialEq + 'a,
{
    build_side_bar(side_bar, direction, side_bar_geometry(tokens))
}

fn side_bar_geometry(tokens: &Tokens) -> SideBarGeometry {
    SideBarGeometry {
        gap: tokens.spacing.md,
        padding: tokens.spacing.lg,
        button_radius: tokens.radius.md,
    }
}

/// Token-derived styled variant of [`crate::app_tab_bar`] (RFC-040).
pub fn app_tab_bar<'a, Message, TabId, F>(
    tokens: &Tokens,
    bar: TabBar<TabId>,
    on_action: &'a F,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    TabId: Clone + Debug + PartialEq + 'a,
    F: Fn(TabAction<TabId>) -> Message + 'a,
{
    build_tab_bar(bar, on_action, direction, tab_geometry(tokens))
}

fn tab_geometry(tokens: &Tokens) -> TabGeometry {
    TabGeometry {
        bar_gap: tokens.spacing.xs,
        bar_pad_x: tokens.spacing.md,
        content_gap: tokens.spacing.sm,
        tab_pad_x: tokens.spacing.md,
        tab_pad_y: tokens.spacing.sm,
        bar_border_radius: tokens.radius.sm,
    }
}

/// Token-derived styled variant of [`crate::app_breadcrumb`] (RFC-040).
pub fn app_breadcrumb<'a, Message, CrumbId, F>(
    tokens: &Tokens,
    crumbs: Vec<Crumb<CrumbId>>,
    on_action: &'a F,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    CrumbId: Clone + Debug + 'a,
    F: Fn(BreadcrumbAction<CrumbId>) -> Message + 'a,
{
    build_breadcrumb(crumbs, on_action, direction, breadcrumb_geometry(tokens))
}

fn breadcrumb_geometry(tokens: &Tokens) -> CrumbGeometry {
    CrumbGeometry {
        gap: tokens.spacing.sm,
        row_pad_x: tokens.spacing.md,
        row_pad_y: tokens.spacing.xs,
        btn_pad_x: tokens.spacing.xs,
        btn_pad_y: tokens.spacing.xs,
        btn_radius: tokens.radius.sm,
    }
}

#[cfg(test)]
mod tests;
